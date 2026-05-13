# Development Checklist

Use this checklist to verify the app is working correctly.

## Phase 1: Environment Setup

- [ ] Run `npm install` in project root
- [ ] Install Rust: `winget install -e --id Rustlang.Rustup` (Windows) or from rustup.rs
- [ ] Restart terminal and verify `cargo --version`
- [ ] Run `cargo check` in `src-tauri/` to verify Rust backend compiles

## Phase 2: Frontend Verification

- [ ] Run `npm run dev` — should open at http://localhost:3000
- [ ] Frontend loads without errors (shows configuration prompt if no settings)
- [ ] If running frontend-only (no Rust), amber banner appears
- [ ] App does not crash when Tauri commands fail

## Phase 3: Tauri Backend Verification

- [ ] Run `npm run tauri:dev` — desktop window should open
- [ ] Dashboard displays correctly
- [ ] No Rust panics or errors in terminal

## Phase 4: Settings Verification

- [ ] On first launch, app prompts for root projects folder
- [ ] Enter a path like `C:\Users\YourName\Documents\Codex`
- [ ] Click "Continue" — settings are saved
- [ ] Go to Settings page — root folder path is persisted
- [ ] Change the root folder path — click "Rescan" — list updates

## Phase 5: Project Scanning

- [ ] Create a test folder with 2-3 subdirectories (mock projects)
- [ ] Point root folder to parent of those subdirectories
- [ ] Dashboard shows the subdirectories as projects
- [ ] Project type is detected and displayed (Next.js, Node, Python, Rust, Unknown)
- [ ] Projects without `.context-bridge/` show "No context" badge
- [ ] Projects with `.context-bridge/` show "Ready" badge

## Phase 6: Context Initialization

- [ ] Click the "+ Add context" button on a project without context
- [ ] Preview modal appears showing:
  - Detected project type
  - Detected files/markers
  - List of files that will be created
  - Note about .gitignore update (if applicable)
- [ ] Click "Create" — `.context-bridge/` is created
- [ ] Verify folder contains:
  - [ ] `meta.json` (with project_type, favorite, timestamps)
  - [ ] `architecture.md` (auto-generated with stack detection)
  - [ ] `current.md` (empty template)
  - [ ] `recent-work.md` (empty template)
  - [ ] `sessions.json` (empty array)
  - [ ] `history/` (empty directory)
- [ ] If project has `.git` folder, verify `.gitignore` was updated
- [ ] Dashboard refreshes — project now shows "Ready" badge

## Phase 7: Error States (Manual Tests)

- [ ] Select a folder that does not exist — shows error, does not crash
- [ ] Select a folder with no projects — shows empty state with icon
- [ ] Running frontend-only (`npm run dev`) — amber banner shown, inputs disabled
- [ ] Project already has `.context-bridge/` — init shows error message
- [ ] Permission error when writing — shows error message, does not crash

## Phase 8: Settings Persistence

- [ ] Close and reopen Tauri app
- [ ] Previous root folder path is remembered
- [ ] OpenCode command preference is remembered

## Verification Commands

```bash
# Check Rust
cargo --version
cargo check --manifest-path src-tauri/Cargo.toml

# Build frontend
npm run build

# Run frontend only (some features limited)
npm run dev

# Run full Tauri app
npm run tauri:dev
```

## Expected File Structure After Init

```
project-name/
├── .context-bridge/
│   ├── meta.json          # Project metadata with type
│   ├── current.md         # Current focus
│   ├── architecture.md    # Auto-generated structure
│   ├── recent-work.md     # Recent session summaries
│   ├── sessions.json      # Session history
│   ├── launch-prompt.md   # Generated launch context
│   └── history/           # Session prompt copies
├── src/
├── app/
└── ... (project files)
```

## Project Type Detection

The app detects these types (no deep scanning):

| Type | Markers |
|------|---------|
| Next.js | `package.json` with `"next"` |
| Node | `package.json` without Next.js |
| Rust | `Cargo.toml` |
| Python | `pyproject.toml` or `requirements.txt` |
| Unknown | None of the above |