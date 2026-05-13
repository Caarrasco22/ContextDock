"use client";

import { useEffect, useState, useCallback } from "react";
import { useParams, useRouter } from "next/navigation";
import { useProjectStore } from "../../stores/projectStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { ProjectMeta } from "../../lib/types";

interface FileState {
  content: string;
  originalContent: string;
  isDirty: boolean;
  lastSaved: string | null;
  error: string | null;
}

export default function ProjectPage() {
  const params = useParams();
  const router = useRouter();
  const projectId = params.id as string;

  const { settings, loadSettings, tauriAvailable } = useSettingsStore();
  const {
    projects,
    scanProjects,
    loadContextFiles,
    contextFiles,
    updateContextFile,
    selectProject,
    selectedProject,
  } = useProjectStore();

  const [fileStates, setFileStates] = useState<Record<string, FileState>>({});
  const [savingFiles, setSavingFiles] = useState<Record<string, boolean>>({});
  const [isLoading, setIsLoading] = useState(true);
  const [projectMeta, setProjectMeta] = useState<ProjectMeta | null>(null);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  useEffect(() => {
    if (settings?.root_projects_path && projects.length === 0) {
      scanProjects(settings.root_projects_path);
    }
  }, [settings, projects.length, scanProjects]);

  useEffect(() => {
    if (projects.length > 0 && projectId) {
      const project = projects.find((p) => p.id === projectId);
      if (project) {
        selectProject(project);
        loadContextFiles(project.path);
        setIsLoading(false);
      } else {
        router.push("/");
      }
    }
  }, [projects, projectId, selectProject, loadContextFiles, router]);

  useEffect(() => {
    if (contextFiles?.meta) {
      setProjectMeta(contextFiles.meta);
    }
  }, [contextFiles]);

  useEffect(() => {
    if (contextFiles) {
      setFileStates({
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
  }, [contextFiles]);

  const handleFileChange = useCallback((key: string, value: string) => {
    setFileStates((prev) => ({
      ...prev,
      [key]: {
        ...prev[key],
        content: value,
        isDirty: value !== prev[key].originalContent,
        error: null,
      },
    }));
  }, []);

  const handleSave = useCallback(async (filename: string) => {
    const keyMap: Record<string, string> = {
      "current.md": "current",
      "architecture.md": "architecture",
      "recent-work.md": "recentWork",
    };
    const fileKey = keyMap[filename];
    if (!fileKey || !selectedProject) return;

    setSavingFiles((prev) => ({ ...prev, [fileKey]: true }));

    try {
      await updateContextFile(selectedProject.path, filename, fileStates[fileKey].content);
      const now = new Date().toLocaleTimeString();
      setFileStates((prev) => ({
        ...prev,
        [fileKey]: {
          ...prev[fileKey],
          originalContent: fileStates[fileKey].content,
          isDirty: false,
          lastSaved: now,
          error: null,
        },
      }));
    } catch (err) {
      setFileStates((prev) => ({
        ...prev,
        [fileKey]: {
          ...prev[fileKey],
          error: `Failed to save: ${String(err)}`,
        },
      }));
    } finally {
      setSavingFiles((prev) => ({ ...prev, [fileKey]: false }));
    }
  }, [fileStates, selectedProject, updateContextFile]);

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

  if (isLoading || !projectMeta) {
    return (
      <div className="min-h-screen bg-zinc-950 text-white flex items-center justify-center">
        <div className="flex items-center gap-3">
          <svg className="animate-spin w-5 h-5 text-zinc-500" viewBox="0 0 24 24" fill="none">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
          <span className="text-zinc-500">Loading project...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-zinc-950 text-white">
      <header className="flex items-center gap-4 px-6 py-4 border-b border-zinc-800/60">
        <button
          onClick={() => router.push("/")}
          className="px-3 py-1.5 text-xs text-zinc-400 hover:text-zinc-200 border border-zinc-700/60 rounded-lg hover:bg-zinc-800/50 transition-colors"
        >
          ← Back
        </button>
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-lg font-semibold text-zinc-100">{projectMeta.name}</h1>
            <span className={`px-2 py-0.5 text-xs rounded-md border ${typeColors[projectMeta.project_type] || typeColors.unknown}`}>
              {typeLabels[projectMeta.project_type] || "Project"}
            </span>
          </div>
          <p className="text-xs text-zinc-500 font-mono mt-0.5">{projectMeta.path}</p>
        </div>
      </header>

      <main className="p-6 space-y-6 max-w-3xl">
        <EditorSection
          title="Current Focus"
          filename="current.md"
          value={fileStates.current?.content || ""}
          onChange={(v) => handleFileChange("current", v)}
          onSave={() => handleSave("current.md")}
          isSaving={savingFiles.current}
          isDirty={fileStates.current?.isDirty || false}
          lastSaved={fileStates.current?.lastSaved}
          error={fileStates.current?.error}
          disabled={!tauriAvailable}
        />

        <EditorSection
          title="Architecture"
          filename="architecture.md"
          value={fileStates.architecture?.content || ""}
          onChange={(v) => handleFileChange("architecture", v)}
          onSave={() => handleSave("architecture.md")}
          isSaving={savingFiles.architecture}
          isDirty={fileStates.architecture?.isDirty || false}
          lastSaved={fileStates.architecture?.lastSaved}
          error={fileStates.architecture?.error}
          disabled={!tauriAvailable}
        />

        <EditorSection
          title="Recent Work"
          filename="recent-work.md"
          value={fileStates.recentWork?.content || ""}
          onChange={(v) => handleFileChange("recentWork", v)}
          onSave={() => handleSave("recent-work.md")}
          isSaving={savingFiles.recentWork}
          isDirty={fileStates.recentWork?.isDirty || false}
          lastSaved={fileStates.recentWork?.lastSaved}
          error={fileStates.recentWork?.error}
          disabled={!tauriAvailable}
        />
      </main>
    </div>
  );
}

function EditorSection({
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