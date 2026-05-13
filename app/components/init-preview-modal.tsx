"use client";

import { InitPreview } from "../lib/types";

interface InitPreviewModalProps {
  preview: InitPreview;
  projectName: string;
  onConfirm: () => void;
  onCancel: () => void;
  isInitializing: boolean;
}

const projectTypeLabels: Record<string, string> = {
  nextjs: "Next.js",
  node: "Node",
  python: "Python",
  rust: "Rust",
  unknown: "Unknown",
};

export function InitPreviewModal({
  preview,
  projectName,
  onConfirm,
  onCancel,
  isInitializing,
}: InitPreviewModalProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
      <div className="w-full max-w-md border border-zinc-700/80 rounded-xl bg-zinc-900/95 shadow-2xl">
        <div className="p-6 border-b border-zinc-800">
          <h2 className="text-lg font-semibold text-zinc-100">Initialize Context</h2>
          <p className="text-sm text-zinc-500 mt-1">
            Create <span className="text-zinc-300">.context-bridge/</span> in <span className="text-zinc-300">{projectName}</span>
          </p>
        </div>

        <div className="p-6 space-y-5">
          <div>
            <h3 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-2">Detected</h3>
            <div className="flex items-center gap-2">
              <span className="px-2 py-1 text-xs bg-zinc-800 text-zinc-300 rounded-md border border-zinc-700">
                {projectTypeLabels[preview.project_type] || "Unknown"}
              </span>
              {preview.detected_files.length > 0 && (
                <span className="text-xs text-zinc-500">
                  {preview.detected_files.slice(0, 3).join(", ")}
                  {preview.detected_files.length > 3 && ` +${preview.detected_files.length - 3}`}
                </span>
              )}
            </div>
          </div>

          <div>
            <h3 className="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-2">Will create</h3>
            <div className="p-3 bg-zinc-950/50 rounded-lg border border-zinc-800/80 font-mono text-xs">
              {preview.will_create.map((item) => (
                <div key={item} className="text-zinc-400">
                  {item}
                </div>
              ))}
            </div>
          </div>

          {preview.gitignore_needs_update && (
            <div className="p-3 bg-amber-900/10 rounded-lg border border-amber-800/30">
              <p className="text-xs text-amber-400">
                <span className="font-medium">Note:</span> .gitignore will be updated to exclude .context-bridge/ from version control.
              </p>
            </div>
          )}
        </div>

        <div className="p-4 border-t border-zinc-800 flex gap-3 justify-end">
          <button
            onClick={onCancel}
            disabled={isInitializing}
            className="px-4 py-2 text-sm text-zinc-400 hover:text-zinc-200 transition-colors disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            disabled={isInitializing}
            className="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-colors disabled:opacity-50"
          >
            {isInitializing ? "Creating..." : "Create"}
          </button>
        </div>
      </div>
    </div>
  );
}