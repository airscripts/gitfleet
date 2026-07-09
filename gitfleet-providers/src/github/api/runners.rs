use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::RunnerSummary;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct RunnersApi;

impl RunnersApi {
    pub async fn list_repo(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<Vec<RunnerSummary>, GitfleetError> {
        let endpoint = format!("{}?per_page=100", repo_path(repo, &["actions", "runners"]));

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list runners: {e}")))?;

        let runners = data
            .get("runners")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(runners
            .iter()
            .map(|raw| RunnerSummary {
                id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                name: raw
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                os: raw
                    .get("os")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                status: raw
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                busy: raw.get("busy").and_then(|v| v.as_bool()).unwrap_or(false),
                labels: raw
                    .get("labels")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| {
                                v.get("name")
                                    .and_then(|n| n.as_str())
                                    .map(|s| s.to_string())
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            })
            .collect())
    }

    pub async fn remove_repo(
        client: &ProviderClient,
        repo: &str,
        runner_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["actions", "runners", &runner_id.to_string()]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::RunnerSummary;

    fn normalize_runner(raw: &serde_json::Value) -> RunnerSummary {
        RunnerSummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            name: raw
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            os: raw
                .get("os")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            status: raw
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            busy: raw.get("busy").and_then(|v| v.as_bool()).unwrap_or(false),
            labels: raw
                .get("labels")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| {
                            v.get("name")
                                .and_then(|n| n.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    #[test]
    fn test_normalize_runner_full() {
        let json = serde_json::json!({
            "id": 1,
            "name": "runner-1",
            "os": "linux",
            "status": "online",
            "busy": false,
            "labels": [
                { "name": "self-hosted", "type": "read-only" },
                { "name": "linux", "type": "read-only" }
            ]
        });

        let result = normalize_runner(&json);

        assert_eq!(result.id, 1);

        assert_eq!(result.name, "runner-1");
        assert_eq!(result.os, "linux");

        assert_eq!(result.status, "online");
        assert!(!result.busy);

        assert_eq!(result.labels, vec!["self-hosted", "linux"]);
    }

    #[test]
    fn test_normalize_runner_minimal() {
        let json = serde_json::json!({ "id": 5 });

        let result = normalize_runner(&json);

        assert_eq!(result.id, 5);

        assert_eq!(result.name, "");
        assert_eq!(result.os, "");

        assert_eq!(result.status, "");
        assert!(!result.busy);

        assert!(result.labels.is_empty());
    }
}
