#[cfg(test)]
mod git_tests {
    use crate::commands::{get_git_info, is_git_repo};

    #[test]
    fn test_git_info_non_repo() {
        let result = get_git_info("C:\\Users\\Caarrasco22\\AppData\\Local\\Temp\\test-non-git".to_string());
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(!info.is_repo);
        assert!(info.branch.is_none());
        assert!(info.staged_files.is_empty());
        assert!(info.unstaged_files.is_empty());
        assert!(info.untracked_files.is_empty());
    }

    #[test]
    fn test_git_info_this_repo() {
        let result = get_git_info("C:\\Users\\Caarrasco22\\ai-context-bridge".to_string());
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.is_repo);
        assert_eq!(info.branch.as_deref(), Some("master"));
        assert!(!info.recent_commits.is_empty());
    }

    #[test]
    fn test_git_info_portfolio_repo() {
        let result = get_git_info("C:\\Users\\Caarrasco22\\portfolio-ivan".to_string());
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.is_repo);
        assert_eq!(info.branch.as_deref(), Some("main"));
    }

    #[test]
    fn test_git_info_nonexistent_path() {
        let result = get_git_info("C:\\Users\\Nobody\\nonexistent".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_is_git_repo_non_repo() {
        let result = is_git_repo("C:\\Users\\Caarrasco22\\AppData\\Local\\Temp\\test-non-git".to_string());
        assert!(!result);
    }

    #[test]
    fn test_is_git_repo_this_repo() {
        let result = is_git_repo("C:\\Users\\Caarrasco22\\ai-context-bridge".to_string());
        assert!(result);
    }
}