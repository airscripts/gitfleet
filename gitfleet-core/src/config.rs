use std::io::Write;
use std::path::{Path, PathBuf};

use dirs::home_dir;

use crate::constants::{
    CREDENTIALS_FILE, DEFAULT_PROFILE_NAME, GITFLEET_CREDENTIAL_STORE_ENV, GITFLEET_FOLDER,
    GITFLEET_HOME_ENV, GITFLEET_PROFILE_ENV, GITFLEET_TEST_CREDENTIAL_STORE_ENV,
    GITFLEET_TRUST_REPO_CONFIG_ENV, KEYRING_SERVICE,
};
use crate::errors::ConfigError;
use crate::file_lock::FileLock;
use crate::provider::{ProviderCapability, ProviderContext, ProviderId, TokenSource};
use crate::types::{CredentialsFile, Profile, ProfileRcFile};

pub(crate) fn gitfleet_folder() -> Result<PathBuf, ConfigError> {
    if let Some(home) = std::env::var_os(GITFLEET_HOME_ENV).filter(|home| !home.is_empty()) {
        return Ok(PathBuf::from(home).join(GITFLEET_FOLDER));
    }

    home_dir()
        .map(|home| home.join(GITFLEET_FOLDER))
        .ok_or_else(|| ConfigError::new("Unable to determine the home directory."))
}

fn credentials_path() -> Result<PathBuf, ConfigError> {
    Ok(gitfleet_folder()?.join(CREDENTIALS_FILE))
}

fn with_credentials_lock<T, F>(exclusive: bool, operation: F) -> Result<T, ConfigError>
where
    F: FnOnce() -> Result<T, ConfigError>,
{
    let folder = gitfleet_folder()?;
    std::fs::create_dir_all(&folder)
        .map_err(|e| ConfigError::new(format!("Failed to create config directory: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| ConfigError::new(format!("Failed to secure config directory: {e}")))?;
    }

    let lock_path = folder.join(format!(".{CREDENTIALS_FILE}.lock"));
    let _lock = if exclusive {
        FileLock::exclusive(&lock_path)
    } else {
        FileLock::shared(&lock_path)
    }
    .map_err(|e| ConfigError::new(format!("Failed to lock credentials: {e}")))?;

    operation()
}

pub fn read_credentials() -> Result<CredentialsFile, ConfigError> {
    let path = credentials_path()?;

    if !path.exists() {
        return Ok(default_credentials());
    }

    with_credentials_lock(false, read_credentials_unlocked)
}

fn default_credentials() -> CredentialsFile {
    CredentialsFile {
        active_profile: DEFAULT_PROFILE_NAME.to_string(),
        profiles: std::collections::HashMap::new(),
        aliases: std::collections::HashMap::new(),
    }
}

fn read_credentials_unlocked() -> Result<CredentialsFile, ConfigError> {
    let path = credentials_path()?;

    if !path.exists() {
        return Ok(default_credentials());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| ConfigError::new(format!("Invalid credentials file: {e}")))?;

    let mut creds: CredentialsFile = toml::from_str(&content)
        .map_err(|e| ConfigError::new(format!("Invalid credentials file: {e}")))?;

    if !uses_file_credential_store() {
        for (name, profile) in &mut creds.profiles {
            if profile.token.is_none() {
                profile.token = load_token(name);
            }
        }
    }

    Ok(creds)
}

pub fn write_credentials(creds: &CredentialsFile) -> Result<(), ConfigError> {
    with_credentials_lock(true, || write_credentials_unlocked(creds))
}

fn write_credentials_unlocked(creds: &CredentialsFile) -> Result<(), ConfigError> {
    if uses_file_credential_store() {
        let previous = read_credentials_unlocked()?;

        write_credentials_file_unlocked(creds)?;

        let profile_names: std::collections::HashSet<&String> = previous
            .profiles
            .keys()
            .chain(creds.profiles.keys())
            .collect();

        for name in profile_names {
            delete_profile_token(name)?;
        }

        return Ok(());
    }

    let previous = read_credentials_unlocked()?;

    if let Err(error) = write_credentials_file_unlocked(creds) {
        let _ = write_credentials_file_unlocked(&previous);

        return Err(error);
    }

    if let Err(error) = sync_keyring(&previous, creds) {
        let keyring_rollback = sync_keyring(creds, &previous);
        let file_rollback = write_credentials_file_unlocked(&previous);

        if keyring_rollback.is_err() || file_rollback.is_err() {
            return Err(ConfigError::new(format!(
                "{error} Credential rollback was incomplete; inspect the configured profiles before retrying."
            )));
        }

        return Err(error);
    }

    Ok(())
}

fn write_credentials_file_unlocked(creds: &CredentialsFile) -> Result<(), ConfigError> {
    let folder = gitfleet_folder()?;
    std::fs::create_dir_all(&folder)
        .map_err(|e| ConfigError::new(format!("Failed to create config directory: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| ConfigError::new(format!("Failed to secure config directory: {e}")))?;
    }

    let serializable_creds = if uses_file_credential_store() {
        creds.clone()
    } else {
        let mut metadata = creds.clone();
        for profile in metadata.profiles.values_mut() {
            profile.token = None;
        }
        metadata
    };

    let content = toml::to_string_pretty(&serializable_creds)
        .map_err(|e| ConfigError::new(format!("Failed to serialize credentials: {e}")))?;

    let path = credentials_path()?;
    let temporary_path = folder.join(format!(
        ".{CREDENTIALS_FILE}.{}.{}.tmp",
        std::process::id(),
        format_args!("{:?}", std::thread::current().id())
    ));

    let write_result = (|| -> Result<(), ConfigError> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);

        let mut file = options.open(&temporary_path).map_err(|e| {
            ConfigError::new(format!("Failed to create temporary credentials file: {e}"))
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .map_err(|e| {
                    ConfigError::new(format!("Failed to secure temporary credentials file: {e}"))
                })?;
        }

        file.write_all(content.as_bytes())
            .map_err(|e| ConfigError::new(format!("Failed to write credentials: {e}")))?;
        file.sync_all()
            .map_err(|e| ConfigError::new(format!("Failed to flush credentials: {e}")))?;
        drop(file);

        replace_file(&temporary_path, &path)
            .map_err(|e| ConfigError::new(format!("Failed to replace credentials: {e}")))?;

        Ok(())
    })();

    if write_result.is_err() {
        let _ = std::fs::remove_file(&temporary_path);
    }

    write_result?;

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
    resolve_provider_context()
        .ok()
        .and_then(|context| context.token)
}

pub fn resolve_provider_context() -> Result<ProviderContext, ConfigError> {
    let profile_name = get_resolved_profile()?;

    let profile = get_profile(&profile_name)?.unwrap_or(Profile {
        token: None,
        host: None,
        provider: None,
        extra: Default::default(),
    });

    let provider = match profile.provider.as_deref() {
        Some("github") | None => ProviderId::GitHub,
        Some("gitlab") => ProviderId::GitLab,
        Some(provider) => {
            return Err(ConfigError::new(format!(
                "Unsupported provider '{provider}' in profile '{profile_name}'."
            )))
        }
    };

    let host = normalize_host(profile.host.as_deref().unwrap_or(match provider {
        ProviderId::GitHub => "github.com",
        ProviderId::GitLab => "gitlab.com",
    }))?;

    let environment_token = match provider {
        ProviderId::GitHub => std::env::var("GITFLEET_GITHUB_TOKEN").ok(),
        ProviderId::GitLab => std::env::var("GITFLEET_GITLAB_TOKEN").ok(),
    }
    .filter(|token| !token.is_empty())
    .filter(|_| host.eq_ignore_ascii_case(default_host(provider)));

    let (token, token_source) = match (profile.token, environment_token) {
        (Some(token), _) => (Some(token), TokenSource::Profile),
        (None, Some(token)) => (Some(token), TokenSource::Environment),
        (None, None) => (None, TokenSource::None),
    };

    Ok(ProviderContext {
        profile_name,
        provider,
        host,
        token,
        token_source,
        capabilities: Vec::<ProviderCapability>::new(),
    })
}

pub fn get_provider_token_optional(provider: ProviderId) -> Option<String> {
    if let Ok(context) = resolve_provider_context() {
        if context.provider == provider {
            return context.token;
        }
    }

    let environment_token = match provider {
        ProviderId::GitHub => std::env::var("GITFLEET_GITHUB_TOKEN").ok(),
        ProviderId::GitLab => std::env::var("GITFLEET_GITLAB_TOKEN").ok(),
    };

    environment_token.filter(|token| !token.is_empty())
}

pub fn get_token_for_host(host: &str) -> Option<String> {
    let normalized_host = normalize_host(host).ok()?;

    if let Ok(context) = resolve_provider_context() {
        if context.host.eq_ignore_ascii_case(&normalized_host) {
            return context.token;
        }
    }

    let profile_name = find_profile_by_host(&normalized_host).ok().flatten()?;
    let profile = get_profile(&profile_name).ok().flatten()?;
    let provider = match profile.provider.as_deref() {
        Some("github") | None => ProviderId::GitHub,
        Some("gitlab") => ProviderId::GitLab,
        Some(_) => return None,
    };

    let environment_token = match provider {
        ProviderId::GitHub => std::env::var("GITFLEET_GITHUB_TOKEN").ok(),
        ProviderId::GitLab => std::env::var("GITFLEET_GITLAB_TOKEN").ok(),
    };

    profile.token.or_else(|| {
        let default_host = match provider {
            ProviderId::GitHub => "github.com",
            ProviderId::GitLab => "gitlab.com",
        };

        (normalized_host.eq_ignore_ascii_case(default_host))
            .then(|| environment_token.filter(|token| !token.is_empty()))
            .flatten()
    })
}

pub fn get_github_token_optional() -> Option<String> {
    get_provider_token_optional(ProviderId::GitHub)
}

pub fn get_gitlab_token_optional() -> Option<String> {
    get_provider_token_optional(ProviderId::GitLab)
}

pub fn get_resolved_profile() -> Result<String, ConfigError> {
    let creds = read_credentials()?;

    resolve_profile_from(&creds)
}

fn resolve_profile_from(creds: &CredentialsFile) -> Result<String, ConfigError> {
    if let Ok(env_profile) = std::env::var(GITFLEET_PROFILE_ENV) {
        if creds.profiles.contains_key(&env_profile) {
            return Ok(env_profile);
        }

        return Err(ConfigError::new(crate::constants::ERROR_PROFILE_NOT_FOUND));
    }

    let trust_repo_config = std::env::var(GITFLEET_TRUST_REPO_CONFIG_ENV)
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if trust_repo_config {
        if let Some(repo_profile) = get_repo_local_profile() {
            if creds.profiles.contains_key(&repo_profile) {
                return Ok(repo_profile);
            }
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

fn update_credentials<F>(update: F) -> Result<(), ConfigError>
where
    F: FnOnce(&mut CredentialsFile) -> Result<(), ConfigError>,
{
    with_credentials_lock(true, || {
        let mut creds = read_credentials_unlocked()?;

        update(&mut creds)?;

        write_credentials_unlocked(&creds)
    })
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
    update_credentials(|creds| {
        let is_first = creds.profiles.is_empty();
        creds.profiles.insert(name.to_string(), profile);

        if is_first {
            creds.active_profile = name.to_string();
        }

        Ok(())
    })
}

pub fn set_active_profile(name: &str) -> Result<(), ConfigError> {
    update_credentials(|creds| {
        if !creds.profiles.contains_key(name) {
            return Err(ConfigError::new(crate::constants::ERROR_PROFILE_NOT_FOUND));
        }

        creds.active_profile = name.to_string();
        Ok(())
    })
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
    update_credentials(|creds| {
        let profile_name = resolve_profile_from(creds)?;
        let profile = creds.profiles.entry(profile_name).or_insert(Profile {
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

        Ok(())
    })
}

pub fn unset(key: &str) -> Result<(), ConfigError> {
    update_credentials(|creds| {
        let profile_name = resolve_profile_from(creds)?;
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

        Ok(())
    })
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
    resolve_provider_context()
        .map(|context| context.host)
        .unwrap_or_else(|_| "github.com".to_string())
}

pub fn remove_profile(name: &str) -> Result<(), ConfigError> {
    update_credentials(|creds| {
        if creds.profiles.remove(name).is_none() {
            return Err(ConfigError::new(format!("Profile \"{name}\" not found.")));
        }

        Ok(())
    })
}

pub fn clear_credentials() -> Result<(), ConfigError> {
    with_credentials_lock(true, || {
        let path = credentials_path()?;
        let creds = match read_credentials_unlocked() {
            Ok(creds) => creds,
            Err(error) if path.exists() => {
                tracing::warn!(%error, "Removing unreadable credentials metadata");
                std::fs::remove_file(&path).map_err(|remove_error| {
                    ConfigError::new(format!("Failed to remove credentials: {remove_error}"))
                })?;

                return Ok(());
            }
            Err(error) => return Err(error),
        };

        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| ConfigError::new(format!("Failed to remove credentials: {e}")))?;
        }

        if !uses_file_credential_store() {
            let empty = default_credentials();

            if let Err(error) = sync_keyring(&creds, &empty) {
                let keyring_rollback = sync_keyring(&empty, &creds);
                let file_rollback = write_credentials_file_unlocked(&creds);

                if keyring_rollback.is_err() || file_rollback.is_err() {
                    return Err(ConfigError::new(format!(
                        "{error} Credential rollback was incomplete; inspect the configured profiles before retrying."
                    )));
                }

                return Err(error);
            }
        }

        Ok(())
    })
}

pub fn normalize_host(value: &str) -> Result<String, ConfigError> {
    let value = value.trim();

    if value.is_empty() || value.chars().any(char::is_control) || value.contains("://") {
        return Err(ConfigError::new(
            "Host must be a hostname or host:port authority.",
        ));
    }

    let parsed = url::Url::parse(&format!("https://{value}"))
        .map_err(|e| ConfigError::new(format!("Invalid host '{value}': {e}")))?;

    if parsed.username() != ""
        || parsed.password().is_some()
        || parsed.path() != "/"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(ConfigError::new(
            "Host must not contain credentials, paths, queries, or fragments.",
        ));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| ConfigError::new("Host must contain a valid hostname."))?;
    let authority = match parsed.port() {
        Some(port) if host.contains(':') => format!("[{host}]:{port}"),
        Some(port) => format!("{host}:{port}"),
        None if host.contains(':') => format!("[{host}]"),
        None => host.to_string(),
    };

    Ok(authority.to_ascii_lowercase())
}

fn uses_file_credential_store() -> bool {
    cfg!(test)
        || std::env::var_os(GITFLEET_TEST_CREDENTIAL_STORE_ENV).is_some()
        || std::env::var(GITFLEET_CREDENTIAL_STORE_ENV)
            .map(|value| value.eq_ignore_ascii_case("file"))
            .unwrap_or(false)
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

fn delete_profile_token(profile: &str) -> Result<(), ConfigError> {
    match delete_token(profile) {
        Ok(()) => Ok(()),
        Err(error) if uses_file_credential_store() => {
            tracing::debug!(profile, %error, "Unable to remove stale keyring credential");
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn sync_keyring(previous: &CredentialsFile, desired: &CredentialsFile) -> Result<(), ConfigError> {
    for (name, profile) in &desired.profiles {
        let previous_token = previous
            .profiles
            .get(name)
            .and_then(|profile| profile.token.as_deref());
        let desired_token = profile.token.as_deref();

        if previous_token == desired_token {
            continue;
        }

        match desired_token {
            Some(token) => store_token(name, token)?,
            None => delete_profile_token(name)?,
        }
    }

    for name in previous.profiles.keys() {
        if !desired.profiles.contains_key(name) {
            delete_profile_token(name)?;
        }
    }

    Ok(())
}

fn keyring_entry(profile: &str) -> Result<keyring::Entry, ConfigError> {
    keyring::Entry::new(KEYRING_SERVICE, profile)
        .map_err(|e| ConfigError::new(format!("Failed to access the system credential store: {e}")))
}

fn load_token(profile: &str) -> Option<String> {
    keyring_entry(profile).ok()?.get_password().ok()
}

fn store_token(profile: &str, token: &str) -> Result<(), ConfigError> {
    keyring_entry(profile)?
        .set_password(token)
        .map_err(|e| ConfigError::new(format!("Failed to store profile token securely: {e}")))
}

fn delete_token(profile: &str) -> Result<(), ConfigError> {
    let entry = keyring_entry(profile)?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(error) => {
            let message = error.to_string().to_ascii_lowercase();
            if message.contains("no credential")
                || message.contains("no entry")
                || message.contains("not found")
            {
                Ok(())
            } else {
                Err(ConfigError::new(format!(
                    "Failed to remove profile token securely: {error}"
                )))
            }
        }
    }
}

fn default_host(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::GitHub => "github.com",
        ProviderId::GitLab => "gitlab.com",
    }
}

pub fn set_alias(name: &str, expansion: &str, force: bool) -> Result<(), ConfigError> {
    update_credentials(|creds| {
        if !force && creds.aliases.contains_key(name) {
            return Err(ConfigError::new(format!(
                "Alias '{name}' already exists. Use --force to overwrite."
            )));
        }

        creds
            .aliases
            .insert(name.to_string(), expansion.to_string());

        Ok(())
    })
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
    update_credentials(|creds| {
        if creds.aliases.remove(name).is_none() {
            return Err(ConfigError::new(format!("Alias '{name}' not found.")));
        }

        Ok(())
    })
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

        std::env::set_var("GITFLEET_HOME", dir.path().to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var("GITFLEET_PROFILE");

        let creds = read_credentials().unwrap();

        assert_eq!(creds.active_profile, DEFAULT_PROFILE_NAME);

        assert!(creds.profiles.is_empty());
        std::env::remove_var("GITFLEET_HOME");
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());

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
            std::env::set_var("GITFLEET_HOME", home);
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
    fn test_get_token_for_host_uses_matching_profile() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var(
            "GITFLEET_HOME",
            tmp_dir.path().to_string_lossy().to_string(),
        );
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var("GITFLEET_GITLAB_TOKEN");

        add_profile(
            "github-work",
            Profile {
                token: Some("github-token".to_string()),
                host: Some("github.example.com".to_string()),
                provider: Some("github".to_string()),
                extra: Default::default(),
            },
        )
        .unwrap();
        add_profile(
            "gitlab-work",
            Profile {
                token: Some("gitlab-token".to_string()),
                host: Some("gitlab.example.com".to_string()),
                provider: Some("gitlab".to_string()),
                extra: Default::default(),
            },
        )
        .unwrap();

        assert_eq!(
            get_token_for_host("gitlab.example.com").as_deref(),
            Some("gitlab-token")
        );

        std::env::set_var("GITFLEET_GITLAB_TOKEN", "environment-token");

        assert_eq!(
            get_token_for_host("gitlab.example.com").as_deref(),
            Some("gitlab-token")
        );

        std::env::remove_var("GITFLEET_GITLAB_TOKEN");
        assert_eq!(get_token_for_host("unknown.example.com"), None);

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
        } else {
            std::env::remove_var("GITFLEET_HOME");
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_provider_context_scopes_environment_token_to_public_host() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_provider_context");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let original_home = std::env::var("GITFLEET_HOME").ok();
        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var(GITFLEET_PROFILE_ENV);
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var("GITFLEET_GITLAB_TOKEN");

        add_profile(
            "gitlab-work",
            Profile {
                token: Some("profile-token".to_string()),
                host: Some("git.example.com:8443".to_string()),
                provider: Some("gitlab".to_string()),
                extra: Default::default(),
            },
        )
        .unwrap();
        set_active_profile("gitlab-work").unwrap();

        let profile_context = resolve_provider_context().unwrap();

        assert_eq!(profile_context.profile_name, "gitlab-work");
        assert_eq!(profile_context.provider, ProviderId::GitLab);
        assert_eq!(profile_context.host, "git.example.com:8443");
        assert_eq!(profile_context.token, Some("profile-token".to_string()));
        assert_eq!(profile_context.token_source, TokenSource::Profile);

        std::env::set_var("GITFLEET_GITLAB_TOKEN", "environment-token");

        let environment_context = resolve_provider_context().unwrap();

        assert_eq!(environment_context.token, Some("profile-token".to_string()));
        assert_eq!(environment_context.token_source, TokenSource::Profile);

        std::env::remove_var("GITFLEET_GITLAB_TOKEN");
        let _ = std::fs::remove_dir_all(&tmp_dir);

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
        }
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());

        let result = get_token();

        match original_home {
            Some(home) => std::env::set_var("GITFLEET_HOME", home),
            None => std::env::remove_var("GITFLEET_HOME"),
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let result = set_active_profile("nonexistent");

        assert!(result.is_err());

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_remove_nonexistent_profile() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_remove_err");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let result = remove_profile("nonexistent");

        assert!(result.is_err());

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_clear_credentials() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_clear");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_clear_credentials_recovers_from_malformed_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let original_home = std::env::var("GITFLEET_HOME").ok();
        let folder = dir.path().join(GITFLEET_FOLDER);
        let path = folder.join(CREDENTIALS_FILE);

        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(&path, "not valid = [toml").unwrap();
        std::env::set_var("GITFLEET_HOME", dir.path());

        clear_credentials().unwrap();

        assert!(!path.exists());

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
        } else {
            std::env::remove_var("GITFLEET_HOME");
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_get_host_default() {
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");

        let host = get_host();

        assert_eq!(host, "github.com");
    }

    #[test]
    fn test_normalize_host_rejects_url_components() {
        assert!(normalize_host("https://github.com").is_err());
        assert!(normalize_host("github.com/path").is_err());
        assert!(normalize_host("user:pass@github.com").is_err());
    }

    #[test]
    fn test_normalize_host_preserves_port_and_lowercases_authority() {
        assert_eq!(
            normalize_host("Git.Example.COM:8443").unwrap(),
            "git.example.com:8443"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_get_profile_existing() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_get_profile");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_get_profile_nonexistent() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_get_profile_err");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let found = get_profile("nonexistent").unwrap();

        assert!(found.is_none());

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_write_and_read_key() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_write_key");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_unset_key() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_unset");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        set_alias("ll", "repo list", false).unwrap();

        let expansion = get_alias("ll");

        assert_eq!(expansion, Some("repo list".to_string()));

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        set_alias("co", "checkout", false).unwrap();

        let result = set_alias("co", "checkout -b", false);

        assert!(result.is_err());

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        set_alias("co", "checkout", false).unwrap();
        set_alias("co", "checkout -b", true).unwrap();

        let expansion = get_alias("co");

        assert_eq!(expansion, Some("checkout -b".to_string()));

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_get_alias_nonexistent() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_alias_get");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let result = get_alias("nonexistent");

        assert_eq!(result, None);

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let entries = list_aliases().unwrap();

        assert!(entries.is_empty());

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        set_alias("temp", "repo list", false).unwrap();
        delete_alias("temp").unwrap();

        assert_eq!(get_alias("temp"), None);

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
        }

        let _ = std::fs::remove_dir_all(tmp_dir.join(GITFLEET_FOLDER));
    }

    #[test]
    #[serial_test::serial]
    fn test_delete_alias_nonexistent() {
        let tmp_dir = std::env::temp_dir().join("gitfleet_test_config_alias_delete_err");

        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
        std::env::remove_var("GITFLEET_GITHUB_TOKEN");
        std::env::remove_var(GITFLEET_PROFILE_ENV);

        let result = delete_alias("nonexistent");

        assert!(result.is_err());

        if let Some(home) = original_home {
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
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

        let original_home = std::env::var("GITFLEET_HOME").ok();

        std::env::set_var("GITFLEET_HOME", tmp_dir.to_string_lossy().to_string());
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
            std::env::set_var("GITFLEET_HOME", home);
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
