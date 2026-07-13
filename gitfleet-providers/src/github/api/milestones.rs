use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{Milestone, MilestoneState};

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct MilestonesApi;

impl MilestonesApi {
    pub async fn list(
        client: &ProviderClient,
        repo: &str,
        state: &str,
        limit: u32,
    ) -> Result<Vec<Milestone>, GitfleetError> {
        let endpoint = format!(
            "{}?state={state}&per_page={limit}",
            repo_path(repo, &["milestones"])
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list milestones: {e}")))?;

        Ok(raw.iter().map(normalize_milestone).collect())
    }

    pub async fn create(
        client: &ProviderClient,
        repo: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<Milestone, GitfleetError> {
        let endpoint = repo_path(repo, &["milestones"]);

        let mut payload = serde_json::json!({ "title": title });

        if let Some(desc) = description {
            payload["description"] = serde_json::Value::String(desc.to_string());
        }

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(payload), None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create milestone: {e}")))?;

        Ok(normalize_milestone(&raw))
    }

    pub async fn get(
        client: &ProviderClient,
        repo: &str,
        number: u64,
    ) -> Result<Milestone, GitfleetError> {
        let endpoint = repo_path(repo, &["milestones", &number.to_string()]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get milestone: {e}")))?;

        Ok(normalize_milestone(&raw))
    }

    pub async fn update(
        client: &ProviderClient,
        repo: &str,
        number: u64,
        options: serde_json::Value,
    ) -> Result<Milestone, GitfleetError> {
        let endpoint = repo_path(repo, &["milestones", &number.to_string()]);

        let response = client
            .request_token_required(reqwest::Method::PATCH, &endpoint, Some(options), None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update milestone: {e}")))?;

        Ok(normalize_milestone(&raw))
    }

    pub async fn delete(
        client: &ProviderClient,
        repo: &str,
        number: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["milestones", &number.to_string()]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_milestone(raw: &serde_json::Value) -> Milestone {
    let state_str = raw.get("state").and_then(|v| v.as_str()).unwrap_or("open");
    Milestone {
        id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
        url: raw
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        title: raw
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        number: raw.get("number").and_then(|v| v.as_u64()).unwrap_or(0),
        html_url: raw
            .get("html_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        open_issues: raw.get("open_issues").and_then(|v| v.as_u64()).unwrap_or(0),
        state: if state_str == "closed" {
            MilestoneState::Closed
        } else {
            MilestoneState::Open
        },
        due_on: raw
            .get("due_on")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        closed_issues: raw
            .get("closed_issues")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_milestone_full() {
        let json = serde_json::json!({
            "id": 1,
            "url": "https://api.github.com/repos/o/r/milestones/1",
            "title": "v1.0",
            "number": 1,
            "html_url": "https://github.com/o/r/milestone/1",
            "open_issues": 5,
            "state": "open",
            "due_on": "2024-12-31T00:00:00Z",
            "closed_issues": 2
        });

        let result = normalize_milestone(&json);

        assert_eq!(result.id, 1);

        assert_eq!(result.title, "v1.0");
        assert_eq!(result.number, 1);

        assert_eq!(result.open_issues, 5);
        assert!(matches!(result.state, MilestoneState::Open));

        assert_eq!(result.due_on, Some("2024-12-31T00:00:00Z".to_string()));
        assert_eq!(result.closed_issues, 2);
    }

    #[test]
    fn test_normalize_milestone_closed() {
        let json = serde_json::json!({
            "id": 2,
            "url": "",
            "title": "v0.9",
            "number": 2,
            "html_url": "",
            "open_issues": 0,
            "state": "closed",
            "closed_issues": 10
        });

        let result = normalize_milestone(&json);

        assert!(matches!(result.state, MilestoneState::Closed));

        assert!(result.due_on.is_none());
    }

    #[test]
    fn test_normalize_milestone_minimal() {
        let json = serde_json::json!({});

        let result = normalize_milestone(&json);

        assert_eq!(result.id, 0);

        assert_eq!(result.title, "");
        assert!(matches!(result.state, MilestoneState::Open));
    }
}
