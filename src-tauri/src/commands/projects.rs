use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectType {
    #[serde(rename = "nextjs")]
    Nextjs,
    #[serde(rename = "node")]
    Node,
    #[serde(rename = "python")]
    Python,
    #[serde(rename = "rust")]
    Rust,
    #[serde(rename = "unknown")]
    Unknown,
}

impl Default for ProjectType {
    fn default() -> Self {
        ProjectType::Unknown
    }
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Nextjs => write!(f, "Next.js"),
            ProjectType::Node => write!(f, "Node"),
            ProjectType::Python => write!(f, "Python"),
            ProjectType::Rust => write!(f, "Rust"),
            ProjectType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub path: String,
    pub has_context: bool,
    pub project_type: ProjectType,
    pub last_opened_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub id: String,
    pub name: String,
    pub path: String,
    pub project_type: ProjectType,
    pub created_at: String,
    pub last_opened_at: Option<String>,
    pub last_context_update_at: Option<String>,
    pub last_session_id: Option<String>,
    pub favorite: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextFiles {
    pub meta: Option<ProjectMeta>,
    pub current: Option<String>,
    pub architecture: Option<String>,
    pub recent_work: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitPreview {
    pub will_create: Vec<String>,
    pub project_type: ProjectType,
    pub detected_files: Vec<String>,
    pub gitignore_needs_update: bool,
}

fn is_ignored_dir(name: &str) -> bool {
    matches!(
        name,
        "node_modules"
            | ".next"
            | ".turbo"
            | "dist"
            | "build"
            | "coverage"
            | "__pycache__"
            | ".venv"
            | "venv"
            | "target"
            | ".cache"
            | ".context-bridge"
    )
}

fn is_hidden_or_system(name: &str) -> bool {
    name.starts_with('.') || name == "$RECYCLE.BIN" || name == "System Volume Information"
}

fn is_project_root(path: &Path) -> bool {
    let strong_markers = [
        "package.json",
        "Cargo.toml",
        "pyproject.toml",
        "requirements.txt",
        "next.config.js",
        "next.config.ts",
        "vite.config.js",
        "vite.config.ts",
    ];

    for marker in &strong_markers {
        if path.join(marker).exists() {
            return true;
        }
    }

    path.join(".git").is_dir()
}

fn is_internal_dir(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let internal_names = [
        "app",
        "components",
        "lib",
        "src",
        "public",
        "pages",
        "tests",
        "docs",
    ];

    internal_names.contains(&name.as_str())
}

fn detect_project_type(path: &Path) -> ProjectType {
    if path.join("package.json").exists() {
        if let Ok(content) = fs::read_to_string(path.join("package.json")) {
            if content.contains("\"next\"") {
                return ProjectType::Nextjs;
            }
            return ProjectType::Node;
        }
    }

    if path.join("Cargo.toml").exists() {
        return ProjectType::Rust;
    }

    if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
        return ProjectType::Python;
    }

    ProjectType::Unknown
}

fn detect_project_markers(path: &Path) -> Vec<String> {
    let mut markers = Vec::new();

    let marker_files = [
        "package.json",
        "Cargo.toml",
        "pyproject.toml",
        "requirements.txt",
        "README.md",
        ".gitignore",
        "tsconfig.json",
        "next.config.ts",
        "next.config.js",
        "vite.config.ts",
        "webpack.config.js",
    ];

    let marker_dirs = ["src", "app", "lib", "components", "pages", "tests", "docs"];

    for marker in &marker_files {
        if path.join(marker).exists() {
            markers.push(marker.to_string());
        }
    }

    for marker in &marker_dirs {
        if path.join(marker).is_dir() {
            markers.push(format!("{}/", marker));
        }
    }

    markers.truncate(10);
    markers
}

fn scan_dir_for_structure(path: &Path, depth: usize) -> Vec<String> {
    if depth > 1 {
        return vec![];
    }

    let mut entries: Vec<_> = fs::read_dir(path)
        .map(|d| {
            d.filter_map(|e| {
                let entry = match e {
                    Ok(e) => e,
                    Err(_) => return None,
                };
                let name = entry.file_name().to_string_lossy().to_string();
                if is_hidden_or_system(&name) && name != ".gitignore" {
                    return None;
                }
                let is_dir = entry.path().is_dir();
                Some((name, is_dir))
            })
            .collect()
        })
        .unwrap_or_default();

    entries.sort_by(|a, b| {
        match (a.1, b.1) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0),
        }
    });

    entries
        .into_iter()
        .take(50)
        .filter_map(|(name, is_dir)| {
            if is_dir && !is_ignored_dir(&name) {
                Some(format!("{}/", name))
            } else if !is_ignored_dir(&name) {
                Some(name)
            } else {
                None
            }
        })
        .collect()
}

fn detect_stack(path: &Path) -> Vec<String> {
    let mut stack = Vec::new();

    if path.join("package.json").exists() {
        if let Ok(content) = fs::read_to_string(path.join("package.json")) {
            if content.contains("\"next\"") {
                stack.push("Next.js".to_string());
            }
            if content.contains("\"react\"") {
                stack.push("React".to_string());
            }
            if content.contains("\"typescript\"") {
                stack.push("TypeScript".to_string());
            }
        }
    }

    if path.join("next.config.ts").exists() || path.join("next.config.js").exists() {
        if !stack.contains(&"Next.js".to_string()) {
            stack.push("Next.js".to_string());
        }
    }

    if path.join("tailwind.config.ts").exists() || path.join("tailwind.config.js").exists() {
        if !stack.contains(&"Tailwind CSS".to_string()) {
            stack.push("Tailwind CSS".to_string());
        }
    }

    if path.join("Cargo.toml").exists() {
        stack.push("Rust".to_string());
    }

    if path.join("pyproject.toml").exists() {
        stack.push("Python".to_string());
    }

    if path.join("requirements.txt").exists() {
        stack.push("Python".to_string());
    }

    stack
}

fn check_gitignore_for_context_bridge(path: &Path) -> bool {
    let gitignore_path = path.join(".gitignore");
    if !gitignore_path.exists() {
        return false;
    }

    if let Ok(content) = fs::read_to_string(&gitignore_path) {
        content.lines().any(|line| line.trim() == ".context-bridge/")
    } else {
        false
    }
}

fn append_to_gitignore(path: &Path) -> Result<(), String> {
    let gitignore_path = path.join(".gitignore");
    let entry = "\n# ContextDock\n.context-bridge/\n";

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path).map_err(|e| e.to_string())?;
        if !content.contains(".context-bridge/") {
            fs::write(&gitignore_path, content + entry).map_err(|e| e.to_string())?;
        }
    } else {
        fs::write(&gitignore_path, entry).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[command]
pub fn scan_projects(root_path: String) -> Result<Vec<ProjectSummary>, String> {
    let root = Path::new(&root_path);

    if !root.exists() {
        return Err(format!("Path does not exist: {}", root_path));
    }

    let mut projects = Vec::new();

    if is_project_root(root) {
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let has_context = root.join(".context-bridge").exists();
        let project_type = detect_project_type(root);

        let last_opened = if has_context {
            let meta_path = root.join(".context-bridge/meta.json");
            fs::read_to_string(&meta_path)
                .ok()
                .and_then(|content| serde_json::from_str::<ProjectMeta>(&content).ok())
                .and_then(|meta| meta.last_opened_at)
        } else {
            None
        };

        projects.push(ProjectSummary {
            id: name.clone(),
            name,
            path: root.to_string_lossy().to_string(),
            has_context,
            project_type,
            last_opened_at: last_opened,
        });
    }

    let entries = fs::read_dir(root).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if is_hidden_or_system(&name) {
            continue;
        }

        if is_ignored_dir(&name) {
            continue;
        }

        if is_internal_dir(&path) && !is_project_root(&path) {
            continue;
        }

        let has_context = path.join(".context-bridge").exists();
        let project_type = detect_project_type(&path);

        let last_opened = if has_context {
            let meta_path = path.join(".context-bridge/meta.json");
            fs::read_to_string(&meta_path)
                .ok()
                .and_then(|content| serde_json::from_str::<ProjectMeta>(&content).ok())
                .and_then(|meta| meta.last_opened_at)
        } else {
            None
        };

        projects.push(ProjectSummary {
            id: name.clone(),
            name,
            path: path.to_string_lossy().to_string(),
            has_context,
            project_type,
            last_opened_at: last_opened,
        });
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(projects)
}

#[command]
pub fn get_init_preview(project_path: String) -> Result<InitPreview, String> {
    let path = Path::new(&project_path);

    if !path.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    if path.join(".context-bridge").exists() {
        return Err("Project already has .context-bridge/ folder.".to_string());
    }

    let project_type = detect_project_type(path);
    let detected_files = detect_project_markers(path);
    let gitignore_needs_update = path.join(".git").exists() && !check_gitignore_for_context_bridge(path);

    let will_create = vec![
        ".context-bridge/".to_string(),
        ".context-bridge/meta.json".to_string(),
        ".context-bridge/current.md".to_string(),
        ".context-bridge/architecture.md".to_string(),
        ".context-bridge/recent-work.md".to_string(),
        ".context-bridge/sessions.json".to_string(),
        ".context-bridge/history/".to_string(),
    ];

    Ok(InitPreview {
        will_create,
        project_type,
        detected_files,
        gitignore_needs_update,
    })
}

#[command]
pub fn init_context(project_path: String) -> Result<ProjectMeta, String> {
    let path = Path::new(&project_path);

    if !path.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    if path.join(".context-bridge").exists() {
        return Err("Project already has .context-bridge/ folder.".to_string());
    }

    let context_dir = path.join(".context-bridge");
    fs::create_dir_all(&context_dir).map_err(|e| e.to_string())?;

    let history_dir = context_dir.join("history");
    fs::create_dir_all(&history_dir).map_err(|e| e.to_string())?;

    let name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let id = name.to_lowercase().replace(' ', "-");
    let project_type = detect_project_type(path);
    let now = chrono_now();

    let meta = ProjectMeta {
        id: id.clone(),
        name: name.clone(),
        path: project_path.clone(),
        project_type: project_type.clone(),
        created_at: now.clone(),
        last_opened_at: None,
        last_context_update_at: Some(now),
        last_session_id: None,
        favorite: false,
    };

    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(context_dir.join("meta.json"), meta_json).map_err(|e| e.to_string())?;

    let stack = detect_stack(path);
    let structure = scan_dir_for_structure(path, 0);

    let architecture_md = format!(
        "# Architecture\n\nGenerated from lightweight folder scan.\n\n## Detected stack\n\n{}\n\n## Folder structure\n\n```txt\n{}\n```\n",
        if stack.is_empty() {
            "- (not detected)".to_string()
        } else {
            stack.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
        },
        structure.join("\n")
    );
    fs::write(context_dir.join("architecture.md"), architecture_md).map_err(|e| e.to_string())?;

    fs::write(context_dir.join("current.md"), "# Current Focus\n\n").map_err(|e| e.to_string())?;
    fs::write(context_dir.join("recent-work.md"), "# Recent Work\n\n").map_err(|e| e.to_string())?;
    fs::write(context_dir.join("sessions.json"), "{\"sessions\":[]}").map_err(|e| e.to_string())?;

    if path.join(".git").exists() {
        let _ = append_to_gitignore(path);
    }

    Ok(meta)
}

#[command]
pub fn get_context_files(project_path: String) -> Result<ContextFiles, String> {
    let context_dir = Path::new(&project_path).join(".context-bridge");

    let meta = if context_dir.join("meta.json").exists() {
        fs::read_to_string(context_dir.join("meta.json"))
            .ok()
            .and_then(|c| serde_json::from_str::<ProjectMeta>(&c).ok())
    } else {
        None
    };

    let read_md = |name: &str| -> Option<String> {
        let p = context_dir.join(name);
        if p.exists() {
            fs::read_to_string(&p).ok()
        } else {
            None
        }
    };

    Ok(ContextFiles {
        meta,
        current: read_md("current.md"),
        architecture: read_md("architecture.md"),
        recent_work: read_md("recent-work.md"),
    })
}

#[command]
pub fn write_context_file(project_path: String, filename: String, content: String) -> Result<(), String> {
    let context_dir = Path::new(&project_path).join(".context-bridge");

    if !context_dir.exists() {
        return Err(".context-bridge directory not initialized".to_string());
    }

    let valid_files = ["meta.json", "current.md", "architecture.md", "recent-work.md", "sessions.json", "launch-prompt.md"];
    if !valid_files.contains(&filename.as_str()) {
        return Err(format!("Invalid filename: {}", filename));
    }

    fs::write(context_dir.join(&filename), content).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique_test_dir(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "contextdock-{}-{}",
            label,
            std::process::id()
        ))
    }

    fn create_project_at(path: &std::path::Path, name: &str) -> std::path::PathBuf {
        let proj = path.join(name);
        fs::create_dir_all(&proj).unwrap();
        fs::write(proj.join("package.json"), "{}").unwrap();
        proj
    }

    #[test]
    fn test_scan_root_contains_child_project() {
        let root = unique_test_dir("scan-child");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        create_project_at(&root, "my-app");

        let result = scan_projects(root.to_string_lossy().to_string());
        let _ = fs::remove_dir_all(&root);

        assert!(result.is_ok());
        let projects = result.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "my-app");
        assert!(matches!(projects[0].project_type, ProjectType::Node));
    }

    #[test]
    fn test_scan_root_is_itself_a_project() {
        let root = unique_test_dir("scan-self");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("package.json"), "{}").unwrap();

        let result = scan_projects(root.to_string_lossy().to_string());
        let _ = fs::remove_dir_all(&root);

        assert!(result.is_ok());
        let projects = result.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].path, root.to_string_lossy().to_string());
        assert!(matches!(projects[0].project_type, ProjectType::Node));
    }

    #[test]
    fn test_scan_root_project_with_app_subfolder_ignores_app() {
        let root = unique_test_dir("scan-app-ignored");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("package.json"), "{}").unwrap();
        fs::create_dir_all(root.join("app")).unwrap();
        fs::write(root.join("app/page.tsx"), "export default function Home() {}").unwrap();

        let result = scan_projects(root.to_string_lossy().to_string());
        let _ = fs::remove_dir_all(&root);

        assert!(result.is_ok());
        let projects = result.unwrap();
        assert_eq!(projects.len(), 1, "app/ subfolder should not be treated as a standalone project");
        assert_eq!(projects[0].path, root.to_string_lossy().to_string());
    }

    #[test]
    fn test_scan_root_project_with_app_that_has_own_packagejson() {
        let root = unique_test_dir("scan-app-pkg");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("package.json"), "{}").unwrap();
        let app_dir = root.join("app");
        fs::create_dir_all(&app_dir).unwrap();
        fs::write(app_dir.join("package.json"), "{}").unwrap();

        let result = scan_projects(root.to_string_lossy().to_string());
        let _ = fs::remove_dir_all(&root);

        assert!(result.is_ok());
        let projects = result.unwrap();
        assert_eq!(projects.len(), 2, "app/ with its own package.json should be a project");
    }

    #[test]
    fn test_scan_multiple_child_projects() {
        let root = unique_test_dir("scan-multi");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        create_project_at(&root, "project-a");
        create_project_at(&root, "project-b");
        create_project_at(&root, "project-c");

        let result = scan_projects(root.to_string_lossy().to_string());
        let _ = fs::remove_dir_all(&root);

        assert!(result.is_ok());
        let projects = result.unwrap();
        assert_eq!(projects.len(), 3);
        let names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"project-a"));
        assert!(names.contains(&"project-b"));
        assert!(names.contains(&"project-c"));
    }

    #[test]
    fn test_scan_nonexistent_path() {
        let path = unique_test_dir("nonexistent").join("definitely-does-not-exist-98765");
        let result = scan_projects(path.to_string_lossy().to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_is_project_root_detects_package_json() {
        let dir = unique_test_dir("pkg-json");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("package.json"), "{}").unwrap();
        assert!(is_project_root(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_project_root_detects_cargo_toml() {
        let dir = unique_test_dir("cargo-toml");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
        assert!(is_project_root(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_project_root_detects_git_dir() {
        let dir = unique_test_dir("git-dir");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".git")).unwrap();
        assert!(is_project_root(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_project_root_false_for_empty_dir() {
        let dir = unique_test_dir("empty");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        assert!(!is_project_root(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_project_root_false_for_readme_only() {
        let dir = unique_test_dir("readme");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("README.md"), "# Hello").unwrap();
        assert!(!is_project_root(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_internal_dir_true() {
        let dir = unique_test_dir("internal-true");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("app")).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("components")).unwrap();
        assert!(is_internal_dir(&dir.join("app")));
        assert!(is_internal_dir(&dir.join("src")));
        assert!(is_internal_dir(&dir.join("components")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_internal_dir_false_for_regular_names() {
        let dir = unique_test_dir("internal-false");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("my-project")).unwrap();
        fs::create_dir_all(dir.join("backend")).unwrap();
        assert!(!is_internal_dir(&dir.join("my-project")));
        assert!(!is_internal_dir(&dir.join("backend")));
        let _ = fs::remove_dir_all(&dir);
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();
    format!("{}.{:09}Z", secs, nanos)
}