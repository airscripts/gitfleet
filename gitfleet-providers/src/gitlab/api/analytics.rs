use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};

use crate::gitlab::client::ProviderClient;

pub struct AnalyticsApi;

impl AnalyticsApi {
    pub async fn get_traffic_views(
        _client: &ProviderClient,
        _project: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(unsupported())
    }

    pub async fn get_traffic_clones(
        _client: &ProviderClient,
        _project: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(unsupported())
    }
}

fn unsupported() -> GitfleetError {
    GitfleetError::from(UnsupportedCapabilityError::new(
        ProviderId::GitLab,
        ProviderCapability::Analytics,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_analytics_are_explicitly_unsupported() {
        assert!(matches!(
            unsupported(),
            GitfleetError::UnsupportedCapability(_)
        ));
    }
}
