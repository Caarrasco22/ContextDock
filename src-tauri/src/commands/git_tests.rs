#[cfg(test)]
mod git_tests {
    use crate::commands::{get_git_info, is_git_repo};
    use std::fs;
    use std::process::Command;

    fn test_temp_root() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("contextdock-test-{}", std::process::id()))
    }

    fn is_git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn setup_git_repo() -> Option<(std::path::PathBuf, String)> {
        if !is_git_available() {
            return None;
        }

        let dir = test_temp_root().join("test-repo");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).ok()?;

        Command::new("git")
            .args(["-c", "user.name=Test", "-c", "user.email=test@test.com", "init"])
            .current_dir(&dir)
            .output()
            .ok()
            .filter(|o| o.status.success())?;

        fs::write(dir.join("test.txt"), "hello").ok()?;

        Command::new("git")
            .args(["-c", "user.name=Test", "-c", "user.email=test@test.com", "add", "test.txt"])
            .current_dir(&dir)
            .output()
            .ok()
            .filter(|o| o.status.success())?;

        Command::new("git")
            .args(["-c", "user.name=Test", "-c", "user.email=test@test.com", "commit", "-m", "initial"])
            .current_dir(&dir)
            .output()
            .ok()
            .filter(|o| o.status.success())?;

        let path_str = dir.to_string_lossy().to_string();
        Some((dir, path_str))
    }

    #[test]
    fn test_git_info_non_repo() {
        let dir = test_temp_root().join("test-non-repo");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let result = get_git_info(dir.to_string_lossy().to_string());
        let _ = fs::remove_dir_all(&dir);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(!info.is_repo);
        assert!(info.branch.is_none());
    }

    #[test]
    fn test_git_info_repo() {
        let (dir, path_str) = match setup_git_repo() {
            Some(v) => v,
            None => return,
        };
        let result = get_git_info(path_str);
        let _ = fs::remove_dir_all(&dir);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.is_repo);
        assert!(!info.recent_commits.is_empty());
    }

    #[test]
    fn test_git_info_nonexistent_path() {
        let path = test_temp_root().join("definitely-does-not-exist-12345");
        let result = get_git_info(path.to_string_lossy().to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_is_git_repo_true() {
        let (dir, path_str) = match setup_git_repo() {
            Some(v) => v,
            None => return,
        };
        let result = is_git_repo(path_str);
        let _ = fs::remove_dir_all(&dir);
        assert!(result);
    }

    #[test]
    fn test_is_git_repo_false() {
        let dir = test_temp_root().join("test-non-git-dir");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let result = is_git_repo(dir.to_string_lossy().to_string());
        let _ = fs::remove_dir_all(&dir);
        assert!(!result);
    }
}