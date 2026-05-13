import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "../lib/types";

interface SettingsStore {
  settings: AppSettings | null;
  isLoading: boolean;
  error: string | null;
  tauriAvailable: boolean;
  loadSettings: () => Promise<void>;
  updateRootPath: (path: string) => Promise<void>;
  updateOpenCodeCommand: (cmd: string) => Promise<void>;
  checkTauriAvailability: () => Promise<boolean>;
  clearError: () => void;
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: null,
  isLoading: false,
  error: null,
  tauriAvailable: true,

  checkTauriAvailability: async () => {
    try {
      await invoke("get_settings");
      set({ tauriAvailable: true });
      return true;
    } catch {
      set({ tauriAvailable: false });
      return false;
    }
  },

  loadSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      const settings = await invoke<AppSettings>("get_settings");
      set({ settings, isLoading: false, tauriAvailable: true });
    } catch (err) {
      const message = String(err);
      if (message.includes("Unknown command") || message.includes("not found") || message.includes("404")) {
        set({
          error: "Tauri backend not available. Run `npm run tauri:dev` to use full functionality.",
          isLoading: false,
          tauriAvailable: false,
        });
      } else {
        set({ error: `Failed to load settings: ${message}`, isLoading: false });
      }
    }
  },

  updateRootPath: async (path: string) => {
    const current = get().settings;
    if (!current) return;

    if (!get().tauriAvailable) {
      set({ error: "This action requires the Tauri desktop app. Run `npm run tauri:dev`." });
      return;
    }

    const updated = { ...current, root_projects_path: path };
    try {
      await invoke("save_settings", { settings: updated });
      set({ settings: updated, error: null });
    } catch (err) {
      set({ error: `Failed to save settings: ${String(err)}` });
    }
  },

  updateOpenCodeCommand: async (cmd: string) => {
    const current = get().settings;
    if (!current) return;

    if (!get().tauriAvailable) {
      set({ error: "This action requires the Tauri desktop app. Run `npm run tauri:dev`." });
      return;
    }

    const updated = { ...current, opencode_command: cmd };
    try {
      await invoke("save_settings", { settings: updated });
      set({ settings: updated, error: null });
    } catch (err) {
      set({ error: `Failed to save settings: ${String(err)}` });
    }
  },

  clearError: () => set({ error: null }),
}));