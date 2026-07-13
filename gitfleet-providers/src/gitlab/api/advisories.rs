use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};

use crate::gitlab::client::ProviderClient;

pub struct AdvisoriesApi;

fn unsupported() -> GitfleetError {
    GitfleetError::from(UnsupportedCapabilityError::new(
        ProviderId::GitLab,
        ProviderCapability::Advisories,
    ))
}

impl AdvisoriesApi {
    pub async fn list_vulnerabilities(
        _client: &ProviderClient,
        _project: &str,
        _state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(unsupported())
    }

    pub async fn list_dependabot_alerts(
        _client: &ProviderClient,
        _project: &str,
        _state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(unsupported())
    }

    pub async fn list_codeql_alerts(
        _client: &ProviderClient,
        _project: &str,
        _state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(unsupported())
    }

    pub async fn list_secret_scanning_alerts(
        _client: &ProviderClient,
        _project: &str,
        _state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(unsupported())
    }

    pub async fn get_alert(
        _client: &ProviderClient,
        _project: &str,
        _number: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(unsupported())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_advisories_are_explicitly_unsupported() {
        assert_eq!(
            gitfleet_core::provider::ProviderId::GitLab.to_string(),
            "gitlab"
        );
    }
}
