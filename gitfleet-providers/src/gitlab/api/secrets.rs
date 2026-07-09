use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{PublicKeyResponse, RepoSecret, SecretListResponse};

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct SecretsApi;

impl SecretsApi {
    pub async fn list_repo(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
    ) -> Result<SecretListResponse<RepoSecret>, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let endpoint = format!("/projects/{encoded}/variables");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list secrets: {e}")))?;

        let secrets: Vec<RepoSecret> = data
            .iter()
            .map(|raw| RepoSecret {
                name: raw
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                created_at: raw
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                updated_at: raw
                    .get("updated_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect();

        Ok(SecretListResponse {
            total_count: secrets.len() as u64,
            secrets,
        })
    }

    pub async fn get_repo_public_key(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
    ) -> Result<PublicKeyResponse, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let endpoint = format!("/projects/{encoded}/variables");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let _data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get public key: {e}")))?;

        Ok(PublicKeyResponse {
            key_id: String::new(),
            key: String::new(),
        })
    }

    pub async fn set_repo(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
        _encrypted_value: &str,
        _key_id: &str,
    ) -> Result<(), GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let endpoint = format!("/projects/{encoded}/variables");

        let body = serde_json::json!({
            "key": name,
            "value": _encrypted_value,
            "masked": true,
        });

        client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
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

        let encoded = encode_path(&full);
        let enc_name = urlencoding::encode(name);

        let endpoint = format!("/projects/{encoded}/variables/{enc_name}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_secrets_encode_path() {
        let full = "owner/repo";
        let encoded = urlencoding::encode(full).to_string();

        assert_eq!(encoded, "owner%2Frepo");
    }
}
