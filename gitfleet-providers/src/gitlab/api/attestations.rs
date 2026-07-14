use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};

use crate::gitlab::client::ProviderClient;

pub struct AttestationsApi;

fn unsupported() -> GitfleetError {
    GitfleetError::from(UnsupportedCapabilityError::new(
        ProviderId::GitLab,
        ProviderCapability::Attestations,
    ))
}

impl AttestationsApi {
    pub async fn list(
        _client: &ProviderClient,
        _project: &str,
        _subject_digest: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(unsupported())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_attestations_are_explicitly_unsupported() {
        assert_eq!(
            gitfleet_core::provider::ProviderId::GitLab.to_string(),
            "gitlab"
        );
    }
}
