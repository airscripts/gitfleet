use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};

use crate::gitlab::client::ProviderClient;

pub struct GovernanceApi;

impl GovernanceApi {
    pub async fn list_rulesets(
        _client: &ProviderClient,
        _project: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(unsupported())
    }

    pub async fn create_ruleset(
        _client: &ProviderClient,
        _project: &str,
        _input: &gitfleet_core::types::RulesetInput,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(unsupported())
    }

    pub async fn delete_ruleset(
        _client: &ProviderClient,
        _project: &str,
        _ruleset_id: u64,
    ) -> Result<(), GitfleetError> {
        Err(unsupported())
    }
}

fn unsupported() -> GitfleetError {
    GitfleetError::from(UnsupportedCapabilityError::new(
        ProviderId::GitLab,
        ProviderCapability::Governance,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_governance_is_explicitly_unsupported() {
        assert!(matches!(
            unsupported(),
            GitfleetError::UnsupportedCapability(_)
        ));
    }
}
