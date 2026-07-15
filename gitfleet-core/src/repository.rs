use serde::{Deserialize, Serialize};

use crate::constants::HOST_PROVIDERS;
use crate::errors::ConfigError;
use crate::provider::ProviderId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRef {
    pub provider: ProviderId,
    pub host: String,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRef {
    pub provider: ProviderId,
    pub host: String,
    pub namespace: String,
    pub name: String,
}

impl RepositoryRef {
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.namespace, self.name)
    }

    pub fn qualified(&self) -> String {
        format!("{}@{}:{}", self.provider, self.host, self.full_name())
    }
}

pub fn repository_ref_from_remote(remote_url: &str) -> Result<RepositoryRef, ConfigError> {
    let (host, path) = parse_remote_url(remote_url)?;

    let provider = HOST_PROVIDERS
        .iter()
        .find(|(h, _)| h.eq_ignore_ascii_case(host.as_str()))
        .map(|(_, p)| *p)
        .or_else(|| {
            let profile_name = crate::config::find_profile_by_host(&host).ok().flatten()?;
            let profile = crate::config::get_profile(&profile_name).ok().flatten()?;

            Some(match profile.provider.as_deref() {
                Some("gitlab") => ProviderId::GitLab,
                _ => ProviderId::GitHub,
            })
        })
        .or_else(|| {
            let context = crate::config::resolve_provider_context().ok()?;

            context
                .host
                .eq_ignore_ascii_case(&host)
                .then_some(context.provider)
        })
        .ok_or_else(|| ConfigError::new(format!("Unsupported git provider host: {host}")))?;

    let path = path.trim_start_matches('/');

    let path = path.strip_suffix(".git").unwrap_or(path);
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if segments.len() < 2 {
        return Err(ConfigError::new(format!(
            "Invalid repository remote: {remote_url}"
        )));
    }

    let name = segments
        .last()
        .expect("at least two segments guaranteed by length check")
        .to_string();

    let namespace = segments[..segments.len() - 1].join("/");

    if namespace.is_empty() || name.is_empty() {
        return Err(ConfigError::new(format!(
            "Invalid repository remote: {remote_url}"
        )));
    }

    Ok(RepositoryRef {
        provider,
        host,
        namespace,
        name,
    })
}

fn parse_remote_url(remote_url: &str) -> Result<(String, String), ConfigError> {
    if let Some(captures) = regex_captures_scp(remote_url) {
        return Ok(captures);
    }

    let url = url::Url::parse(remote_url)
        .or_else(|_| url::Url::parse(&format!("https://{remote_url}")))
        .map_err(|_| ConfigError::new(format!("Invalid git remote URL: {remote_url}")))?;

    let host = url
        .host_str()
        .ok_or_else(|| ConfigError::new(format!("Invalid git remote URL: {remote_url}")))?
        .to_string();

    let path = url.path().to_string();

    Ok((host, path))
}

fn regex_captures_scp(remote_url: &str) -> Option<(String, String)> {
    let re = regex::Regex::new(r"^([^@\s]+)@([^:/\s]+):(.+)$").ok()?;

    let caps = re.captures(remote_url)?;
    let user = caps.get(1)?.as_str().to_string();

    let host = caps.get(2)?.as_str().to_string();
    let path = caps.get(3)?.as_str().to_string();
    drop(user);
    Some((host, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_https_remote() {
        let result = repository_ref_from_remote("https://github.com/airscripts/gitfleet.git");

        assert!(result.is_ok());

        let repo = result.unwrap();

        assert_eq!(repo.provider, ProviderId::GitHub);

        assert_eq!(repo.host, "github.com");
        assert_eq!(repo.namespace, "airscripts");

        assert_eq!(repo.name, "gitfleet");
    }

    #[test]
    fn test_parse_scp_remote() {
        let result = repository_ref_from_remote("git@github.com:airscripts/gitfleet.git");

        assert!(result.is_ok());

        let repo = result.unwrap();

        assert_eq!(repo.provider, ProviderId::GitHub);

        assert_eq!(repo.host, "github.com");
        assert_eq!(repo.namespace, "airscripts");

        assert_eq!(repo.name, "gitfleet");
    }

    #[test]
    fn test_parse_without_git_suffix() {
        let result = repository_ref_from_remote("https://github.com/airscripts/gitfleet");

        assert!(result.is_ok());

        let repo = result.unwrap();

        assert_eq!(repo.name, "gitfleet");
    }

    #[test]
    fn test_invalid_remote() {
        let result = repository_ref_from_remote("invalid");

        assert!(result.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test_parse_configured_enterprise_remote() {
        let dir = tempfile::tempdir().unwrap();
        let original_home = std::env::var("GITFLEET_HOME").ok();

        // SAFETY: This test serializes process-environment mutation with `serial_test`.
        unsafe { std::env::set_var("GITFLEET_HOME", dir.path().to_string_lossy().to_string()) };

        crate::config::add_profile(
            "enterprise",
            crate::types::Profile {
                token: Some("token".to_string()),
                host: Some("github.example.com".to_string()),
                provider: Some("github".to_string()),
                extra: Default::default(),
            },
        )
        .unwrap();

        let result =
            repository_ref_from_remote("https://github.example.com/enterprise/project.git")
                .unwrap();

        assert_eq!(result.provider, ProviderId::GitHub);
        assert_eq!(result.host, "github.example.com");
        assert_eq!(result.full_name(), "enterprise/project");

        if let Some(home) = original_home {
            // SAFETY: This test serializes process-environment mutation with `serial_test`.
            unsafe { std::env::set_var("GITFLEET_HOME", home) };
        } else {
            // SAFETY: This test serializes process-environment mutation with `serial_test`.
            unsafe { std::env::remove_var("GITFLEET_HOME") };
        }
    }
}
