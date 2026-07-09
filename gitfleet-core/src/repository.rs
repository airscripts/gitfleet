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
}
