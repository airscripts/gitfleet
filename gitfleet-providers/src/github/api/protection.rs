use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::TagProtection;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct ProtectionApi;

impl ProtectionApi {
    pub async fn get_branch_protection(
        client: &ProviderClient,
        repo: &str,
        branch: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!(
            "/repos/{repo}/branches/{}/protection",
            urlencoding::encode(branch)
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get branch protection: {e}")))?;

        Ok(data)
    }

    pub async fn protect_branch(
        client: &ProviderClient,
        repo: &str,
        branch: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!(
            "/repos/{repo}/branches/{}/protection",
            urlencoding::encode(branch)
        );

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(input), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to set branch protection: {e}")))?;

        Ok(data)
    }

    pub async fn unprotect_branch(
        client: &ProviderClient,
        repo: &str,
        branch: &str,
    ) -> Result<(), GitfleetError> {
        let endpoint = format!(
            "/repos/{repo}/branches/{}/protection",
            urlencoding::encode(branch)
        );

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn list_tag_protection(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<Vec<TagProtection>, GitfleetError> {
        let endpoint = repo_path(repo, &["tags-protection"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list tag protection: {e}")))?;

        Ok(raw
            .iter()
            .map(|t| TagProtection {
                id: t.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                pattern: t
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                created_at: t
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }

    pub async fn create_tag_protection(
        client: &ProviderClient,
        repo: &str,
        pattern: &str,
    ) -> Result<TagProtection, GitfleetError> {
        let endpoint = repo_path(repo, &["tags-protection"]);

        let payload = serde_json::json!({ "pattern": pattern });
        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(payload), None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create tag protection: {e}")))?;

        Ok(TagProtection {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            pattern: raw
                .get("pattern")
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

    pub async fn delete_tag_protection(
        client: &ProviderClient,
        repo: &str,
        id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["tags-protection", &id.to_string()]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::TagProtection;

    fn normalize_tag_protection(raw: &serde_json::Value) -> TagProtection {
        TagProtection {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            pattern: raw
                .get("pattern")
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
    fn test_normalize_tag_protection_full() {
        let json = serde_json::json!({
            "id": 42,
            "pattern": "v*",
            "created_at": "2024-01-01T00:00:00Z"
        });

        let result = normalize_tag_protection(&json);

        assert_eq!(result.id, 42);

        assert_eq!(result.pattern, "v*");
        assert_eq!(result.created_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_normalize_tag_protection_minimal() {
        let json = serde_json::json!({});

        let result = normalize_tag_protection(&json);

        assert_eq!(result.id, 0);

        assert_eq!(result.pattern, "");
        assert_eq!(result.created_at, "");
    }
}
