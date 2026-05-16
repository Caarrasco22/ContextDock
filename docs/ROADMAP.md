# Roadmap

## v0.1 — Workflow Base ✅ (current)

- [x] Dashboard with project list
- [x] Project type detection (Next.js, Node, Python, Rust, Unknown)
- [x] `.context-bridge/` initialization with preview modal
- [x] Auto `.gitignore` integration
- [x] Context file editors (current.md, architecture.md, recent-work.md)
- [x] Launch prompt generation with full context
- [x] Git info panel (branch, staged/modified/untracked, commits)
- [x] Settings persistence (root path + OpenCode command)
- [x] Error handling and safe fallbacks
- [x] Frontend-only mode
- [x] OpenCode launch (Windows only)

## v0.2 — Session Awareness

- [x] Editable `## Requested Task` in UI (user can customize the task before launching)
- [x] `launch_opencode` cross-platform (macOS support)
- [ ] Session capture — start/end sessions, populate `sessions.json`
- [x] Prompt history — save generated prompts to `history/` with timestamps
- [ ] Project "health" indicator (untracked files count, stale context warning)
- [ ] Reset/re-init `.context-bridge/` from UI

## v0.3 — Context Enrichment

- [ ] Better architecture auto-detection (read package.json dependencies, folder patterns)
- [ ] Custom prompt templates (user-defined sections)
- [ ] Export/import context as a ZIP
- [ ] Dark/light theme toggle
- [ ] Favorite projects (pin to top)
- [ ] Recently opened projects list
- [ ] Cross-platform test suite

## Future Ideas (post v0.3)

- [ ] Multiple root folders
- [ ] Project groups / workspaces
- [ ] CLI companion tool
- [ ] VS Code extension integration
- [ ] Automatic `launch-prompt.md` refresh on project change detection
- [ ] i18n support
