use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};
use gitfleet_core::types::{PublicKeyResponse, RepoSecret, SecretListResponse};

use crate::gitlab::client::ProviderClient;

pub struct SecretsApi;

impl SecretsApi {
    pub async fn list_repo(
        _client: &ProviderClient,
        _owner: &str,
        _repo: &str,
    ) -> Result<SecretListResponse<RepoSecret>, GitfleetError> {
        Err(unsupported())
    }

    pub async fn get_repo_public_key(
        _client: &ProviderClient,
        _owner: &str,
        _repo: &str,
    ) -> Result<PublicKeyResponse, GitfleetError> {
        Err(unsupported())
    }

    pub async fn set_repo(
        _client: &ProviderClient,
        _owner: &str,
        _repo: &str,
        _name: &str,
        _encrypted_value: &str,
        _key_id: &str,
    ) -> Result<(), GitfleetError> {
        Err(unsupported())
    }

    pub async fn delete_repo(
        _client: &ProviderClient,
        _owner: &str,
        _repo: &str,
        _name: &str,
    ) -> Result<(), GitfleetError> {
        Err(unsupported())
    }
}

fn unsupported() -> GitfleetError {
    GitfleetError::from(UnsupportedCapabilityError::new(
        ProviderId::GitLab,
        ProviderCapability::Secrets,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_secrets_are_explicitly_unsupported() {
        assert!(matches!(
            unsupported(),
            GitfleetError::UnsupportedCapability(_)
        ));
    }
}
