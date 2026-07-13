use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{PublicKeyResponse, RepoSecret, SecretListResponse};

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct SecretsApi;

impl SecretsApi {
    pub async fn list_repo(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
    ) -> Result<SecretListResponse<RepoSecret>, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let endpoint = repo_path(&full, &["actions", "secrets"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: SecretListResponse<RepoSecret> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list repo secrets: {e}")))?;

        Ok(data)
    }

    pub async fn get_repo_public_key(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
    ) -> Result<PublicKeyResponse, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let endpoint = repo_path(&full, &["actions", "secrets", "public-key"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: PublicKeyResponse = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get public key: {e}")))?;

        Ok(data)
    }

    pub async fn set_repo(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
        encrypted_value: &str,
        key_id: &str,
    ) -> Result<(), GitfleetError> {
        let full = format!("{owner}/{repo}");

        let endpoint = repo_path(&full, &["actions", "secrets", name]);

        let body = serde_json::json!({
            "encrypted_value": encrypted_value,
            "key_id": key_id,
        });

        client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn delete_repo(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), GitfleetError> {
        let full = format!("{owner}/{repo}");

        let endpoint = repo_path(&full, &["actions", "secrets", name]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secrets_set_body() {
        let encrypted_value = "abc123";
        let key_id = "key1";
        let body = serde_json::json!({
            "encrypted_value": encrypted_value,
            "key_id": key_id,
        });

        assert_eq!(body["encrypted_value"], "abc123");

        assert_eq!(body["key_id"], "key1");
    }

    #[test]
    fn test_secrets_repo_path() {
        let full = "owner/repo";
        let endpoint = repo_path(full, &["actions", "secrets"]);

        assert_eq!(endpoint, "/repos/owner/repo/actions/secrets");
    }
}
