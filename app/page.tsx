"use client";

import { useEffect, useState, useCallback, useMemo } from "react";
import { useRouter } from "next/navigation";
import { useSettingsStore } from "./stores/settingsStore";
import { useProjectStore } from "./stores/projectStore";
import { ProjectCard } from "./components/project-card";
import { InitPreviewModal } from "./components/init-preview-modal";
import { SettingsDrawer } from "./components/settings-drawer";
import { ProjectSummary, ProjectMeta, GitInfo, PromptHistoryEntry } from "./lib/types";
import { getGitInfo, generateOpenCodeLaunchPrompt, launchOpenCode, listPromptHistory, readPromptHistoryFile } from "./lib/tauri";

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function TauriUnavailableBanner() {
  return (
    <div className="mx-6 mt-4 p-3 bg-amber-900/20 border border-amber-700/50 rounded-lg">
      <p className="text-sm text-amber-400">
        <span className="font-medium">Frontend-only mode:</span> Some features are limited. Run{" "}
        <code className="text-amber-300">npm run tauri:dev</code> for full functionality.
      </p>
    </div>
  );
}

function ErrorBanner({ message, onDismiss }: { message: string; onDismiss: () => void }) {
  return (
    <div className="mx-6 mt-4 p-3 bg-red-900/20 border border-red-700/50 rounded-lg flex items-center justify-between">
      <p className="text-sm text-red-400">{message}</p>
      <button
        onClick={onDismiss}
        className="ml-4 px-3 py-1 text-xs text-red-400 hover:text-red-300 border border-red-700/50 rounded-lg hover:bg-red-900/30 transition-colors"
      >
        Dismiss
      </button>
    </div>
  );
}

function EmptyState({ message, submessage }: { message: string; submessage?: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-20 px-8 text-center">
      <div className="w-12 h-12 mb-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50 flex items-center justify-center">
        <svg className="w-6 h-6 text-zinc-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
      </div>
      <p className="text-zinc-400 font-medium">{message}</p>
      {submessage && <p className="text-sm text-zinc-600 mt-1 max-w-sm">{submessage}</p>}
    </div>
  );
}

function GitInfoPanel({ gitInfo, error }: { gitInfo: GitInfo | null; error: string | null }) {
  if (error) {
    return (
      <div className="border border-zinc-800/80 rounded-xl bg-zinc-900/40 p-4">
        <div className="flex items-center gap-2 text-red-400 text-sm">
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span>{error}</span>
        </div>
      </div>
    );
  }

  if (!gitInfo) {
    return (
      <div className="border border-zinc-800/80 rounded-xl bg-zinc-900/40 p-4">
        <div className="flex items-center gap-2 text-zinc-500 text-sm">
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
          </svg>
          <span>Not a Git repository</span>
        </div>
      </div>
    );
  }

  return (
    <div className="border border-zinc-800/80 rounded-xl bg-zinc-900/40 overflow-hidden">
      <div className="px-4 py-3 border-b border-zinc-800/60 bg-zinc-900/60">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <svg className="w-4 h-4 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
            <span className="text-sm font-medium text-zinc-200">Git Status</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="px-2 py-0.5 text-xs bg-zinc-800 text-zinc-300 rounded border border-zinc-700 font-mono">
              {gitInfo.branch || "unknown"}
            </span>
            {gitInfo.is_clean ? (
              <span className="px-2 py-0.5 text-xs bg-emerald-900/30 text-emerald-400 rounded border border-emerald-800/50">
                Clean
              </span>
            ) : (
              <span className="px-2 py-0.5 text-xs bg-amber-900/30 text-amber-400 rounded border border-amber-800/50">
                {gitInfo.changed_files_count} changed
              </span>
            )}
          </div>
        </div>
      </div>

      <div className="p-4 space-y-4">
        {gitInfo.last_commit_message && (
          <div>
            <h4 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-1">Last commit</h4>
            <div className="flex items-start gap-2">
              <code className="text-xs text-blue-400 font-mono">{gitInfo.last_commit_hash?.slice(0, 7)}</code>
              <span className="text-sm text-zinc-300">{gitInfo.last_commit_message}</span>
            </div>
            {gitInfo.last_commit_date && (
              <p className="text-xs text-zinc-600 mt-1">{gitInfo.last_commit_date}</p>
            )}
          </div>
        )}

        {!gitInfo.is_clean && (
          <div>
            <h4 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-2">Changes</h4>
            <div className="space-y-1">
              {gitInfo.staged_files.length > 0 && (
                <div>
                  <span className="text-xs text-emerald-400">Staged ({gitInfo.staged_files.length})</span>
                  <div className="mt-1 space-y-0.5">
                    {gitInfo.staged_files.slice(0, 5).map((f) => (
                      <div key={f} className="text-xs text-zinc-400 font-mono pl-2">{f}</div>
                    ))}
                    {gitInfo.staged_files.length > 5 && (
                      <div className="text-xs text-zinc-600 pl-2">+{gitInfo.staged_files.length - 5} more</div>
                    )}
                  </div>
                </div>
              )}
              {gitInfo.unstaged_files.length > 0 && (
                <div>
                  <span className="text-xs text-amber-400">Modified ({gitInfo.unstaged_files.length})</span>
                  <div className="mt-1 space-y-0.5">
                    {gitInfo.unstaged_files.slice(0, 5).map((f) => (
                      <div key={f} className="text-xs text-zinc-400 font-mono pl-2">{f}</div>
                    ))}
                    {gitInfo.unstaged_files.length > 5 && (
                      <div className="text-xs text-zinc-600 pl-2">+{gitInfo.unstaged_files.length - 5} more</div>
                    )}
                  </div>
                </div>
              )}
              {gitInfo.untracked_files.length > 0 && (
                <div>
                  <span className="text-xs text-zinc-500">Untracked ({gitInfo.untracked_files.length})</span>
                  <div className="mt-1 space-y-0.5">
                    {gitInfo.untracked_files.slice(0, 5).map((f) => (
                      <div key={f} className="text-xs text-zinc-500 font-mono pl-2">{f}</div>
                    ))}
                    {gitInfo.untracked_files.length > 5 && (
                      <div className="text-xs text-zinc-600 pl-2">+{gitInfo.untracked_files.length - 5} more</div>
                    )}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {gitInfo.recent_commits.length > 0 && (
          <div>
            <h4 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-2">Recent commits</h4>
            <div className="space-y-1">
              {gitInfo.recent_commits.slice(0, 3).map((commit) => (
                <div key={commit.hash} className="flex items-start gap-2">
                  <code className="text-xs text-blue-400 font-mono shrink-0">{commit.hash.slice(0, 7)}</code>
                  <div className="min-w-0">
                    <p className="text-xs text-zinc-300 truncate">{commit.message}</p>
                    <p className="text-xs text-zinc-600">{commit.date}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function ProjectDetailPanel({
  project,
  contextFiles,
  gitInfo,
  gitError,
  onClose,
  onSave,
  savingFiles,
  fileStates,
  onFileChange,
  tauriAvailable,
  isLoadingGit,
  launchPromptContent,
  promptStatus,
  onGenerateLaunchPrompt,
  onCopyPrompt,
  onLaunchOpenCode,
  isGeneratingPrompt,
  requestedTask,
  onRequestedTaskChange,
  promptHistory,
  isLoadingPromptHistory,
  selectedHistoryContent,
  selectedHistoryFilename,
  onViewHistoryFile,
  onCopyHistoryFile,
  onCloseHistoryPreview,
  historyStatus,
}: {
  project: ProjectSummary;
  contextFiles: { meta: ProjectMeta | null; current: string | null; architecture: string | null; recent_work: string | null } | null;
  gitInfo: GitInfo | null;
  gitError: string | null;
  onClose: () => void;
  onSave: (filename: string) => void;
  savingFiles: Record<string, boolean>;
  fileStates: Record<string, { content: string; originalContent: string; isDirty: boolean; lastSaved: string | null; error: string | null }>;
  onFileChange: (key: string, value: string) => void;
  tauriAvailable: boolean;
  isLoadingGit: boolean;
  launchPromptContent: string | null;
  promptStatus: string | null;
  onGenerateLaunchPrompt: () => void;
  onCopyPrompt: () => void;
  onLaunchOpenCode: () => void;
  isGeneratingPrompt: boolean;
  requestedTask: string;
  onRequestedTaskChange: (value: string) => void;
  promptHistory: PromptHistoryEntry[];
  isLoadingPromptHistory: boolean;
  selectedHistoryContent: string | null;
  selectedHistoryFilename: string | null;
  onViewHistoryFile: (entry: PromptHistoryEntry) => void;
  onCopyHistoryFile: (entry: PromptHistoryEntry) => void;
  onCloseHistoryPreview: () => void;
  historyStatus: string | null;
}) {
  const typeColors: Record<string, string> = {
    nextjs: "bg-blue-900/30 text-blue-400 border-blue-800/50",
    node: "bg-green-900/30 text-green-400 border-green-800/50",
    python: "bg-yellow-900/30 text-yellow-400 border-yellow-800/50",
    rust: "bg-orange-900/30 text-orange-400 border-orange-800/50",
    unknown: "bg-zinc-800 text-zinc-400 border-zinc-700",
  };

  const typeLabels: Record<string, string> = {
    nextjs: "Next.js",
    node: "Node",
    python: "Python",
    rust: "Rust",
    unknown: "Project",
  };

  if (!contextFiles?.meta) {
    return (
      <div className="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4">
        <div className="w-full max-w-2xl border border-zinc-700/80 rounded-xl bg-zinc-900 p-6">
          <p className="text-zinc-400">No context found for this project.</p>
          <button onClick={onClose} className="mt-4 px-4 py-2 text-sm text-zinc-400 hover:text-zinc-200 border border-zinc-700 rounded-lg">
            Close
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 z-50 bg-black/80 flex items-start justify-center p-4 overflow-y-auto">
      <div className="w-full max-w-2xl border border-zinc-700/80 rounded-xl bg-zinc-900 my-8 flex flex-col max-h-[calc(100vh-4rem)]">
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800/60 shrink-0">
          <div className="flex items-center gap-3">
            <button
              onClick={onClose}
              className="px-3 py-1.5 text-xs text-zinc-400 hover:text-zinc-200 border border-zinc-700/60 rounded-lg hover:bg-zinc-800/50 transition-colors"
            >
              ← Back
            </button>
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-lg font-semibold text-zinc-100">{contextFiles.meta.name}</h2>
                <span className={`px-2 py-0.5 text-xs rounded-md border ${typeColors[contextFiles.meta.project_type] || typeColors.unknown}`}>
                  {typeLabels[contextFiles.meta.project_type] || "Project"}
                </span>
              </div>
              <p className="text-xs text-zinc-500 font-mono mt-0.5">{contextFiles.meta.path}</p>
            </div>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {isLoadingGit ? (
            <div className="border border-zinc-800/80 rounded-xl bg-zinc-900/40 p-4 flex items-center gap-3">
              <svg className="animate-spin w-4 h-4 text-zinc-500" viewBox="0 0 24 24" fill="none">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
              </svg>
              <span className="text-sm text-zinc-500">Loading Git info...</span>
            </div>
          ) : (
            <GitInfoPanel gitInfo={gitInfo} error={gitError} />
          )}

          <div className="border border-zinc-800/80 rounded-xl bg-zinc-900/40 overflow-hidden">
            <div className="px-4 py-3 border-b border-zinc-800/60 bg-zinc-900/60">
              <div className="flex items-center gap-2">
                <svg className="w-4 h-4 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
                <span className="text-sm font-medium text-zinc-200">OpenCode</span>
              </div>
            </div>
            <div className="p-4 space-y-3">
              <div className="flex items-center gap-2 flex-wrap">
                <button
                  onClick={onGenerateLaunchPrompt}
                  disabled={!tauriAvailable || isGeneratingPrompt}
                  className="px-3 py-1.5 text-xs bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
                >
                  {isGeneratingPrompt ? (
                    <>
                      <svg className="animate-spin w-3 h-3" viewBox="0 0 24 24" fill="none">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                      </svg>
                      Generating...
                    </>
                  ) : (
                    <>
                      <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                      </svg>
                      Generate launch prompt
                    </>
                  )}
                </button>
                <button
                  onClick={onCopyPrompt}
                  disabled={!tauriAvailable || !launchPromptContent}
                  className="px-3 py-1.5 text-xs bg-zinc-700 hover:bg-zinc-600 text-zinc-200 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
                >
                  <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  Copy prompt
                </button>
                <button
                  onClick={onLaunchOpenCode}
                  disabled={!tauriAvailable || !launchPromptContent}
                  className="px-3 py-1.5 text-xs bg-emerald-700 hover:bg-emerald-600 text-emerald-100 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
                >
                  <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                  Launch OpenCode
                </button>
              </div>
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-zinc-400">Requested Task</label>
                <textarea
                  value={requestedTask}
                  onChange={(e) => onRequestedTaskChange(e.target.value)}
                  rows={3}
                  placeholder="What should OpenCode do?"
                  className="w-full px-3 py-2 text-xs bg-zinc-800/50 border border-zinc-700/60 rounded-lg text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-zinc-600 resize-none"
                />
              </div>
              {promptStatus && (
                <div className={`text-xs px-3 py-2 rounded-lg ${promptStatus.includes("Error") || promptStatus.includes("error") ? "bg-red-900/30 text-red-400" : "bg-emerald-900/30 text-emerald-400"}`}>
                  {promptStatus}
                </div>
              )}
              {launchPromptContent && (
                <div className="mt-2 text-xs text-zinc-500">
                  Prompt saved at: <code className="text-zinc-400">.context-bridge/launch-prompt.md</code>
                </div>
              )}

              {isLoadingPromptHistory ? (
                <div className="border-t border-zinc-800/60 pt-3 mt-3">
                  <div className="flex items-center gap-2 text-xs text-zinc-500">
                    <svg className="animate-spin w-3 h-3" viewBox="0 0 24 24" fill="none">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                    </svg>
                    Loading prompt history...
                  </div>
                </div>
              ) : promptHistory.length > 0 && (
                <div className="border-t border-zinc-800/60 pt-3 mt-3">
                  <h4 className="text-xs font-medium text-zinc-400 mb-2">Prompt History</h4>
                  <div className="space-y-2 max-h-48 overflow-y-auto">
                    {promptHistory.map((entry) => (
                      <div
                        key={entry.filename}
                        className="flex items-center justify-between px-3 py-2 rounded-lg bg-zinc-800/40 border border-zinc-700/40"
                      >
                        <div className="min-w-0 flex-1">
                          <div className="text-xs text-zinc-300 font-mono truncate">{entry.filename}</div>
                          <div className="flex items-center gap-2 mt-0.5">
                            <span className="text-xs text-zinc-500">{formatFileSize(entry.size_bytes)}</span>
                            <span className="text-xs text-zinc-600">{entry.modified}</span>
                          </div>
                        </div>
                        <div className="flex items-center gap-1.5 ml-3 shrink-0">
                          <button
                            onClick={() => onViewHistoryFile(entry)}
                            className="px-2 py-1 text-xs text-zinc-400 hover:text-zinc-200 border border-zinc-700/60 rounded hover:bg-zinc-700/40 transition-colors"
                          >
                            View
                          </button>
                          <button
                            onClick={() => onCopyHistoryFile(entry)}
                            className="px-2 py-1 text-xs text-zinc-400 hover:text-zinc-200 border border-zinc-700/60 rounded hover:bg-zinc-700/40 transition-colors"
                          >
                            Copy
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {historyStatus && (
                <div className={`text-xs px-3 py-2 rounded-lg ${historyStatus.startsWith("Error") ? "bg-red-900/30 text-red-400" : "bg-emerald-900/30 text-emerald-400"}`}>
                  {historyStatus}
                </div>
              )}

              {selectedHistoryContent !== null && (
                <div className="border-t border-zinc-800/60 pt-3 mt-3">
                  <div className="flex items-center justify-between mb-2">
                    <h4 className="text-xs font-medium text-zinc-400 truncate max-w-[80%]">
                      Preview: {selectedHistoryFilename}
                    </h4>
                    <button
                      onClick={onCloseHistoryPreview}
                      className="px-2 py-1 text-xs text-zinc-400 hover:text-zinc-200 border border-zinc-700/60 rounded hover:bg-zinc-700/40 transition-colors"
                    >
                      Close
                    </button>
                  </div>
                  <pre className="w-full max-h-64 overflow-auto bg-zinc-950/80 border border-zinc-700/60 rounded-lg p-3 text-xs text-zinc-300 font-mono whitespace-pre-wrap break-all">
                    {selectedHistoryContent}
                  </pre>
                </div>
              )}
            </div>
          </div>

          <FileEditorInline
            title="Current Focus"
            filename="current.md"
            value={fileStates.current?.content || ""}
            onChange={(v) => onFileChange("current", v)}
            onSave={() => onSave("current.md")}
            isSaving={savingFiles.current}
            isDirty={fileStates.current?.isDirty || false}
            lastSaved={fileStates.current?.lastSaved}
            error={fileStates.current?.error}
            disabled={!tauriAvailable}
          />

          <FileEditorInline
            title="Architecture"
            filename="architecture.md"
            value={fileStates.architecture?.content || ""}
            onChange={(v) => onFileChange("architecture", v)}
            onSave={() => onSave("architecture.md")}
            isSaving={savingFiles.architecture}
            isDirty={fileStates.architecture?.isDirty || false}
            lastSaved={fileStates.architecture?.lastSaved}
            error={fileStates.architecture?.error}
            disabled={!tauriAvailable}
          />

          <FileEditorInline
            title="Recent Work"
            filename="recent-work.md"
            value={fileStates.recentWork?.content || ""}
            onChange={(v) => onFileChange("recentWork", v)}
            onSave={() => onSave("recent-work.md")}
            isSaving={savingFiles.recentWork}
            isDirty={fileStates.recentWork?.isDirty || false}
            lastSaved={fileStates.recentWork?.lastSaved}
            error={fileStates.recentWork?.error}
            disabled={!tauriAvailable}
          />
        </div>
      </div>
    </div>
  );
}

function FileEditorInline({
  title,
  filename,
  value,
  onChange,
  onSave,
  isSaving,
  isDirty,
  lastSaved,
  error,
  disabled,
}: {
  title: string;
  filename: string;
  value: string;
  onChange: (value: string) => void;
  onSave: () => void;
  isSaving: boolean;
  isDirty: boolean;
  lastSaved: string | null;
  error: string | null;
  disabled: boolean;
}) {
  return (
    <div className="border border-zinc-800/80 rounded-xl bg-zinc-900/40 overflow-hidden">
      <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-800/60 bg-zinc-900/60">
        <div className="flex items-center gap-2">
          <h3 className="text-sm font-medium text-zinc-200">{title}</h3>
          <span className="text-xs text-zinc-600 font-mono">{filename}</span>
        </div>
        <div className="flex items-center gap-3">
          {lastSaved && <span className="text-xs text-zinc-500">Saved {lastSaved}</span>}
          {isDirty && !disabled && (
            <span className="px-2 py-0.5 text-xs bg-amber-900/30 text-amber-400 rounded border border-amber-800/50">Unsaved</span>
          )}
          <button
            onClick={onSave}
            disabled={disabled || isSaving || !isDirty}
            className="px-3 py-1.5 text-xs bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSaving ? "Saving..." : "Save"}
          </button>
        </div>
      </div>
      {error && (
        <div className="px-4 py-2 bg-red-900/20 border-b border-red-800/50">
          <p className="text-xs text-red-400">{error}</p>
        </div>
      )}
      <textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled}
        className="w-full min-h-[150px] p-4 bg-zinc-950/50 text-zinc-300 text-sm font-mono resize-y focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
      />
    </div>
  );
}

function DashboardHeader({
  rootPath,
  onRescan,
  isScanning,
  onOpenSettings
}: {
  rootPath: string;
  onRescan: () => void;
  isScanning: boolean;
  onOpenSettings: () => void;
}) {
  return (
    <header className="flex items-center justify-between px-6 py-4 border-b border-zinc-800/60 shrink-0">
      <div className="flex items-center gap-3">
        <div>
          <h1 className="text-base font-semibold text-zinc-100">ContextDock</h1>
          <p className="text-xs text-zinc-500 font-mono mt-0.5 truncate max-w-md">{rootPath}</p>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <button
          onClick={onRescan}
          disabled={isScanning}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs text-zinc-400 hover:text-zinc-200 border border-zinc-700/60 rounded-lg hover:bg-zinc-800/50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isScanning ? (
            <>
              <svg className="animate-spin w-3 h-3" viewBox="0 0 24 24" fill="none">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
              </svg>
              Scanning...
            </>
          ) : (
            <>
              <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              Refresh
            </>
          )}
        </button>
        <button
          onClick={onOpenSettings}
          className="px-3 py-1.5 text-xs text-zinc-400 hover:text-zinc-200 border border-zinc-700/60 rounded-lg hover:bg-zinc-800/50 transition-colors"
        >
          Settings
        </button>
      </div>
    </header>
  );
}

export default function Home() {
  const router = useRouter();
  const { settings, loadSettings } = useSettingsStore();
  const {
    projects,
    scanProjects,
    error,
    clearError,
    tauriAvailable,
    initPreview,
    getInitPreview,
    initContext,
    isInitializing,
    clearInitPreview,
    isLoading: isScanning,
    selectedProject,
    selectProject,
    loadContextFiles,
    contextFiles,
    updateContextFile,
  } = useProjectStore();

  const [isConfiguring, setIsConfiguring] = useState(false);
  const [pendingInitProject, setPendingInitProject] = useState<ProjectSummary | null>(null);
  const [initStarted, setInitStarted] = useState(false);

  const [showProjectDetail, setShowProjectDetail] = useState(false);
  const [detailFileStates, setDetailFileStates] = useState<Record<string, { content: string; originalContent: string; isDirty: boolean; lastSaved: string | null; error: string | null }>>({});
  const [savingFiles, setSavingFiles] = useState<Record<string, boolean>>({});

  const [gitInfo, setGitInfo] = useState<GitInfo | null>(null);
  const [isLoadingGit, setIsLoadingGit] = useState(false);
  const [gitError, setGitError] = useState<string | null>(null);

  const [launchPromptContent, setLaunchPromptContent] = useState<string | null>(null);
  const [isGeneratingPrompt, setIsGeneratingPrompt] = useState(false);
  const [promptStatus, setPromptStatus] = useState<string | null>(null);
  const [isLaunchingOpenCode, setIsLaunchingOpenCode] = useState(false);
  const defaultRequestedTask = "Continue from the Current Goal above. Implement the requested task described there using the project context, architecture notes, and recent work.";
  const [requestedTask, setRequestedTask] = useState(defaultRequestedTask);

  const [promptHistory, setPromptHistory] = useState<PromptHistoryEntry[]>([]);
  const [isLoadingPromptHistory, setIsLoadingPromptHistory] = useState(false);
  const [selectedHistoryContent, setSelectedHistoryContent] = useState<string | null>(null);
  const [selectedHistoryFilename, setSelectedHistoryFilename] = useState<string | null>(null);
  const [historyStatus, setHistoryStatus] = useState<string | null>(null);

  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  useEffect(() => {
    if (settings?.root_projects_path) {
      scanProjects(settings.root_projects_path);
    } else if (settings && !settings.root_projects_path) {
      setIsConfiguring(true);
    }
  }, [settings, scanProjects]);

  const handleRescan = useCallback(() => {
    if (settings?.root_projects_path && !isScanning) {
      scanProjects(settings.root_projects_path);
    }
  }, [settings, scanProjects, isScanning]);

  const handleProjectSelect = useCallback((project: ProjectSummary) => {
    selectProject(project);
    loadContextFiles(project.path);
    setShowProjectDetail(true);
    setGitInfo(null);
    setIsLoadingGit(true);
    setDetailFileStates({
      current: { content: "", originalContent: "", isDirty: false, lastSaved: null, error: null },
      architecture: { content: "", originalContent: "", isDirty: false, lastSaved: null, error: null },
      recentWork: { content: "", originalContent: "", isDirty: false, lastSaved: null, error: null },
    });

    getGitInfo(project.path)
      .then((info) => {
        setGitInfo(info);
        setGitError(null);
        setIsLoadingGit(false);
      })
      .catch((err) => {
        setGitInfo(null);
        setGitError("Failed to load Git info");
        setIsLoadingGit(false);
      });

    setIsLoadingPromptHistory(true);
    listPromptHistory(project.path)
      .then((entries) => setPromptHistory(entries))
      .catch(() => setPromptHistory([]))
      .finally(() => setIsLoadingPromptHistory(false));
  }, [selectProject, loadContextFiles]);

  const handleInitializeClick = useCallback(async (project: ProjectSummary) => {
    if (initStarted) return;
    setInitStarted(true);
    setPendingInitProject(project);
    await getInitPreview(project.path);
    setInitStarted(false);
  }, [initStarted, getInitPreview]);

  const handleInitConfirm = useCallback(async () => {
    if (!pendingInitProject || isInitializing) return;
    const result = await initContext(pendingInitProject.path);
    if (result) {
      setPendingInitProject(null);
      clearInitPreview();
      if (settings?.root_projects_path) {
        scanProjects(settings.root_projects_path);
      }
    }
  }, [pendingInitProject, isInitializing, initContext, clearInitPreview, settings, scanProjects]);

  const handleInitCancel = useCallback(() => {
    if (isInitializing) return;
    setPendingInitProject(null);
    clearInitPreview();
  }, [isInitializing, clearInitPreview]);

  useEffect(() => {
    if (contextFiles && showProjectDetail) {
      setDetailFileStates({
        current: {
          content: contextFiles.current || "",
          originalContent: contextFiles.current || "",
          isDirty: false,
          lastSaved: null,
          error: null,
        },
        architecture: {
          content: contextFiles.architecture || "",
          originalContent: contextFiles.architecture || "",
          isDirty: false,
          lastSaved: null,
          error: null,
        },
        recentWork: {
          content: contextFiles.recent_work || "",
          originalContent: contextFiles.recent_work || "",
          isDirty: false,
          lastSaved: null,
          error: null,
        },
      });
    }
  }, [contextFiles, showProjectDetail]);

  const handleDetailFileChange = useCallback((key: string, value: string) => {
    setDetailFileStates((prev) => ({
      ...prev,
      [key]: {
        ...prev[key],
        content: value,
        isDirty: value !== prev[key].originalContent,
        error: null,
      },
    }));
  }, []);

  const handleDetailSave = useCallback(async (filename: string) => {
    const keyMap: Record<string, string> = {
      "current.md": "current",
      "architecture.md": "architecture",
      "recent-work.md": "recentWork",
    };
    const fileKey = keyMap[filename];
    if (!fileKey || !selectedProject) return;

    setSavingFiles((prev) => ({ ...prev, [fileKey]: true }));

    try {
      await updateContextFile(selectedProject.path, filename, detailFileStates[fileKey].content);
      const now = new Date().toLocaleTimeString();
      setDetailFileStates((prev) => ({
        ...prev,
        [fileKey]: {
          ...prev[fileKey],
          originalContent: detailFileStates[fileKey].content,
          isDirty: false,
          lastSaved: now,
          error: null,
        },
      }));
    } catch (err) {
      setDetailFileStates((prev) => ({
        ...prev,
        [fileKey]: {
          ...prev[fileKey],
          error: `Failed to save: ${String(err)}`,
        },
      }));
    } finally {
      setSavingFiles((prev) => ({ ...prev, [fileKey]: false }));
    }
  }, [detailFileStates, selectedProject, updateContextFile]);

  const handleCloseProjectDetail = useCallback(() => {
    setShowProjectDetail(false);
    setGitInfo(null);
    setLaunchPromptContent(null);
    setPromptStatus(null);
    setRequestedTask(defaultRequestedTask);
    setPromptHistory([]);
    setSelectedHistoryContent(null);
    setSelectedHistoryFilename(null);
    setHistoryStatus(null);
    selectProject(null);
  }, [selectProject, defaultRequestedTask]);

  const handleGenerateLaunchPrompt = useCallback(async () => {
    if (!selectedProject || isGeneratingPrompt) return;
    setIsGeneratingPrompt(true);
    setPromptStatus(null);

    try {
      const task = requestedTask.trim() ? requestedTask : undefined;
      const result = await generateOpenCodeLaunchPrompt(selectedProject.path, task);
      setLaunchPromptContent(result.content);
      setPromptStatus("Launch prompt generated successfully!");

      listPromptHistory(selectedProject.path)
        .then((entries) => setPromptHistory(entries))
        .catch(() => {});
    } catch (err) {
      setPromptStatus(`Error: ${String(err)}`);
    } finally {
      setIsGeneratingPrompt(false);
    }
  }, [selectedProject, isGeneratingPrompt, requestedTask]);

  const handleCopyPrompt = useCallback(async () => {
    if (!launchPromptContent) return;

    try {
      await navigator.clipboard.writeText(launchPromptContent);
      setPromptStatus("Copied to clipboard!");
      setTimeout(() => setPromptStatus(null), 2000);
    } catch (err) {
      setPromptStatus(`Error copying: ${String(err)}`);
    }
  }, [launchPromptContent]);

  const handleLaunchOpenCode = useCallback(async () => {
    if (!selectedProject || isLaunchingOpenCode) return;
    setIsLaunchingOpenCode(true);
    setPromptStatus(null);

    try {
      await launchOpenCode(selectedProject.path);
      setPromptStatus("OpenCode launched!");
    } catch (err) {
      setPromptStatus(`Error: ${String(err)}`);
    } finally {
      setIsLaunchingOpenCode(false);
    }
  }, [selectedProject, isLaunchingOpenCode]);

  const handleViewHistoryFile = useCallback(async (entry: PromptHistoryEntry) => {
    if (!selectedProject) return;
    try {
      const content = await readPromptHistoryFile(selectedProject.path, entry.filename);
      setSelectedHistoryContent(content);
      setSelectedHistoryFilename(entry.filename);
    } catch (err) {
      setHistoryStatus(`Error: ${String(err)}`);
    }
  }, [selectedProject]);

  const handleCopyHistoryFile = useCallback(async (entry: PromptHistoryEntry) => {
    if (!selectedProject) return;
    try {
      const content = await readPromptHistoryFile(selectedProject.path, entry.filename);
      await navigator.clipboard.writeText(content);
      setHistoryStatus("History prompt copied");
      setTimeout(() => setHistoryStatus(null), 2000);
    } catch (err) {
      setHistoryStatus(`Error: ${String(err)}`);
    }
  }, [selectedProject]);

  const handleCloseHistoryPreview = useCallback(() => {
    setSelectedHistoryContent(null);
    setSelectedHistoryFilename(null);
  }, []);

  if (!settings || isConfiguring) {
    return (
      <div className="min-h-screen bg-zinc-950 text-white flex flex-col items-center justify-center p-8">
        <div className="w-full max-w-md">
          <div className="mb-8 text-center">
            <h1 className="text-2xl font-semibold text-zinc-100 mb-2">ContextDock</h1>
            <p className="text-sm text-zinc-500">Keep persistent project context across AI coding sessions</p>
          </div>

          <div className="p-6 border border-zinc-800/80 rounded-xl bg-zinc-900/40">
            <p className="text-sm text-zinc-400 mb-4">Select your projects folder to get started</p>
            <input
              type="text"
              placeholder="C:\Users\YourName\Documents\Codex"
              className="w-full px-4 py-3 bg-zinc-950/80 border border-zinc-700/80 rounded-lg text-zinc-200 text-sm placeholder:text-zinc-600 focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20"
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  const value = (e.target as HTMLInputElement).value;
                  if (value.trim()) {
                    useSettingsStore.getState().updateRootPath(value.trim());
                    setIsConfiguring(false);
                  }
                }
              }}
            />
            <button
              onClick={() => {
                const input = document.querySelector('input') as HTMLInputElement;
                if (input?.value.trim()) {
                  useSettingsStore.getState().updateRootPath(input.value.trim());
                  setIsConfiguring(false);
                }
              }}
              className="w-full mt-4 px-4 py-3 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium rounded-lg transition-colors active:bg-blue-700"
            >
              Continue
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-zinc-950 text-white flex flex-col h-screen">
      <DashboardHeader
        rootPath={settings.root_projects_path}
        onRescan={handleRescan}
        isScanning={isScanning}
        onOpenSettings={() => setShowSettings(true)}
      />

      {!tauriAvailable && <TauriUnavailableBanner />}
      {error && <ErrorBanner message={error} onDismiss={clearError} />}

      <main className="flex-1 p-6 overflow-y-auto">
        <div className="mb-6">
          <h2 className="text-sm font-medium text-zinc-400 uppercase tracking-wider">Projects</h2>
          <p className="text-xs text-zinc-600 mt-1">
            {projects.length} {projects.length === 1 ? "project" : "projects"} found
          </p>
        </div>

        {projects.length === 0 ? (
          <EmptyState
            message="No projects found"
            submessage="Add folders to your projects directory to see them here"
          />
        ) : (
          <div className="grid gap-3">
            {projects.map((project) => (
              <ProjectCard
                key={project.id}
                project={project}
                onInitialize={handleInitializeClick}
                onSelect={handleProjectSelect}
              />
            ))}
          </div>
        )}
      </main>

      {initPreview && pendingInitProject && (
        <InitPreviewModal
          preview={initPreview}
          projectName={pendingInitProject.name}
          onConfirm={handleInitConfirm}
          onCancel={handleInitCancel}
          isInitializing={isInitializing}
        />
      )}

      {showProjectDetail && selectedProject && (
        <ProjectDetailPanel
          project={selectedProject}
          contextFiles={contextFiles}
          gitInfo={gitInfo}
          gitError={gitError}
          onClose={handleCloseProjectDetail}
          onSave={handleDetailSave}
          savingFiles={savingFiles}
          fileStates={detailFileStates}
          onFileChange={handleDetailFileChange}
          tauriAvailable={tauriAvailable}
          isLoadingGit={isLoadingGit}
          launchPromptContent={launchPromptContent}
          promptStatus={promptStatus}
          onGenerateLaunchPrompt={handleGenerateLaunchPrompt}
          onCopyPrompt={handleCopyPrompt}
          onLaunchOpenCode={handleLaunchOpenCode}
          isGeneratingPrompt={isGeneratingPrompt}
          requestedTask={requestedTask}
          onRequestedTaskChange={setRequestedTask}
          promptHistory={promptHistory}
          isLoadingPromptHistory={isLoadingPromptHistory}
          selectedHistoryContent={selectedHistoryContent}
          selectedHistoryFilename={selectedHistoryFilename}
          onViewHistoryFile={handleViewHistoryFile}
          onCopyHistoryFile={handleCopyHistoryFile}
          onCloseHistoryPreview={handleCloseHistoryPreview}
          historyStatus={historyStatus}
        />
      )}
      <SettingsDrawer isOpen={showSettings} onClose={() => setShowSettings(false)} />
    </div>
  );
}