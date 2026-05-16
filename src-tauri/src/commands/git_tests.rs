#[cfg(test)]
mod git_tests {
    use crate::commands::{get_git_info, is_git_repo};
    use std::fs;
    use std::process::Command;

    fn test_temp_root() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("contextdock-test-{}", std::process::id()))
    }

    fn assert_git_available() {
        let ok = Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !ok {
            panic!("git is not available — cannot run this test");
        }
    }

    fn setup_git_repo() -> (std::path::PathBuf, String) {
        assert_git_available();
        let dir = test_temp_root().join("test-repo");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Command::new("git")
            .args(["init"])
            .current_dir(&dir)
            .output()
            .unwrap();
        fs::write(dir.join("test.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(&dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let path_str = dir.to_string_lossy().to_string();
        (dir, path_str)
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
        let (dir, path_str) = setup_git_repo();
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
        let (dir, path_str) = setup_git_repo();
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