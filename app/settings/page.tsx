"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useSettingsStore } from "../stores/settingsStore";
import { useProjectStore } from "../stores/projectStore";

function TauriUnavailableBanner() {
  return (
    <div className="p-3 bg-amber-900/20 border border-amber-700/50 rounded-lg mb-6">
      <p className="text-sm text-amber-400">
        <span className="font-medium">Frontend-only mode:</span> Changes will not persist. Run{" "}
        <code className="text-amber-300">npm run tauri:dev</code> for full functionality.
      </p>
    </div>
  );
}

function ErrorBanner({ message, onDismiss }: { message: string; onDismiss: () => void }) {
  return (
    <div className="p-3 bg-red-900/20 border border-red-700/50 rounded-lg mb-6 flex items-center justify-between">
      <p className="text-sm text-red-400">{message}</p>
      <button
        onClick={onDismiss}
        className="ml-4 px-2 py-1 text-xs text-red-400 hover:text-red-300 border border-red-700/50 rounded transition-colors"
      >
        Dismiss
      </button>
    </div>
  );
}

export default function Settings() {
  const { settings, loadSettings, updateRootPath, updateOpenCodeCommand, tauriAvailable, error, clearError } =
    useSettingsStore();
  const { scanProjects } = useProjectStore();
  const router = useRouter();

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  if (!settings) {
    return (
      <div className="min-h-screen bg-zinc-950 text-white flex items-center justify-center">
        <p className="text-zinc-500">Loading...</p>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-zinc-950 text-white">
      <header className="flex items-center gap-4 px-6 py-4 border-b border-zinc-800/60">
        <button
          onClick={() => router.push("/")}
          className="text-sm text-zinc-500 hover:text-zinc-200 transition-colors"
        >
          ← Back
        </button>
        <h1 className="text-base font-semibold text-zinc-100">Settings</h1>
      </header>

      <main className="p-6 max-w-xl">
        {!tauriAvailable && <TauriUnavailableBanner />}
        {error && <ErrorBanner message={error} onDismiss={clearError} />}

        <div className="space-y-6">
          <div className="p-5 border border-zinc-800/80 rounded-xl bg-zinc-900/40">
            <h2 className="text-sm font-medium text-zinc-200 mb-1">Projects Folder</h2>
            <p className="text-xs text-zinc-500 mb-4">The root folder where your projects are located</p>
            <div className="flex gap-3">
              <input
                type="text"
                value={settings.root_projects_path}
                onChange={(e) => {
                  if (tauriAvailable) {
                    updateRootPath(e.target.value);
                  }
                }}
                disabled={!tauriAvailable}
                className="flex-1 px-4 py-2.5 bg-zinc-950/80 border border-zinc-700/80 rounded-lg text-zinc-200 text-sm font-mono focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
              />
              <button
                onClick={() => scanProjects(settings.root_projects_path)}
                disabled={!tauriAvailable}
                className="px-4 py-2.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-sm rounded-lg transition-colors border border-zinc-700/80 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Rescan
              </button>
            </div>
          </div>

          <div className="p-5 border border-zinc-800/80 rounded-xl bg-zinc-900/40">
            <h2 className="text-sm font-medium text-zinc-200 mb-1">OpenCode Command</h2>
            <p className="text-xs text-zinc-500 mb-4">The command used to launch OpenCode</p>
            <input
              type="text"
              value={settings.opencode_command}
              onChange={(e) => {
                if (tauriAvailable) {
                  updateOpenCodeCommand(e.target.value);
                }
              }}
              disabled={!tauriAvailable}
              className="w-full px-4 py-2.5 bg-zinc-950/80 border border-zinc-700/80 rounded-lg text-zinc-200 text-sm font-mono focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
            />
            <p className="text-xs text-zinc-600 mt-2">
              Windows: <code className="text-zinc-400">opencode.cmd</code> &nbsp;·&nbsp; macOS/Linux:{" "}
              <code className="text-zinc-400">opencode</code>
            </p>
          </div>

          <div className="p-5 border border-zinc-800/80 rounded-xl bg-zinc-900/40">
            <h2 className="text-sm font-medium text-zinc-200 mb-1">About</h2>
            <div className="space-y-1.5 text-xs text-zinc-500">
              <p>
                <span className="text-zinc-400">ContextDock</span> v0.1.0
              </p>
              <p>A lightweight workspace for persistent project context</p>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}