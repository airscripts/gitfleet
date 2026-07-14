use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{Milestone, MilestoneState};

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct MilestonesApi;

impl MilestonesApi {
    pub async fn list(
        client: &ProviderClient,
        project: &str,
        state: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Milestone>, GitfleetError> {
        let encoded = encode_path(project);

        let mut endpoint = format!("/projects/{encoded}/milestones?per_page={limit}");

        if let Some(s) = state {
            let s = if s == "open" { "active" } else { s };

            endpoint.push_str(&format!("&state={}", urlencoding::encode(s)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list milestones: {e}")))?;

        Ok(data.iter().map(normalize_milestone).collect())
    }

    pub async fn create(
        client: &ProviderClient,
        project: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<Milestone, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/milestones");

        let body = serde_json::json!({
            "title": title,
            "description": description.unwrap_or(""),
        });

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create milestone: {e}")))?;

        Ok(normalize_milestone(&data))
    }

    pub async fn get(
        client: &ProviderClient,
        project: &str,
        number: u64,
    ) -> Result<Milestone, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/milestones/{number}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get milestone: {e}")))?;

        Ok(normalize_milestone(&data))
    }

    pub async fn update(
        client: &ProviderClient,
        project: &str,
        number: u64,
        mut input: serde_json::Value,
    ) -> Result<Milestone, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/milestones/{number}");

        if let Some(object) = input.as_object_mut() {
            if let Some(due_date) = object.remove("due_on") {
                object.insert("due_date".to_string(), due_date);
            }

            if let Some(state) = object.remove("state") {
                let state_event = match state.as_str() {
                    Some("open") => serde_json::Value::String("activate".to_string()),
                    Some("closed") => serde_json::Value::String("close".to_string()),
                    _ => state,
                };

                object.insert("state_event".to_string(), state_event);
            }
        }

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(input), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update milestone: {e}")))?;

        Ok(normalize_milestone(&data))
    }

    pub async fn delete(
        client: &ProviderClient,
        project: &str,
        number: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/milestones/{number}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_milestone(raw: &serde_json::Value) -> Milestone {
    let state = raw
        .get("state")
        .and_then(|v| v.as_str())
        .unwrap_or("active");
    Milestone {
        id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
        url: raw
            .get("url")
            .or_else(|| raw.get("web_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        title: raw
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        number: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
        html_url: raw
            .get("web_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        open_issues: raw.get("open_issues").and_then(|v| v.as_u64()).unwrap_or(0),
        state: if state == "closed" {
            MilestoneState::Closed
        } else {
            MilestoneState::Open
        },
        due_on: raw
            .get("due_date")
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
            "id": 5,
            "title": "v1.0",
            "state": "active",
            "web_url": "https://gitlab.com/org/repo/-/milestones/5",
            "open_issues": 3,
            "closed_issues": 7,
            "due_date": "2024-12-31"
        });

        let result = normalize_milestone(&json);

        assert_eq!(result.id, 5);

        assert_eq!(result.title, "v1.0");
        assert!(matches!(result.state, MilestoneState::Open));

        assert_eq!(result.open_issues, 3);
        assert_eq!(result.closed_issues, 7);

        assert_eq!(result.due_on, Some("2024-12-31".to_string()));
    }

    #[test]
    fn test_normalize_milestone_closed() {
        let json = serde_json::json!({ "id": 2, "state": "closed", "title": "done" });

        let result = normalize_milestone(&json);

        assert!(matches!(result.state, MilestoneState::Closed));
    }

    #[test]
    fn test_normalize_milestone_minimal() {
        let json = serde_json::json!({});

        let result = normalize_milestone(&json);

        assert_eq!(result.id, 0);

        assert_eq!(result.title, "");
        assert!(matches!(result.state, MilestoneState::Open));
    }

    #[test]
    fn test_encode_path() {
        assert_eq!(encode_path("org/repo"), "org%2Frepo");
    }
}
