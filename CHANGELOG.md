# Changelog

All notable changes to ContextDock will be documented in this file.

## v0.2.0 - 2026-05-17

### Added
- Editable Requested Task before generating OpenCode prompts.
- Prompt history snapshots under `.context-bridge/history/`.
- Prompt History viewer in the UI.
- Cross-platform OpenCode launcher improvements.

### Fixed
- Project root detection when `root_projects_path` points directly to a project.
- OpenCode prompt passing: use `--prompt` instead of passing `launch-prompt.md` as a positional argument.
- Rust git tests now work without global git user config.

### Improved
- macOS launcher opens Terminal.app correctly.
- Linux launcher now uses `.context-bridge/launch-opencode.sh`.
- Windows launcher now uses `.context-bridge/launch-opencode.ps1`.
- More robust handling of paths with spaces and multi-line prompts.
- Backend test coverage expanded.

### Validation
- `cargo test`: 62/62 passed
- `cargo check`: passed
- `npm run lint`: 0 errors
- `npm run build`: compiled successfully
