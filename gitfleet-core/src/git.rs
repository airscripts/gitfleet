use crate::errors::ConfigError;

pub fn get_repo_root() -> Result<std::path::PathBuf, ConfigError> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| ConfigError::new(format!("Failed to run git: {e}")))?;

    if !output.status.success() {
        return Err(ConfigError::new(crate::constants::ERROR_NO_GIT_ROOT));
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(std::path::PathBuf::from(path))
}

pub fn get_remote_url(remote: Option<&str>) -> Result<String, ConfigError> {
    let remote = remote.unwrap_or("origin");

    let output = std::process::Command::new("git")
        .args(["remote", "get-url", remote])
        .output()
        .map_err(|e| ConfigError::new(format!("Failed to run git: {e}")))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    Err(ConfigError::new(crate::constants::ERROR_NO_REMOTE_URL))
}

pub fn get_remote_host(url: &str) -> Result<String, ConfigError> {
    let url = url.trim();

    if let Some(rest) = url.strip_prefix("git@")
        && let Some((host, _)) = rest.split_once(':')
    {
        return Ok(host.to_string());
    }

    let parsed = url::Url::parse(url)
        .map_err(|_| ConfigError::new(crate::constants::ERROR_NO_REMOTE_URL))?;

    let host = parsed
        .host_str()
        .ok_or_else(|| ConfigError::new(crate::constants::ERROR_NO_REMOTE_URL))?;

    Ok(match parsed.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_string(),
    })
}

pub fn get_remote_names() -> Result<Vec<String>, ConfigError> {
    let output = std::process::Command::new("git")
        .args(["remote"])
        .output()
        .map_err(|e| ConfigError::new(format!("Failed to run git: {e}")))?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let names = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(names)
}

pub fn get_current_branch() -> Result<String, ConfigError> {
    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .map_err(|e| ConfigError::new(format!("Failed to run git: {e}")))?;

    if !output.status.success() {
        return Err(ConfigError::new("Failed to determine current branch."));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if branch.is_empty() {
        return Err(ConfigError::new("No current branch detected."));
    }

    Ok(branch)
}

pub fn get_default_branch() -> Result<String, ConfigError> {
    let output = std::process::Command::new("git")
        .args(["remote", "show", "origin"])
        .output()
        .map_err(|e| ConfigError::new(format!("Failed to run git: {e}")))?;

    if !output.status.success() {
        return Err(ConfigError::new("Failed to determine default branch."));
    }

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if line.contains("HEAD branch:")
            && let Some(branch) = line.split(':').next_back()
        {
            let branch = branch.trim().to_string();

            if !branch.is_empty() {
                return Ok(branch);
            }
        }
    }

    Err(ConfigError::new("Failed to determine default branch."))
}

pub fn clone_repository(
    url: &str,
    depth: Option<u32>,
    directory: Option<&str>,
    remote_name: Option<&str>,
) -> Result<(), ConfigError> {
    let mut args = vec!["clone".to_string()];

    if let Some(d) = depth {
        args.push("--depth".to_string());
        args.push(d.to_string());
    }

    if let Some(rn) = remote_name {
        args.push("--origin".to_string());
        args.push(rn.to_string());
    }

    args.push(url.to_string());

    if let Some(dir) = directory {
        args.push(dir.to_string());
    }

    let status = std::process::Command::new("git")
        .args(&args)
        .status()
        .map_err(|e| ConfigError::new(format!("Failed to run git clone: {e}")))?;

    if !status.success() {
        return Err(ConfigError::new("git clone failed"));
    }

    Ok(())
}

pub fn checkout_branch(branch: &str) -> Result<(), ConfigError> {
    let status = std::process::Command::new("git")
        .args(["checkout", branch])
        .status()
        .map_err(|e| ConfigError::new(format!("Failed to checkout branch: {e}")))?;

    if !status.success() {
        return Err(ConfigError::new(format!(
            "Failed to checkout branch: {branch}"
        )));
    }

    Ok(())
}

pub fn is_inside_repo() -> bool {
    std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn is_working_tree_clean() -> bool {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output();

    match output {
        Ok(o) => {
            if !o.status.success() {
                return false;
            }

            String::from_utf8_lossy(&o.stdout).trim().is_empty()
        }

        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn test_get_remote_url_error_no_git() {
        let original_dir = std::env::current_dir().unwrap();

        let dir = tempfile::tempdir().unwrap();

        std::env::set_current_dir(dir.path()).unwrap();

        let result = get_remote_url(Some("nonexistent_remote_12345"));

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test_get_remote_names_returns_vec() {
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();

        let result = get_remote_names();

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_get_current_branch_returns_string() {
        let original_dir = std::env::current_dir().unwrap();
        let repo_dir = tempfile::tempdir().unwrap();

        let init = std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo_dir.path())
            .output()
            .unwrap();
        assert!(init.status.success());

        let branch = std::process::Command::new("git")
            .args(["symbolic-ref", "HEAD", "refs/heads/test-branch"])
            .current_dir(repo_dir.path())
            .output()
            .unwrap();
        assert!(branch.status.success());

        std::env::set_current_dir(repo_dir.path()).unwrap();

        let result = get_current_branch();

        std::env::set_current_dir(original_dir).unwrap();

        assert_eq!(result.unwrap(), "test-branch");
    }

    #[test]
    fn test_is_inside_repo_returns_bool() {
        let _ = is_inside_repo();
    }

    #[test]
    fn test_get_remote_host_parses_ssh_url() {
        let host = get_remote_host("git@gitlab.example.com:group/project.git").unwrap();

        assert_eq!(host, "gitlab.example.com");
    }

    #[test]
    fn test_get_remote_host_parses_https_url() {
        let host = get_remote_host("https://github.example.com/org/repo.git").unwrap();

        assert_eq!(host, "github.example.com");
    }

    #[test]
    fn test_get_remote_host_preserves_https_port() {
        let host = get_remote_host("https://git.example.com:8443/group/project.git").unwrap();

        assert_eq!(host, "git.example.com:8443");
    }

    #[test]
    fn test_clone_repository_invalid_url() {
        let dir = tempfile::tempdir().unwrap();

        let destination = dir.path().join("repo");
        let result = clone_repository(
            "/definitely/missing/gitfleet/repo.git",
            None,
            Some(destination.to_str().unwrap()),
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_clone_repository_invalid_url_with_depth() {
        let dir = tempfile::tempdir().unwrap();

        let destination = dir.path().join("repo");
        let result = clone_repository(
            "/definitely/missing/gitfleet/repo.git",
            Some(1),
            Some(destination.to_str().unwrap()),
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_config_error_from_git_failure() {
        let err = ConfigError::new("git command failed");

        assert_eq!(err.to_string(), "git command failed");
    }
}
