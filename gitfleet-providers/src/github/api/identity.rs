use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{GpgKeySummary, SshKeySummary};

use crate::github::client::ProviderClient;

pub struct IdentityApi;

impl IdentityApi {
    pub async fn list_ssh_keys(
        client: &ProviderClient,
    ) -> Result<Vec<SshKeySummary>, GitfleetError> {
        let endpoint = "/user/keys?per_page=100";

        let response = client
            .request_token_required(reqwest::Method::GET, endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list SSH keys: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| SshKeySummary {
                id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                title: raw
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                key: raw
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                created_at: raw
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }

    pub async fn add_ssh_key(
        client: &ProviderClient,
        title: &str,
        key: &str,
    ) -> Result<SshKeySummary, GitfleetError> {
        let endpoint = "/user/keys";

        let body = serde_json::json!({ "title": title, "key": key });

        let response = client
            .request_token_required(reqwest::Method::POST, endpoint, Some(body), None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to add SSH key: {e}")))?;

        Ok(SshKeySummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            title: raw
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            key: raw
                .get("key")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            created_at: raw
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    pub async fn delete_ssh_key(client: &ProviderClient, key_id: u64) -> Result<(), GitfleetError> {
        let endpoint = format!("/user/keys/{key_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn list_gpg_keys(
        client: &ProviderClient,
    ) -> Result<Vec<GpgKeySummary>, GitfleetError> {
        let endpoint = "/user/gpg_keys?per_page=100";

        let response = client
            .request_token_required(reqwest::Method::GET, endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list GPG keys: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| GpgKeySummary {
                id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                name: raw
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                key_id: raw
                    .get("key_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                created_at: raw
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }

    pub async fn add_gpg_key(
        client: &ProviderClient,
        armored_key: &str,
    ) -> Result<GpgKeySummary, GitfleetError> {
        let endpoint = "/user/gpg_keys";

        let body = serde_json::json!({ "armored_public_key": armored_key });

        let response = client
            .request_token_required(reqwest::Method::POST, endpoint, Some(body), None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to add GPG key: {e}")))?;

        Ok(GpgKeySummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            name: raw
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            key_id: raw
                .get("key_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            created_at: raw
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    pub async fn delete_gpg_key(client: &ProviderClient, key_id: u64) -> Result<(), GitfleetError> {
        let endpoint = format!("/user/gpg_keys/{key_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalize_ssh_key(raw: &serde_json::Value) -> SshKeySummary {
        SshKeySummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            title: raw
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            key: raw
                .get("key")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            created_at: raw
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    fn normalize_gpg_key(raw: &serde_json::Value) -> GpgKeySummary {
        GpgKeySummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            name: raw
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            key_id: raw
                .get("key_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            created_at: raw
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    #[test]
    fn test_normalize_ssh_key_full() {
        let json = serde_json::json!({
            "id": 42,
            "title": "My Key",
            "key": "ssh-rsa AAAAB3...",
            "created_at": "2024-01-01T00:00:00Z"
        });

        let result = normalize_ssh_key(&json);

        assert_eq!(result.id, 42);

        assert_eq!(result.title, "My Key");
        assert_eq!(result.key, "ssh-rsa AAAAB3...");

        assert_eq!(result.created_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_normalize_ssh_key_minimal() {
        let json = serde_json::json!({ "id": 1 });

        let result = normalize_ssh_key(&json);

        assert_eq!(result.id, 1);

        assert_eq!(result.title, "");
        assert_eq!(result.key, "");

        assert_eq!(result.created_at, "");
    }

    #[test]
    fn test_normalize_gpg_key_full() {
        let json = serde_json::json!({
            "id": 7,
            "name": "My GPG Key",
            "key_id": "ABC123",
            "created_at": "2024-06-01T00:00:00Z"
        });

        let result = normalize_gpg_key(&json);

        assert_eq!(result.id, 7);

        assert_eq!(result.name, "My GPG Key");
        assert_eq!(result.key_id, "ABC123");

        assert_eq!(result.created_at, "2024-06-01T00:00:00Z");
    }

    #[test]
    fn test_normalize_gpg_key_minimal() {
        let json = serde_json::json!({});

        let result = normalize_gpg_key(&json);

        assert_eq!(result.id, 0);

        assert_eq!(result.name, "");
        assert_eq!(result.key_id, "");

        assert_eq!(result.created_at, "");
    }
}
