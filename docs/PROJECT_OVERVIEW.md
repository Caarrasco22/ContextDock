# Project Overview — ContextDock

Technical architecture and internals reference.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  Tauri Window                     │
│  ┌───────────────────────────────────────────┐  │
│  │          Next.js 16 (App Router)           │  │
│  │  ┌──────────┐  ┌──────────┐  ┌────────┐  │  │
│  │  │ Zustand  │  │  Pages   │  │ Comps  │  │  │
│  │  │  Store   │  │          │  │        │  │  │
│  │  └────┬─────┘  └──────────┘  └────────┘  │  │
│  │       │  invoke()                           │  │
│  └───────┼─────────────────────────────────────┘  │
│          │  IPC (tauri::command)                   │
│  ┌───────▼─────────────────────────────────────┐  │
│  │            Rust Backend                      │  │
│  │  ┌───────────┐ ┌──────────┐ ┌───────────┐  │  │
│  │  │ settings  │ │projects  │ │   git     │  │  │
│  │  │   .rs     │ │  .rs     │ │   .rs     │  │  │
│  │  └───────────┘ └──────────┘ └───────────┘  │  │
│  │  ┌──────────────────────────────────────┐   │  │
│  │  │           opencode.rs                │   │  │
│  │  │   prompt generation + launch         │   │  │
│  │  └──────────────────────────────────────┘   │  │
│  └─────────────────────────────────────────────┘  │
│                                                    │
│  ┌─────────────────────────────────────────────┐  │
│  │              Filesystem                      │  │
│  │  ~/Codex/project/.context-bridge/            │  │
│  │  %APPDATA%/contextdock-settings.json   │  │
│  └─────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

## Tauri Commands (IPC API)

All frontend-to-backend communication goes through `invoke()` calls to these Rust commands:

| Command | Module | Purpose |
|---------|--------|---------|
| `get_settings` | `settings.rs` | Load app settings from disk |
| `save_settings` | `settings.rs` | Persist app settings to disk |
| `scan_projects` | `projects.rs` | List subdirectories in root folder with type detection |
| `get_init_preview` | `projects.rs` | Preview what `init_context` will create |
| `init_context` | `projects.rs` | Create `.context-bridge/` folder and files |
| `get_context_files` | `projects.rs` | Read current.md, architecture.md, recent-work.md |
| `write_context_file` | `projects.rs` | Write to a context file (whitelist: 6 filenames) |
| `get_git_info` | `git.rs` | Return branch, status, staged/unstaged/untracked, commits |
| `is_git_repo` | `git.rs` | Check if `.git/` exists |
| `generate_opencode_launch_prompt` | `opencode.rs` | Build and write `launch-prompt.md` |
| `read_launch_prompt` | `opencode.rs` | Read existing `launch-prompt.md` |
| `launch_opencode` | `opencode.rs` | Spawn OpenCode process (cross-platform) |

## Context Files Schema

### `meta.json`
```json
{
  "id": "project-name",
  "name": "Project Name",
  "path": "/absolute/path",
  "project_type": "nextjs | node | python | rust | unknown",
  "created_at": "1700000000.000000000Z",
  "last_opened_at": null,
  "last_context_update_at": "1700000000.000000000Z",
  "last_session_id": null,
  "favorite": false
}
```

### `current.md`
User-editable. Default template: `# Current Focus\n\n`.

### `architecture.md`
Auto-generated on init, user-editable afterward. Contains:
- Detected tech stack
- Top-level folder structure (depth 1, max 50 entries)

### `recent-work.md`
User-editable. Default template: `# Recent Work\n\n`.

### `sessions.json`
```json
{
  "sessions": [
    {
      "id": "uuid",
      "started_at": "timestamp",
      "ended_at": "timestamp",
      "title": "Session title",
      "summary": "What was done",
      "files_changed": ["file1.tsx"],
      "status": "completed",
      "next_steps": ["task"],
      "prompt_used_path": null,
      "notes": null
    }
  ]
}
```

### `launch-prompt.md`
Generated on demand. Structure:
```markdown
# OpenCode Launch Prompt
## Project       (name, path, git status)
## Current Goal  (current.md)
## Architecture  (architecture.md)
## Recent Work   (recent-work.md)
## Git Status    (branch, changed files, recent commits)
## Instructions for OpenCode  (code of conduct)
## Requested Task (context-aware directive)
```

## Prompt Generation Flow

1. User clicks "Generate launch prompt" in `ProjectDetailPanel`
2. Frontend calls `invoke("generate_opencode_launch_prompt", { projectPath })`
3. Rust `opencode.rs::generate_opencode_launch_prompt`:
   - Reads `current.md`, `architecture.md`, `recent-work.md`
   - Calls `get_git_info` for branch/status/commits
   - Builds prompt with `build_launch_prompt()` using structured template
   - Writes result to `.context-bridge/launch-prompt.md`
   - Also saves a timestamped copy to `.context-bridge/history/<timestamp>-launch-prompt.md`
   - Returns content + path to frontend
4. Frontend displays prompt status, enables Copy + Launch buttons

## Settings Storage

Settings are stored as JSON at:
- **Windows**: `%APPDATA%/contextdock-settings.json`
- **macOS**: `~/Library/Application Support/contextdock-settings.json`
- **Linux**: `~/.config/contextdock-settings.json`

Fields:
```json
{
  "root_projects_path": "C:\\Users\\...\\Documents\\Codex",
  "opencode_command": "opencode.cmd",
  "theme": "dark"
}
```

## Known Technical Limitations

1. **No session capture** — `sessions.json` exists but is never populated. `history/` now stores timestamped launch prompt snapshots.
2. **Project type detection is shallow** — only checks for marker files at the root level, no deep scanning.
3. **Frontend-only mode is limited** — when Tauri is unavailable, most features are disabled (only the config prompt works).
4. **No static export** — `next.config.ts` doesn't use `output: "export"`, so Tauri serves from `../.next` (dev server proxy). For production builds, this may need adjustment.
