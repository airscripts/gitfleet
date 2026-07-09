use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::ReactionSummary;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct ReviewApi;

impl ReviewApi {
    pub async fn list(
        client: &ProviderClient,
        project: &str,
        issue_iid: u64,
    ) -> Result<Vec<ReactionSummary>, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/issues/{issue_iid}/award_emoji");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list award emojis: {e}")))?;

        Ok(data.iter().map(normalize_reaction).collect())
    }

    pub async fn create(
        client: &ProviderClient,
        project: &str,
        issue_iid: u64,
        name: &str,
    ) -> Result<ReactionSummary, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/issues/{issue_iid}/award_emoji");

        let body = serde_json::json!({ "name": name });

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create award emoji: {e}")))?;

        Ok(normalize_reaction(&data))
    }

    pub async fn delete(
        client: &ProviderClient,
        project: &str,
        issue_iid: u64,
        award_id: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/issues/{issue_iid}/award_emoji/{award_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_reaction(raw: &serde_json::Value) -> ReactionSummary {
    ReactionSummary {
        id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
        content: raw
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        user: raw
            .get("user")
            .and_then(|u| u.get("username"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
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
    fn test_normalize_reaction_full() {
        let json = serde_json::json!({
            "id": 42,
            "name": "thumbsup",
            "user": { "username": "alice" },
            "created_at": "2024-01-01T00:00:00Z"
        });

        let result = normalize_reaction(&json);

        assert_eq!(result.id, 42);

        assert_eq!(result.content, "thumbsup");
        assert_eq!(result.user, Some("alice".to_string()));

        assert_eq!(result.created_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_normalize_reaction_minimal() {
        let json = serde_json::json!({});

        let result = normalize_reaction(&json);

        assert_eq!(result.id, 0);

        assert_eq!(result.content, "");
        assert!(result.user.is_none());

        assert_eq!(result.created_at, "");
    }

    #[test]
    fn test_encode_path() {
        assert_eq!(encode_path("org/repo"), "org%2Frepo");
    }
}
