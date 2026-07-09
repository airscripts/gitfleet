use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::CodespaceSummary;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct CodespacesApi;

impl CodespacesApi {
    pub async fn list(
        client: &ProviderClient,
        limit: u32,
    ) -> Result<Vec<CodespaceSummary>, GitfleetError> {
        let endpoint = format!("/user/codespaces?per_page={limit}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list codespaces: {e}")))?;

        let items = raw
            .get("codespaces")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(items.iter().map(normalize_codespace).collect())
    }

    pub async fn list_for_repo(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<Vec<CodespaceSummary>, GitfleetError> {
        let endpoint = repo_path(repo, &["codespaces"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list codespaces: {e}")))?;

        let items = raw
            .get("codespaces")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(items.iter().map(normalize_codespace).collect())
    }

    pub async fn create(
        client: &ProviderClient,
        repo: &str,
        r#ref: Option<&str>,
        machine: Option<&str>,
        idle_timeout_minutes: Option<u32>,
    ) -> Result<CodespaceSummary, GitfleetError> {
        let endpoint = repo_path(repo, &["codespaces"]);

        let mut payload = serde_json::json!({});

        if let Some(r) = r#ref {
            payload["ref"] = serde_json::Value::String(r.to_string());
        }

        if let Some(m) = machine {
            payload["machine"] = serde_json::Value::String(m.to_string());
        }

        if let Some(t) = idle_timeout_minutes {
            payload["idle_timeout_minutes"] = serde_json::Value::Number(t.into());
        }

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(payload), None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create codespace: {e}")))?;

        Ok(normalize_codespace(&raw))
    }

    pub async fn delete(client: &ProviderClient, codespace_id: &str) -> Result<(), GitfleetError> {
        let endpoint = format!("/user/codespaces/{codespace_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_codespace(raw: &serde_json::Value) -> CodespaceSummary {
    CodespaceSummary {
        id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
        name: raw
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        state: raw
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        owner: raw
            .get("owner")
            .and_then(|o| o.get("login"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        repo: raw
            .get("repository")
            .and_then(|r| r.get("full_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        branch: raw
            .get("branch")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        created_at: raw
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        idle_timeout_minutes: raw
            .get("idle_timeout_minutes")
            .and_then(|v| v.as_u64())
            .unwrap_or(30) as u32,
        machine: raw
            .get("machine")
            .and_then(|m| m.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_codespace_full() {
        let json = serde_json::json!({
            "id": 99,
            "name": "my-codespace",
            "state": "Available",
            "owner": { "login": "octocat" },
            "repository": { "full_name": "org/repo" },
            "branch": "main",
            "created_at": "2024-01-01T00:00:00Z",
            "idle_timeout_minutes": 60,
            "machine": { "name": "standardLinux" }
        });

        let result = normalize_codespace(&json);

        assert_eq!(result.id, 99);

        assert_eq!(result.name, "my-codespace");
        assert_eq!(result.state, "Available");

        assert_eq!(result.owner, "octocat");
        assert_eq!(result.repo, "org/repo");

        assert_eq!(result.branch, "main");
        assert_eq!(result.idle_timeout_minutes, 60);

        assert_eq!(result.machine, "standardLinux");
    }

    #[test]
    fn test_normalize_codespace_minimal() {
        let json = serde_json::json!({});

        let result = normalize_codespace(&json);

        assert_eq!(result.id, 0);

        assert_eq!(result.name, "");
        assert_eq!(result.state, "");

        assert_eq!(result.idle_timeout_minutes, 30);
    }
}
