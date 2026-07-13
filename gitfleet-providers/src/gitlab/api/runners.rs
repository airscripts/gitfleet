use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::RunnerSummary;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct RunnersApi;

impl RunnersApi {
    pub async fn list(
        client: &ProviderClient,
        project: &str,
    ) -> Result<Vec<RunnerSummary>, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/runners?per_page=100");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list runners: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| RunnerSummary {
                id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                name: raw
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                os: raw
                    .get("platform")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                status: raw
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                busy: raw
                    .get("is_active")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                labels: raw
                    .get("tag_list")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
            })
            .collect())
    }

    pub async fn remove(
        client: &ProviderClient,
        project: &str,
        runner_id: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/runners/{runner_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalize_gitlab_runner(raw: &serde_json::Value) -> RunnerSummary {
        RunnerSummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            name: raw
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            os: raw
                .get("platform")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            status: raw
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            busy: raw
                .get("is_active")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            labels: raw
                .get("tag_list")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    #[test]
    fn test_normalize_gitlab_runner_full() {
        let json = serde_json::json!({
            "id": 10,
            "description": "runner-1",
            "platform": "linux",
            "status": "online",
            "is_active": true,
            "tag_list": ["docker", "shell"]
        });

        let result = normalize_gitlab_runner(&json);

        assert_eq!(result.id, 10);

        assert_eq!(result.name, "runner-1");
        assert_eq!(result.os, "linux");

        assert_eq!(result.status, "online");
        assert!(result.busy);

        assert_eq!(result.labels, vec!["docker", "shell"]);
    }

    #[test]
    fn test_normalize_gitlab_runner_minimal() {
        let json = serde_json::json!({ "id": 3 });

        let result = normalize_gitlab_runner(&json);

        assert_eq!(result.id, 3);

        assert_eq!(result.name, "");
        assert_eq!(result.os, "");

        assert!(!result.busy);
        assert!(result.labels.is_empty());
    }
}
