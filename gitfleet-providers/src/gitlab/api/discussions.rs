use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};
use gitfleet_core::types::Discussion;

use crate::gitlab::client::ProviderClient;

pub struct DiscussionsApi;

fn unsupported() -> GitfleetError {
    GitfleetError::from(UnsupportedCapabilityError::new(
        ProviderId::GitLab,
        ProviderCapability::Discussions,
    ))
}

impl DiscussionsApi {
    pub async fn list(
        _client: &ProviderClient,
        _owner: &str,
        _name: &str,
        _category_id: Option<&str>,
        _limit: u32,
        _page: Option<u32>,
    ) -> Result<Vec<Discussion>, GitfleetError> {
        Err(unsupported())
    }

    pub async fn get(
        _client: &ProviderClient,
        _owner: &str,
        _name: &str,
        _discussion_number: u64,
    ) -> Result<Discussion, GitfleetError> {
        Err(unsupported())
    }

    pub async fn create(
        _client: &ProviderClient,
        _owner: &str,
        _name: &str,
        _title: &str,
        _body: &str,
        _category_id: Option<&str>,
    ) -> Result<Discussion, GitfleetError> {
        Err(unsupported())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_discussions_are_explicitly_unsupported() {
        assert_eq!(
            gitfleet_core::provider::ProviderId::GitLab.to_string(),
            "gitlab"
        );
    }
}
