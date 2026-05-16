export type ProjectType = "nextjs" | "node" | "python" | "rust" | "unknown";

export interface ProjectSummary {
  id: string;
  name: string;
  path: string;
  has_context: boolean;
  project_type: ProjectType;
  last_opened_at: string | null;
}

export interface ProjectMeta {
  id: string;
  name: string;
  path: string;
  project_type: ProjectType;
  created_at: string;
  last_opened_at: string | null;
  last_context_update_at: string | null;
  last_session_id: string | null;
  favorite: boolean;
}

export interface ContextFiles {
  meta: ProjectMeta | null;
  current: string | null;
  architecture: string | null;
  recent_work: string | null;
}

export interface InitPreview {
  will_create: string[];
  project_type: ProjectType;
  detected_files: string[];
  gitignore_needs_update: boolean;
}

export interface AppSettings {
  root_projects_path: string;
  opencode_command: string;
  theme: string;
}

export interface Session {
  id: string;
  started_at: string;
  ended_at: string;
  title: string;
  summary: string;
  files_changed: string[];
  status: string;
  next_steps: string[];
  prompt_used_path: string | null;
  notes: string | null;
}

export interface SessionsJson {
  sessions: Session[];
}

export interface CommitInfo {
  hash: string;
  message: string;
  date: string;
  author: string;
}

export interface GitInfo {
  is_repo: boolean;
  branch: string | null;
  is_clean: boolean;
  changed_files_count: number;
  staged_files: string[];
  unstaged_files: string[];
  untracked_files: string[];
  last_commit_hash: string | null;
  last_commit_message: string | null;
  last_commit_date: string | null;
  recent_commits: CommitInfo[];
}

export interface PromptHistoryEntry {
  filename: string;
  path: string;
  size_bytes: number;
  modified: string;
}