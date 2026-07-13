use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::TagProtection;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct PolicyApi;

impl PolicyApi {
    pub async fn get_branch_protection(
        client: &ProviderClient,
        project: &str,
        branch: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let enc_branch = urlencoding::encode(branch);
        let endpoint = format!("/projects/{encoded}/protected_branches/{enc_branch}");

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
        project: &str,
        branch: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/protected_branches");
        let mut body = input;

        if let Some(obj) = body.as_object_mut() {
            obj.insert(
                "name".to_string(),
                serde_json::Value::String(branch.to_string()),
            );
        } else {
            body = serde_json::json!({ "name": branch });
        }

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to protect branch: {e}")))?;

        Ok(data)
    }

    pub async fn unprotect_branch(
        client: &ProviderClient,
        project: &str,
        branch: &str,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let enc_branch = urlencoding::encode(branch);
        let endpoint = format!("/projects/{encoded}/protected_branches/{enc_branch}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn list_tag_protection(
        client: &ProviderClient,
        project: &str,
    ) -> Result<Vec<TagProtection>, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/protected_tags");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list tag protection: {e}")))?;

        Ok(data.iter().map(normalize_tag_protection).collect())
    }

    pub async fn create_tag_protection(
        client: &ProviderClient,
        project: &str,
        pattern: &str,
    ) -> Result<TagProtection, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/protected_tags");

        let body = serde_json::json!({ "name": pattern });

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create tag protection: {e}")))?;

        Ok(normalize_tag_protection(&data))
    }

    pub async fn delete_tag_protection(
        client: &ProviderClient,
        project: &str,
        tag_name: &str,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let enc_tag = urlencoding::encode(tag_name);
        let endpoint = format!("/projects/{encoded}/protected_tags/{enc_tag}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_tag_protection(raw: &serde_json::Value) -> TagProtection {
    TagProtection {
        id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
        pattern: raw
            .get("name")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_tag_protection_full() {
        let json = serde_json::json!({
            "id": 10,
            "name": "v*",
            "created_at": "2024-01-01T00:00:00Z"
        });

        let result = normalize_tag_protection(&json);

        assert_eq!(result.id, 10);

        assert_eq!(result.pattern, "v*");
        assert_eq!(result.created_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_normalize_tag_protection_minimal() {
        let json = serde_json::json!({});

        let result = normalize_tag_protection(&json);

        assert_eq!(result.id, 0);

        assert_eq!(result.pattern, "");
    }

    #[test]
    fn test_encode_path() {
        assert_eq!(encode_path("org/repo"), "org%2Frepo");
    }
}
