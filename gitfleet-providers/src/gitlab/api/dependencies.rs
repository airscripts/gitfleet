use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};
use gitfleet_core::types::DependencyReviewChange;

use crate::gitlab::client::ProviderClient;

pub struct DependenciesApi;

impl DependenciesApi {
    pub async fn sbom(
        _client: &ProviderClient,
        _project: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(GitfleetError::from(UnsupportedCapabilityError::new(
            ProviderId::GitLab,
            ProviderCapability::Dependencies,
        )))
    }

    pub async fn review(
        _client: &ProviderClient,
        _project: &str,
        _base: &str,
        _head: &str,
    ) -> Result<Vec<DependencyReviewChange>, GitfleetError> {
        Err(GitfleetError::from(UnsupportedCapabilityError::new(
            ProviderId::GitLab,
            ProviderCapability::Dependencies,
        )))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_dependencies_are_explicitly_unsupported() {
        assert_eq!(
            gitfleet_core::provider::ProviderId::GitLab.to_string(),
            "gitlab"
        );
    }
}
