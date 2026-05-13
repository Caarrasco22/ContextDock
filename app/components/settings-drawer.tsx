"use client";

import { useEffect } from "react";
import { useProjectStore } from "../stores/projectStore";
import { useSettingsStore } from "../stores/settingsStore";

interface SettingsDrawerProps {
  isOpen: boolean;
  onClose: () => void;
}

export function SettingsDrawer({ isOpen, onClose }: SettingsDrawerProps) {
  const { settings, loadSettings, updateRootPath, updateOpenCodeCommand, tauriAvailable, error, clearError } =
    useSettingsStore();
  const { scanProjects } = useProjectStore();

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  if (!isOpen) return null;

  return (
    <>
      <div
        className="fixed inset-0 bg-black/60 z-40"
        onClick={onClose}
      />
      <div className="fixed right-0 top-0 h-full w-full max-w-md bg-zinc-900 border-l border-zinc-800/60 z-50 flex flex-col">
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800/60">
          <h2 className="text-base font-semibold text-zinc-100">Settings</h2>
          <button
            onClick={onClose}
            className="px-3 py-1.5 text-xs text-zinc-400 hover:text-zinc-200 border border-zinc-700/60 rounded-lg hover:bg-zinc-800/50 transition-colors"
          >
            Close
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-6">
          {!tauriAvailable && (
            <div className="p-3 bg-amber-900/20 border border-amber-700/50 rounded-lg mb-6">
              <p className="text-sm text-amber-400">
                <span className="font-medium">Frontend-only mode:</span> Changes will not persist. Run{" "}
                <code className="text-amber-300">npm run tauri:dev</code> for full functionality.
              </p>
            </div>
          )}
          {error && (
            <div className="p-3 bg-red-900/20 border border-red-700/50 rounded-lg mb-6 flex items-center justify-between">
              <p className="text-sm text-red-400">{error}</p>
              <button
                onClick={clearError}
                className="ml-4 px-2 py-1 text-xs text-red-400 hover:text-red-300 border border-red-700/50 rounded transition-colors"
              >
                Dismiss
              </button>
            </div>
          )}

          <div className="space-y-6">
            <div className="p-5 border border-zinc-800/80 rounded-xl bg-zinc-900/40">
              <h3 className="text-sm font-medium text-zinc-200 mb-1">Projects Folder</h3>
              <p className="text-xs text-zinc-500 mb-4">The root folder where your projects are located</p>
              <div className="flex gap-3">
                <input
                  type="text"
                  value={settings?.root_projects_path || ""}
                  onChange={(e) => {
                    if (tauriAvailable && settings) {
                      updateRootPath(e.target.value);
                    }
                  }}
                  disabled={!tauriAvailable || !settings}
                  className="flex-1 px-4 py-2.5 bg-zinc-950/80 border border-zinc-700/80 rounded-lg text-zinc-200 text-sm font-mono focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
                />
                <button
                  onClick={() => settings && scanProjects(settings.root_projects_path)}
                  disabled={!tauriAvailable || !settings}
                  className="px-4 py-2.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-sm rounded-lg transition-colors border border-zinc-700/80 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Rescan
                </button>
              </div>
            </div>

            <div className="p-5 border border-zinc-800/80 rounded-xl bg-zinc-900/40">
              <h3 className="text-sm font-medium text-zinc-200 mb-1">OpenCode Command</h3>
              <p className="text-xs text-zinc-500 mb-4">The command used to launch OpenCode</p>
              <input
                type="text"
                value={settings?.opencode_command || ""}
                onChange={(e) => {
                  if (tauriAvailable && settings) {
                    updateOpenCodeCommand(e.target.value);
                  }
                }}
                disabled={!tauriAvailable || !settings}
                className="w-full px-4 py-2.5 bg-zinc-950/80 border border-zinc-700/80 rounded-lg text-zinc-200 text-sm font-mono focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
              />
              <p className="text-xs text-zinc-600 mt-2">
                Windows: <code className="text-zinc-400">opencode.cmd</code> &nbsp;·&nbsp; macOS/Linux:{" "}
                <code className="text-zinc-400">opencode</code>
              </p>
            </div>

            <div className="p-5 border border-zinc-800/80 rounded-xl bg-zinc-900/40">
              <h3 className="text-sm font-medium text-zinc-200 mb-1">About</h3>
              <div className="space-y-1.5 text-xs text-zinc-500">
                <p>
                  <span className="text-zinc-400">ContextDock</span> v0.1.0
                </p>
                <p>A lightweight workspace for persistent project context</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}