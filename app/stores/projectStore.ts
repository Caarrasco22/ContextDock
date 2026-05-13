import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { ProjectSummary, ContextFiles, ProjectMeta, InitPreview } from "../lib/types";
import { useSettingsStore } from "./settingsStore";

interface ProjectStore {
  projects: ProjectSummary[];
  selectedProject: ProjectSummary | null;
  contextFiles: ContextFiles | null;
  initPreview: InitPreview | null;
  isLoading: boolean;
  isInitializing: boolean;
  error: string | null;
  tauriAvailable: boolean;
  scanProjects: (path: string) => Promise<void>;
  selectProject: (project: ProjectSummary | null) => void;
  loadContextFiles: (projectPath: string) => Promise<void>;
  getInitPreview: (projectPath: string) => Promise<InitPreview | null>;
  initContext: (projectPath: string) => Promise<ProjectMeta | null>;
  updateContextFile: (projectPath: string, filename: string, content: string) => Promise<void>;
  refreshProjects: () => Promise<void>;
  clearError: () => void;
  clearInitPreview: () => void;
}

export const useProjectStore = create<ProjectStore>((set, get) => ({
  projects: [],
  selectedProject: null,
  contextFiles: null,
  initPreview: null,
  isLoading: false,
  isInitializing: false,
  error: null,
  tauriAvailable: true,

  scanProjects: async (path: string) => {
    set({ isLoading: true, error: null });

    const settingsStore = useSettingsStore.getState();
    if (!settingsStore.tauriAvailable) {
      set({
        error: "This action requires the Tauri desktop app. Run `npm run tauri:dev`.",
        isLoading: false,
        tauriAvailable: false,
      });
      return;
    }

    try {
      const projects = await invoke<ProjectSummary[]>("scan_projects", { rootPath: path });
      set({ projects, isLoading: false, tauriAvailable: true, error: null });
    } catch (err) {
      const message = String(err);
      if (message.includes("Unknown command") || message.includes("not found") || message.includes("404")) {
        set({
          error: "This action requires the Tauri desktop app. Run `npm run tauri:dev`.",
          isLoading: false,
          tauriAvailable: false,
        });
      } else if (message.includes("Path does not exist")) {
        set({ error: "Folder does not exist. Please check the path.", isLoading: false });
      } else {
        set({ error: `Failed to scan projects: ${message}`, isLoading: false });
      }
    }
  },

  selectProject: (project) => {
    set({ selectedProject: project, contextFiles: null, initPreview: null, error: null });
    if (project) {
      get().loadContextFiles(project.path);
    }
  },

  loadContextFiles: async (projectPath: string) => {
    const settingsStore = useSettingsStore.getState();
    if (!settingsStore.tauriAvailable) {
      return;
    }

    try {
      const files = await invoke<ContextFiles>("get_context_files", { projectPath });
      set({ contextFiles: files, error: null });
    } catch (err) {
      set({ error: `Failed to load context: ${String(err)}` });
    }
  },

  getInitPreview: async (projectPath: string) => {
    const settingsStore = useSettingsStore.getState();
    if (!settingsStore.tauriAvailable) {
      set({ error: "This action requires the Tauri desktop app. Run `npm run tauri:dev`." });
      return null;
    }

    try {
      const preview = await invoke<InitPreview>("get_init_preview", { projectPath });
      set({ initPreview: preview, error: null });
      return preview;
    } catch (err) {
      const message = String(err);
      if (message.includes("already has")) {
        set({ error: "Project already has .context-bridge/ folder." });
      } else {
        set({ error: `Failed to get preview: ${message}` });
      }
      return null;
    }
  },

  initContext: async (projectPath: string) => {
    const settingsStore = useSettingsStore.getState();
    if (!settingsStore.tauriAvailable) {
      set({ error: "This action requires the Tauri desktop app. Run `npm run tauri:dev`." });
      return null;
    }

    set({ isInitializing: true, error: null });

    try {
      const meta = await invoke<ProjectMeta>("init_context", { projectPath });
      await get().refreshProjects();
      set({ isInitializing: false, initPreview: null, error: null });
      return meta;
    } catch (err) {
      const message = String(err);
      if (message.includes("already exists") || message.includes("already has")) {
        set({ error: "Project already has .context-bridge/ folder.", isInitializing: false });
      } else if (message.includes("Permission denied") || message.includes("Access denied")) {
        set({ error: "Permission error: Cannot create files in this location.", isInitializing: false });
      } else {
        set({ error: `Failed to initialize context: ${message}`, isInitializing: false });
      }
      return null;
    }
  },

  updateContextFile: async (projectPath: string, filename: string, content: string) => {
    const settingsStore = useSettingsStore.getState();
    if (!settingsStore.tauriAvailable) {
      set({ error: "This action requires the Tauri desktop app. Run `npm run tauri:dev`." });
      return;
    }

    try {
      await invoke("write_context_file", { projectPath, filename, content });
      set({ error: null });
    } catch (err) {
      set({ error: `Failed to save file: ${String(err)}` });
    }
  },

  refreshProjects: async () => {
    const settingsState = useSettingsStore.getState();
    if (settingsState.settings?.root_projects_path) {
      await get().scanProjects(settingsState.settings.root_projects_path);
    }
  },

  clearError: () => set({ error: null }),

  clearInitPreview: () => set({ initPreview: null }),
}));