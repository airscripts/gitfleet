use std::io::Write;
use std::path::{Path, PathBuf};

use crate::errors::GitfleetError;
use crate::file_lock::FileLock;
use crate::provider::ProviderId;
use crate::repository::RepositoryRef;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Workspace {
    pub name: String,
    pub repositories: Vec<RepositoryRef>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WorkspacesFile {
    workspaces: Vec<Workspace>,
}

fn workspaces_path() -> Result<PathBuf, GitfleetError> {
    crate::config::gitfleet_folder()
        .map(|folder| folder.join("workspaces.toml"))
        .map_err(GitfleetError::from)
}

fn ensure_dir() -> Result<(), GitfleetError> {
    let path = workspaces_path()?;
    let dir = path
        .parent()
        .ok_or_else(|| GitfleetError::new("Workspace directory has no parent."))?;
    std::fs::create_dir_all(dir)
        .map_err(|e| GitfleetError::new(format!("Failed to create directory: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| GitfleetError::new(format!("Failed to secure directory: {e}")))?;
    }

    Ok(())
}

fn load_all() -> Result<Vec<Workspace>, GitfleetError> {
    let path = workspaces_path()?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    with_workspace_lock(false, load_all_unlocked)
}

fn load_all_unlocked() -> Result<Vec<Workspace>, GitfleetError> {
    let path = workspaces_path()?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| GitfleetError::new(format!("Failed to read workspaces: {e}")))?;

    let file: WorkspacesFile = toml::from_str(&content)
        .map_err(|e| GitfleetError::new(format!("Failed to parse workspaces: {e}")))?;

    Ok(file.workspaces)
}

fn save_all_unlocked(workspaces: &[Workspace]) -> Result<(), GitfleetError> {
    ensure_dir()?;

    let file = WorkspacesFile {
        workspaces: workspaces.to_vec(),
    };

    let content = toml::to_string_pretty(&file)
        .map_err(|e| GitfleetError::new(format!("Failed to serialize workspaces: {e}")))?;
    let path = workspaces_path()?;
    let temporary_path = path.with_extension(format!(
        "toml.{}.{}.tmp",
        std::process::id(),
        format_args!("{:?}", std::thread::current().id())
    ));
    let write_result = (|| -> Result<(), GitfleetError> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
            .map_err(|e| GitfleetError::new(format!("Failed to create workspaces: {e}")))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .map_err(|e| {
                    GitfleetError::new(format!("Failed to secure temporary workspaces file: {e}"))
                })?;
        }

        file.write_all(content.as_bytes())
            .map_err(|e| GitfleetError::new(format!("Failed to write workspaces: {e}")))?;
        file.sync_all()
            .map_err(|e| GitfleetError::new(format!("Failed to flush workspaces: {e}")))?;
        drop(file);

        replace_file(&temporary_path, &path)
            .map_err(|e| GitfleetError::new(format!("Failed to replace workspaces: {e}")))?;

        Ok(())
    })();

    if write_result.is_err() {
        let _ = std::fs::remove_file(&temporary_path);
    }

    write_result
}

fn with_workspace_lock<T, F>(exclusive: bool, operation: F) -> Result<T, GitfleetError>
where
    F: FnOnce() -> Result<T, GitfleetError>,
{
    ensure_dir()?;

    let path = workspaces_path()?;
    let lock_path = path.with_extension("toml.lock");
    let _lock = if exclusive {
        FileLock::exclusive(&lock_path)
    } else {
        FileLock::shared(&lock_path)
    }
    .map_err(|e| GitfleetError::new(format!("Failed to lock workspaces: {e}")))?;

    operation()
}

fn update_all<F>(update: F) -> Result<(), GitfleetError>
where
    F: FnOnce(&mut Vec<Workspace>) -> Result<(), GitfleetError>,
{
    with_workspace_lock(true, || {
        let mut workspaces = load_all_unlocked()?;

        update(&mut workspaces)?;

        save_all_unlocked(&workspaces)
    })
}

fn replace_file(temporary_path: &Path, destination: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;

        let source: Vec<u16> = temporary_path
            .as_os_str()
            .encode_wide()
            .chain(once(0))
            .collect();
        let target: Vec<u16> = destination
            .as_os_str()
            .encode_wide()
            .chain(once(0))
            .collect();

        let result = unsafe {
            windows_sys::Win32::Storage::FileSystem::MoveFileExW(
                source.as_ptr(),
                target.as_ptr(),
                windows_sys::Win32::Storage::FileSystem::MOVEFILE_REPLACE_EXISTING
                    | windows_sys::Win32::Storage::FileSystem::MOVEFILE_WRITE_THROUGH,
            )
        };

        if result == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    #[cfg(not(windows))]
    {
        std::fs::rename(temporary_path, destination)
    }
}

#[cfg(test)]
fn parse_repository(value: &str) -> Result<RepositoryRef, GitfleetError> {
    parse_repository_with_defaults(value, ProviderId::GitHub, "github.com")
}

fn parse_repository_with_defaults(
    value: &str,
    default_provider: ProviderId,
    default_host: &str,
) -> Result<RepositoryRef, GitfleetError> {
    let re = regex::Regex::new(r"(?i)^([a-z][a-z0-9-]*)@([^:]+):(.+)$")
        .expect("workspace regex is valid");

    if let Some(caps) = re.captures(value) {
        let provider_str = caps
            .get(1)
            .expect("capture group 1 matches provider")
            .as_str()
            .to_lowercase();

        let host = caps
            .get(2)
            .expect("capture group 2 matches host")
            .as_str()
            .to_string();

        let path = caps.get(3).expect("capture group 3 matches path").as_str();
        let provider = match provider_str.as_str() {
            "github" => ProviderId::GitHub,
            "gitlab" => ProviderId::GitLab,
            _ => {
                return Err(GitfleetError::new(format!(
                    "Unsupported repository provider \"{provider_str}\"."
                )))
            }
        };

        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        let name = segments
            .last()
            .expect("at least one segment after filtering")
            .to_string();

        let namespace = segments[..segments.len().saturating_sub(1)].join("/");

        if name.is_empty() || namespace.is_empty() {
            return Err(GitfleetError::new(format!(
                "Invalid repository \"{value}\". Expected namespace/repository."
            )));
        }

        return Ok(RepositoryRef {
            provider,
            host,
            namespace,
            name,
        });
    }

    let segments: Vec<&str> = value.split('/').filter(|s| !s.is_empty()).collect();

    if segments.len() < 2 {
        return Err(GitfleetError::new(format!(
            "Invalid repository \"{value}\". Expected namespace/repository."
        )));
    }

    let name = segments
        .last()
        .expect("at least two segments guaranteed by length check")
        .to_string();

    let namespace = segments[..segments.len() - 1].join("/");
    Ok(RepositoryRef {
        provider: default_provider,
        host: default_host.to_string(),
        namespace,
        name,
    })
}

pub fn define(name: &str, repos: &[String]) -> Result<Workspace, GitfleetError> {
    define_with_defaults(name, repos, ProviderId::GitHub, "github.com")
}

pub fn define_with_defaults(
    name: &str,
    repos: &[String],
    default_provider: ProviderId,
    default_host: &str,
) -> Result<Workspace, GitfleetError> {
    let repositories: Result<Vec<RepositoryRef>, GitfleetError> = repos
        .iter()
        .map(|r| parse_repository_with_defaults(r, default_provider, default_host))
        .collect();

    let workspace = Workspace {
        name: name.to_string(),
        repositories: repositories?,
    };

    update_all(|workspaces| {
        if let Some(idx) = workspaces.iter().position(|w| w.name == name) {
            workspaces[idx] = workspace.clone();
        } else {
            workspaces.push(workspace.clone());
        }

        Ok(())
    })?;

    Ok(workspace)
}

pub fn get(name: &str) -> Result<Workspace, GitfleetError> {
    let workspaces = load_all()?;
    workspaces
        .into_iter()
        .find(|w| w.name == name)
        .ok_or_else(|| GitfleetError::new(format!("Workspace \"{name}\" not found.")))
}

pub fn list() -> Result<Vec<Workspace>, GitfleetError> {
    load_all()
}

pub fn remove(name: &str) -> Result<(), GitfleetError> {
    update_all(|workspaces| {
        let original_len = workspaces.len();
        workspaces.retain(|w| w.name != name);

        if workspaces.len() == original_len {
            return Err(GitfleetError::new(format!(
                "Workspace \"{name}\" not found."
            )));
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEnvironment {
        _directory: tempfile::TempDir,
        original_home: Option<String>,
    }

    fn setup_test_env() -> TestEnvironment {
        let dir = tempfile::tempdir().unwrap();

        let gitfleet_dir = dir.path().join(".config").join("gitfleet");
        std::fs::create_dir_all(&gitfleet_dir).unwrap();

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", dir.path().to_string_lossy().to_string());

        TestEnvironment {
            _directory: dir,
            original_home,
        }
    }

    fn cleanup_test_env(environment: TestEnvironment) {
        if let Some(home) = environment.original_home {
            std::env::set_var("GITFLEET_HOME", home);
        } else {
            std::env::remove_var("GITFLEET_HOME");
        }
    }

    #[test]
    fn test_parse_repository_github_shorthand() {
        let result = parse_repository("org/repo").unwrap();

        assert_eq!(result.provider, ProviderId::GitHub);

        assert_eq!(result.host, "github.com");
        assert_eq!(result.namespace, "org");

        assert_eq!(result.name, "repo");
    }

    #[test]
    fn test_parse_repository_gitlab_shorthand_with_defaults() {
        let result =
            parse_repository_with_defaults("group/repo", ProviderId::GitLab, "gitlab.com").unwrap();

        assert_eq!(result.provider, ProviderId::GitLab);
        assert_eq!(result.host, "gitlab.com");
        assert_eq!(result.namespace, "group");
        assert_eq!(result.name, "repo");
    }

    #[test]
    fn test_parse_repository_qualified() {
        let result = parse_repository("github@github.com:org/repo").unwrap();

        assert_eq!(result.provider, ProviderId::GitHub);

        assert_eq!(result.host, "github.com");
        assert_eq!(result.namespace, "org");

        assert_eq!(result.name, "repo");
    }

    #[test]
    fn test_parse_repository_invalid_too_short() {
        let result = parse_repository("single");

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_repository_nested_path() {
        let result = parse_repository("org/subgroup/repo").unwrap();

        assert_eq!(result.namespace, "org/subgroup");

        assert_eq!(result.name, "repo");
    }

    #[test]
    fn test_parse_repository_gitlab_qualified() {
        let result = parse_repository("gitlab@gitlab.com:org/repo").unwrap();

        assert_eq!(result.provider, ProviderId::GitLab);

        assert_eq!(result.host, "gitlab.com");
        assert_eq!(result.namespace, "org");

        assert_eq!(result.name, "repo");
    }

    #[test]
    fn test_parse_repository_unsupported_provider() {
        let result = parse_repository("bitbucket@bitbucket.org:org/repo");

        assert!(result.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test_workspace_define_and_list() {
        let dir = setup_test_env();

        let repos = vec!["org/repo1".to_string(), "org/repo2".to_string()];
        let ws = define("test-ws", &repos).unwrap();

        assert_eq!(ws.name, "test-ws");

        assert_eq!(ws.repositories.len(), 2);

        let listed = list().unwrap();

        assert!(listed.iter().any(|w| w.name == "test-ws"));

        cleanup_test_env(dir);
    }

    #[test]
    #[serial_test::serial]
    fn test_workspace_define_overwrite() {
        let dir = setup_test_env();

        let repos1 = vec!["org/repo1".to_string()];
        let repos2 = vec!["org/repo2".to_string(), "org/repo3".to_string()];
        define("overwrite-ws", &repos1).unwrap();
        define("overwrite-ws", &repos2).unwrap();

        let listed = list().unwrap();

        let ws = listed.iter().find(|w| w.name == "overwrite-ws").unwrap();

        assert_eq!(ws.repositories.len(), 2);

        cleanup_test_env(dir);
    }

    #[test]
    #[serial_test::serial]
    fn test_workspace_remove() {
        let dir = setup_test_env();

        let repos = vec!["org/repo1".to_string()];
        define("remove-ws", &repos).unwrap();
        remove("remove-ws").unwrap();

        let listed = list().unwrap();

        assert!(listed.iter().all(|w| w.name != "remove-ws"));

        cleanup_test_env(dir);
    }

    #[test]
    #[serial_test::serial]
    fn test_workspace_remove_nonexistent() {
        let dir = setup_test_env();

        let result = remove("nonexistent-ws-xyz");

        assert!(result.is_err());

        cleanup_test_env(dir);
    }

    #[test]
    #[serial_test::serial]
    fn test_workspace_get() {
        let dir = setup_test_env();

        let repos = vec!["org/repo1".to_string()];
        define("get-ws", &repos).unwrap();

        let ws = get("get-ws").unwrap();

        assert_eq!(ws.name, "get-ws");

        cleanup_test_env(dir);
    }

    #[test]
    #[serial_test::serial]
    fn test_workspace_get_nonexistent() {
        let dir = setup_test_env();

        let result = get("nonexistent-ws-xyz");

        assert!(result.is_err());

        cleanup_test_env(dir);
    }

    #[test]
    fn test_workspace_struct_serialization() {
        let ws = Workspace {
            name: "test".to_string(),
            repositories: vec![RepositoryRef {
                provider: ProviderId::GitHub,
                host: "github.com".to_string(),
                namespace: "org".to_string(),
                name: "repo".to_string(),
            }],
        };

        let file = WorkspacesFile {
            workspaces: vec![ws],
        };

        let toml_str = toml::to_string_pretty(&file).unwrap();

        let deserialized: WorkspacesFile = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.workspaces.len(), 1);

        assert_eq!(deserialized.workspaces[0].name, "test");
        assert_eq!(deserialized.workspaces[0].repositories.len(), 1);
    }

    #[test]
    #[serial_test::serial]
    fn test_workspace_list_empty() {
        let dir = setup_test_env();

        let listed = list().unwrap();

        for ws in &listed {
            let _ = remove(&ws.name);
        }

        let listed = list().unwrap();

        assert!(listed.is_empty());

        cleanup_test_env(dir);
    }
}
