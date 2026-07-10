use std::path::PathBuf;

use dirs::home_dir;

use crate::constants::{
    CREDENTIALS_FILE, DEFAULT_PROFILE_NAME, GITFLEET_FOLDER, GITFLEET_PROFILE_ENV,
};
use crate::errors::ConfigError;
use crate::provider::ProviderId;
use crate::types::{CredentialsFile, Profile, ProfileRcFile};

fn gitfleet_folder() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(GITFLEET_FOLDER)
}

fn credentials_path() -> PathBuf {
    gitfleet_folder().join(CREDENTIALS_FILE)
}

pub fn read_credentials() -> Result<CredentialsFile, ConfigError> {
    let path = credentials_path();

    if !path.exists() {
        return Ok(CredentialsFile {
            active_profile: DEFAULT_PROFILE_NAME.to_string(),
            profiles: std::collections::HashMap::new(),
            aliases: std::collections::HashMap::new(),
        });
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| ConfigError::new(format!("Invalid credentials file: {e}")))?;

    let creds: CredentialsFile = toml::from_str(&content)
        .map_err(|e| ConfigError::new(format!("Invalid credentials file: {e}")))?;

    Ok(creds)
}

pub fn write_credentials(creds: &CredentialsFile) -> Result<(), ConfigError> {
    let folder = gitfleet_folder();
    std::fs::create_dir_all(&folder)
        .map_err(|e| ConfigError::new(format!("Failed to create config directory: {e}")))?;

    let content = toml::to_string_pretty(creds)
        .map_err(|e| ConfigError::new(format!("Failed to serialize credentials: {e}")))?;

    let path = credentials_path();
    std::fs::write(&path, &content)
        .map_err(|e| ConfigError::new(format!("Failed to write credentials: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, perms).map_err(|e| {
            ConfigError::new(format!("Failed to set credentials file permissions: {e}"))
        })?;
    }

    Ok(())
}

pub fn get_token() -> Result<String, ConfigError> {
    get_token_optional().ok_or_else(|| ConfigError::new(crate::constants::ERROR_NO_TOKEN))
}

pub fn get_token_optional() -> Option<String> {
    let provider = get_resolved_profile()
        .ok()
        .and_then(|name| get_profile(&name).ok().flatten())
        .and_then(|profile| match profile.provider.as_deref() {
            Some("gitlab") => Some(ProviderId::GitLab),
            Some("github") | None => Some(ProviderId::GitHub),
            Some(_) => None,
        })
        .unwrap_or(ProviderId::GitHub);

    get_provider_token_optional(provider)
}

pub fn get_provider_token_optional(provider: ProviderId) -> Option<String> {
    match provider {
        ProviderId::GitHub => {
            if let Ok(token) = std::env::var("GITFLEET_GITHUB_TOKEN") {
                if !token.is_empty() {
                    return Some(token);
                }
            }
        }

        ProviderId::GitLab => {
            if let Ok(token) = std::env::var("GITFLEET_GITLAB_TOKEN") {
                if !token.is_empty() {
                    return Some(token);
                }
            }
        }
    }

    read("token")
}

pub fn get_github_token_optional() -> Option<String> {
    if let Ok(token) = std::env::var("GITFLEET_GITHUB_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }

    read("token")
}

pub fn get_gitlab_token_optional() -> Option<String> {
    if let Ok(token) = std::env::var("GITFLEET_GITLAB_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }

    read("token")
}

pub fn get_resolved_profile() -> Result<String, ConfigError> {
    let creds = read_credentials()?;

    if let Ok(env_profile) = std::env::var(GITFLEET_PROFILE_ENV) {
        if creds.profiles.contains_key(&env_profile) {
            return Ok(env_profile);
        }

        return Err(ConfigError::new(crate::constants::ERROR_PROFILE_NOT_FOUND));
    }

    if let Some(repo_profile) = get_repo_local_profile() {
        if creds.profiles.contains_key(&repo_profile) {
            return Ok(repo_profile);
        }
    }

    if creds.profiles.contains_key(&creds.active_profile) {
        return Ok(creds.active_profile.clone());
    }

    let mut sorted_keys: Vec<String> = creds.profiles.keys().cloned().collect();
    sorted_keys.sort();

    if let Some(first) = sorted_keys.into_iter().next() {
        return Ok(first);
    }

    Ok(DEFAULT_PROFILE_NAME.to_string())
}

pub fn get_profile(name: &str) -> Result<Option<Profile>, ConfigError> {
    let creds = read_credentials()?;

    Ok(creds.profiles.get(name).cloned())
}

pub fn find_profile_by_host(host: &str) -> Result<Option<String>, ConfigError> {
    let creds = read_credentials()?;
    let normalized_host = host.trim_end_matches('/').to_ascii_lowercase();

    let mut matches: Vec<String> = creds
        .profiles
        .iter()
        .filter_map(|(name, profile)| {
            profile.host.as_ref().and_then(|profile_host| {
                (profile_host
                    .trim_end_matches('/')
                    .eq_ignore_ascii_case(&normalized_host))
                .then(|| name.clone())
            })
        })
        .collect();

    matches.sort();
    Ok(matches.into_iter().next())
}

pub fn list_profiles() -> Result<Vec<ProfileEntry>, ConfigError> {
    let creds = read_credentials()?;

    let active = get_resolved_profile().ok();
    let mut entries = Vec::new();

    for (name, profile) in &creds.profiles {
        entries.push(ProfileEntry {
            name: name.clone(),
            active: active.as_deref() == Some(name.as_str()),
            has_token: profile.token.is_some(),
        });
    }

    Ok(entries)
}

pub fn add_profile(name: &str, profile: Profile) -> Result<(), ConfigError> {
    let mut creds = read_credentials()?;

    let is_first = creds.profiles.is_empty();
    creds.profiles.insert(name.to_string(), profile);

    if is_first {
        creds.active_profile = name.to_string();
    }

    write_credentials(&creds)
}

pub fn set_active_profile(name: &str) -> Result<(), ConfigError> {
    let mut creds = read_credentials()?;

    if !creds.profiles.contains_key(name) {
        return Err(ConfigError::new(crate::constants::ERROR_PROFILE_NOT_FOUND));
    }

    creds.active_profile = name.to_string();
    write_credentials(&creds)
}

pub fn read(key: &str) -> Option<String> {
    let creds = read_credentials().ok()?;

    let profile_name = get_resolved_profile().ok()?;
    let profile = creds.profiles.get(&profile_name)?;

    match key {
        "token" => profile.token.clone(),
        "host" => profile.host.clone(),
        _ => profile.extra.get(key).cloned(),
    }
}

pub fn write(key: &str, value: &str) -> Result<(), ConfigError> {
    let mut creds = read_credentials()?;

    let profile_name = get_resolved_profile()?;
    let profile = creds
        .profiles
        .entry(profile_name.clone())
        .or_insert(Profile {
            token: None,
            host: None,
            provider: None,
            extra: std::collections::HashMap::new(),
        });

    match key {
        "token" => profile.token = Some(value.to_string()),
        "host" => profile.host = Some(value.to_string()),
        _ => {
            profile.extra.insert(key.to_string(), value.to_string());
        }
    }

    write_credentials(&creds)
}

pub fn unset(key: &str) -> Result<(), ConfigError> {
    let mut creds = read_credentials()?;

    let profile_name = get_resolved_profile()?;
    let profile = creds
        .profiles
        .get_mut(&profile_name)
        .ok_or_else(|| ConfigError::new(format!("Profile \"{profile_name}\" not found.")))?;

    match key {
        "token" => profile.token = None,
        "host" => profile.host = None,
        _ => {
            if profile.extra.remove(key).is_none() {
                return Err(ConfigError::new(crate::constants::ERROR_UNSUPPORTED_KEY));
            }
        }
    }

    write_credentials(&creds)
}

fn get_repo_local_profile() -> Option<String> {
    let repo_root = crate::git::get_repo_root().ok()?;

    let rc_path = repo_root.join(crate::constants::GITFLEET_RC_FILE);

    if !rc_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&rc_path).ok()?;

    let rc: ProfileRcFile = toml::from_str(&content).ok()?;
    rc.profile
}

pub fn get_host() -> String {
    read("host").unwrap_or_else(|| "github.com".to_string())
}

pub fn remove_profile(name: &str) -> Result<(), ConfigError> {
    let mut creds = read_credentials()?;

    if creds.profiles.remove(name).is_none() {
        return Err(ConfigError::new(format!("Profile \"{name}\" not found.")));
    }

    write_credentials(&creds)
}

pub fn clear_credentials() -> Result<(), ConfigError> {
    let path = credentials_path();

    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| ConfigError::new(format!("Failed to remove credentials: {e}")))?;
    }

    Ok(())
}

pub fn set_alias(name: &str, expansion: &str, force: bool) -> Result<(), ConfigError> {
    let mut creds = read_credentials()?;

    if !force && creds.aliases.contains_key(name) {
        return Err(ConfigError::new(format!(
            "Alias '{name}' already exists. Use --force to overwrite."
        )));
    }

    creds
        .aliases
        .insert(name.to_string(), expansion.to_string());
    write_credentials(&creds)
}

pub fn get_alias(name: &str) -> Option<String> {
    let creds = read_credentials().ok()?;
    creds.aliases.get(name).cloned()
}

pub fn list_aliases() -> Result<Vec<crate::types::AliasEntry>, ConfigError> {
    let creds = read_credentials()?;

    let mut entries: Vec<crate::types::AliasEntry> = creds
        .aliases
        .iter()
        .map(|(name, expansion)| crate::types::AliasEntry {
            name: name.clone(),
            expansion: expansion.clone(),
        })
        .collect();

    entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(entries)
}

pub fn delete_alias(name: &str) -> Result<(), ConfigError> {
    let mut creds = read_credentials()?;

    if creds.aliases.remove(name).is_none() {
        return Err(ConfigError::new(format!("Alias '{name}' not found.")));
    }

    write_credentials(&creds)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfileEntry {
    pub name: String,
    pub active: bool,
    pub has_token: bool,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_profile_toml_serialization() {
        let profile = Profile {
            token: Some("ghp_abc123".into()),
            host: Some("github.com".into()),
            provider: Some("github".into()),
            extra: Default::default(),
        };

        let toml_str = toml::to_string(&profile).unwrap();

        let deserialized: Profile = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.token, Some("ghp_abc123".into()));

        assert_eq!(deserialized.host, Some("github.com".into()));
        assert_eq!(deserialized.provider, Some("github".into()));
    }

    #[test]
    fn test_profile_skips_none_fields() {
        let profile = Profile {
            token: None,
            host: None,
            provider: None,
            extra: Default::default(),
        };

        let toml_str = toml::to_string(&profile).unwrap();

        assert!(!toml_str.contains("token"));

        assert!(!toml_str.contains("host"));
        assert!(!toml_str.contains("provider"));

        let deserialized: Profile = toml::from_str(&toml_str).unwrap();

        assert!(deserialized.token.is_none());
    }

    #[test]
    fn test_credentials_file_toml_round_trip() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "default".into(),
            Profile {
                token: Some("ghp_test".into()),
                host: None,
                provider: None,
                extra: Default::default(),
            },
        );

        let cf = CredentialsFile {
            active_profile: "default".into(),
            profiles,
            aliases: std::collections::HashMap::new(),
        };

        let toml_str = toml::to_string_pretty(&cf).unwrap();

        let deserialized: CredentialsFile = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.active_profile, "default");

        assert!(deserialized.profiles.contains_key("default"));
        assert_eq!(
            deserialized.profiles.get("default").unwrap().token,
            Some("ghp_test".into())
        );
    }

    #[test]
    fn test_credentials_file_round_trip_empty_profiles() {
        let cf = CredentialsFile {
            active_profile: "default".into(),
            profiles: HashMap::new(),
            aliases: HashMap::new(),
        };

        let toml_str = toml::to_string(&cf).unwrap();

        let result: CredentialsFile = toml::from_str(&toml_str).unwrap();

        assert!(result.profiles.is_empty());
    }

    #[test]
    #[serial_test::serial]
    fn test_read_credentials_returns_default_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();

        std::env::set_var("HOME", dir.path().to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var("GITFLEET_PROFILE");

        let creds = read_credentials().unwrap();

        assert_eq!(creds.active_profile, DEFAULT_PROFILE_NAME);

        assert!(creds.profiles.is_empty());
        std::env::remove_var("HOME");
    }

    #[test]
    #[serial_test::serial]
    fn test_write_and_read_credentials() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_write");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());

        let mut profiles = HashMap::new();
        profiles.insert(
            "test-profile".into(),
            Profile {
                token: Some("ghp_testtoken".into()),
                host: Some("github.com".into()),
                provider: Some("github".into()),
                extra: Default::default(),
            },
        );

        let creds = CredentialsFile {
            active_profile: "test-profile".into(),
            profiles,
            aliases: HashMap::new(),
        };

        write_credentials(&creds).unwrap();

        let read_creds = read_credentials().unwrap();

        assert_eq!(read_creds.active_profile, "test-profile");

        assert!(read_creds.profiles.contains_key("test-profile"));

        let _ = std::fs::remove_file(&creds_path);

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_get_token_optional_reads_env_var() {
        std::env::set_var("GITFLEET_GITHUB_TOKEN", "test-token-123");

        let token = get_token_optional();

        assert_eq!(token, Some("test-token-123".into()));

        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
    }

    #[test]
    #[serial_test::serial]
    fn test_get_provider_token_optional_reads_gitlab_env_var() {
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::set_var("GITFLEET_GITLAB_TOKEN", "gl-token-123");

        let token = get_provider_token_optional(ProviderId::GitLab);

        assert_eq!(token, Some("gl-token-123".into()));

        std::env::remove_var("GITFLEET_GITLAB_TOKEN");
    }

    #[test]
    #[serial_test::serial]
    fn test_get_token_optional_empty_env_var() {
        std::env::set_var("GITFLEET_GITHUB_TOKEN", "");

        let token = get_token_optional();

        assert!(token.is_none() || token != Some(String::new()));

        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
    }

    #[test]
    #[serial_test::serial]
    fn test_get_token_returns_error_when_none() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_get_token_none");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());

        let result = get_token();

        match original_home {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }

        assert!(result.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test_add_profile_and_list() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_add_profile");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let profile = Profile {
            token: Some("ghp_new".into()),
            host: Some("github.com".into()),
            provider: Some("github".into()),
            extra: Default::default(),
        };

        add_profile("test-add", profile).unwrap();

        let entries = list_profiles().unwrap();

        assert!(entries.iter().any(|e| e.name == "test-add"));

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_set_active_profile() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_active");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let profile = Profile {
            token: Some("ghp_test".into()),
            host: None,
            provider: None,
            extra: Default::default(),
        };

        add_profile("active-profile", profile).unwrap();
        set_active_profile("active-profile").unwrap();

        let creds = read_credentials().unwrap();

        assert_eq!(creds.active_profile, "active-profile");

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_set_active_profile_nonexistent() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_active_err");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let result = set_active_profile("nonexistent");

        assert!(result.is_err());

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_remove_profile() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_remove");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let profile = Profile {
            token: Some("ghp_test".into()),
            host: None,
            provider: None,
            extra: Default::default(),
        };

        add_profile("to-remove", profile).unwrap();
        remove_profile("to-remove").unwrap();

        let entries = list_profiles().unwrap();

        assert!(!entries.iter().any(|e| e.name == "to-remove"));

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_remove_nonexistent_profile() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_remove_err");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let result = remove_profile("nonexistent");

        assert!(result.is_err());

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_clear_credentials() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_clear");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let profile = Profile {
            token: Some("ghp_test".into()),
            host: None,
            provider: None,
            extra: Default::default(),
        };

        add_profile("clear-test", profile).unwrap();
        clear_credentials().unwrap();

        let creds = read_credentials().unwrap();

        assert!(creds.profiles.is_empty());

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_get_host_default() {
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");

        let host = get_host();

        assert_eq!(host, "github.com");
    }

    #[test]
    #[serial_test::serial]
    fn test_get_profile_existing() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_get_profile");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let profile = Profile {
            token: Some("ghp_test".into()),
            host: Some("github.com".into()),
            provider: Some("github".into()),
            extra: Default::default(),
        };

        add_profile("profile-test", profile.clone()).unwrap();

        let found = get_profile("profile-test").unwrap();

        assert!(found.is_some());

        assert_eq!(found.unwrap().token, Some("ghp_test".into()));

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_get_profile_nonexistent() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_get_profile_err");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let found = get_profile("nonexistent").unwrap();

        assert!(found.is_none());

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_write_and_read_key() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_write_key");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let profile = Profile {
            token: Some("ghp_test".into()),
            host: None,
            provider: None,
            extra: Default::default(),
        };

        add_profile("write-test", profile).unwrap();
        set_active_profile("write-test").unwrap();
        write("host", "gitlab.com").unwrap();

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_unset_key() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_unset");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let profile = Profile {
            token: Some("ghp_test".into()),
            host: Some("github.com".into()),
            provider: None,
            extra: Default::default(),
        };

        add_profile("unset-test", profile).unwrap();
        set_active_profile("unset-test").unwrap();
        unset("token").unwrap();

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_profile_entry_fields() {
        let entry = ProfileEntry {
            name: "default".into(),
            active: true,
            has_token: true,
        };

        assert_eq!(entry.name, "default");

        assert!(entry.active);
        assert!(entry.has_token);
    }

    #[test]
    #[serial_test::serial]
    fn test_set_alias_new() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_set_alias");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        set_alias("ll", "repo list", false).unwrap();

        let expansion = get_alias("ll");

        assert_eq!(expansion, Some("repo list".to_string()));

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_set_alias_duplicate_without_force() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_alias_dup");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        set_alias("co", "checkout", false).unwrap();

        let result = set_alias("co", "checkout -b", false);

        assert!(result.is_err());

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_set_alias_overwrite_with_force() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_alias_force");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        set_alias("co", "checkout", false).unwrap();
        set_alias("co", "checkout -b", true).unwrap();

        let expansion = get_alias("co");

        assert_eq!(expansion, Some("checkout -b".to_string()));

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_get_alias_nonexistent() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_alias_get");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let result = get_alias("nonexistent");

        assert_eq!(result, None);

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_list_aliases_empty() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_alias_list_empty");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let entries = list_aliases().unwrap();

        assert!(entries.is_empty());

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_list_aliases_sorted() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_alias_list_sorted");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        set_alias("zebra", "repo list", false).unwrap();
        set_alias("alpha", "repo view", false).unwrap();
        set_alias("middle", "repo clone", false).unwrap();

        let entries = list_aliases().unwrap();

        assert_eq!(entries.len(), 3);

        assert_eq!(entries[0].name, "alpha");
        assert_eq!(entries[1].name, "middle");

        assert_eq!(entries[2].name, "zebra");

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_delete_alias() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_alias_delete");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        set_alias("temp", "repo list", false).unwrap();
        delete_alias("temp").unwrap();

        assert_eq!(get_alias("temp"), None);

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_delete_alias_nonexistent() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_alias_delete_err");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let result = delete_alias("nonexistent");

        assert!(result.is_err());

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_aliases_persist_across_writes() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_alias_persist");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        set_alias("ll", "repo list", false).unwrap();
        set_alias("co", "checkout", false).unwrap();

        let entries = list_aliases().unwrap();

        assert_eq!(entries.len(), 2);

        delete_alias("ll").unwrap();

        let entries = list_aliases().unwrap();

        assert_eq!(entries.len(), 1);

        assert_eq!(entries[0].name, "co");

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_aliases_survive_profile_operations() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_alias_with_profiles");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        set_alias("ll", "repo list", false).unwrap();

        let profile = Profile {
            token: Some("ghp_test".into()),
            host: None,
            provider: None,
            extra: Default::default(),
        };

        add_profile("test-profile", profile).unwrap();

        let expansion = get_alias("ll");

        assert_eq!(expansion, Some("repo list".to_string()));

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_alias_toml_round_trip() {
        let mut creds = CredentialsFile {
            active_profile: "default".into(),
            profiles: HashMap::new(),
            aliases: HashMap::new(),
        };

        creds.aliases.insert("ll".into(), "repo list".into());
        creds.aliases.insert("co".into(), "checkout".into());

        let toml_str = toml::to_string_pretty(&creds).unwrap();

        assert!(toml_str.contains("[aliases]"));

        assert!(toml_str.contains("ll = \"repo list\""));

        let deserialized: CredentialsFile = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.aliases.len(), 2);

        assert_eq!(
            deserialized.aliases.get("ll"),
            Some(&"repo list".to_string())
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_alias_deserialize_without_aliases_field() {
        let toml_str = r#"
active_profile = "default"

[profiles.default]
token = "ghp_test"
"#;
        let creds: CredentialsFile = toml::from_str(toml_str).unwrap();

        assert!(creds.aliases.is_empty());
    }

    #[test]
    #[serial_test::serial]
    fn test_write_read_arbitrary_key() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_arbitrary");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let profile = Profile {
            token: Some("ghp_test".into()),
            host: Some("github.com".into()),
            provider: Some("github".into()),
            extra: Default::default(),
        };

        add_profile(DEFAULT_PROFILE_NAME, profile).unwrap();

        write("default_org", "myorg").unwrap();

        let value = read("default_org");

        assert_eq!(value, Some("myorg".to_string()));

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_unset_arbitrary_key() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_unset_arb");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let profile = Profile {
            token: Some("ghp_test".into()),
            host: Some("github.com".into()),
            provider: Some("github".into()),
            extra: Default::default(),
        };

        add_profile(DEFAULT_PROFILE_NAME, profile).unwrap();

        write("custom_key", "custom_value").unwrap();
        unset("custom_key").unwrap();

        assert!(read("custom_key").is_none());

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_unset_nonexistent_key_errors() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_unset_nonexist");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let creds_path = tmp_dir.join(GITFLEET_FOLDER).join(CREDENTIALS_FILE);

        if creds_path.exists() {
            let _ = std::fs::remove_file(&creds_path);
        }

        let original_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let profile = Profile {
            token: Some("ghp_test".into()),
            host: Some("github.com".into()),
            provider: Some("github".into()),
            extra: Default::default(),
        };

        add_profile(DEFAULT_PROFILE_NAME, profile).unwrap();

        let result = unset("nonexistent_key");

        assert!(result.is_err());

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    fn test_profile_extra_toml_round_trip() {
        let mut extra = HashMap::new();
        extra.insert("default_org".into(), "myorg".into());
        extra.insert("theme".into(), "dark".into());

        let profile = Profile {
            token: Some("ghp_abc".into()),
            host: None,
            provider: None,
            extra,
        };

        let toml_str = toml::to_string(&profile).unwrap();

        let deserialized: Profile = toml::from_str(&toml_str).unwrap();

        assert_eq!(
            deserialized.extra.get("default_org"),
            Some(&"myorg".to_string())
        );

        assert_eq!(deserialized.extra.get("theme"), Some(&"dark".to_string()));
    }

    #[test]
    fn test_profile_deserialize_without_extra() {
        let toml_str = r#"token = "ghp_test"
host = "github.com"
"#;
        let profile: Profile = toml::from_str(toml_str).unwrap();

        assert_eq!(profile.token, Some("ghp_test".into()));

        assert!(profile.extra.is_empty());
    }
}
