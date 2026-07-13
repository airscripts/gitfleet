use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::DeploymentSummary;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct DeploymentsApi;

impl DeploymentsApi {
    pub async fn list(
        client: &ProviderClient,
        repo: &str,
        environment: Option<&str>,
        limit: u32,
    ) -> Result<Vec<DeploymentSummary>, GitfleetError> {
        let mut endpoint = format!("{}?per_page={limit}", repo_path(repo, &["deployments"]));

        if let Some(env) = environment {
            endpoint.push_str(&format!("&environment={env}"));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list deployments: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| DeploymentSummary {
                id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                r#ref: raw
                    .get("ref")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                environment: raw
                    .get("environment")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                task: raw
                    .get("task")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                description: raw
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                creator: raw
                    .get("creator")
                    .and_then(|v| v.get("login"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                created_at: raw
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                production: raw
                    .get("production_environment")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            })
            .collect())
    }

    pub async fn create(
        client: &ProviderClient,
        repo: &str,
        input: serde_json::Value,
    ) -> Result<DeploymentSummary, GitfleetError> {
        let endpoint = repo_path(repo, &["deployments"]);

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(input), None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create deployment: {e}")))?;

        Ok(DeploymentSummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            r#ref: raw
                .get("ref")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            environment: raw
                .get("environment")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            task: raw
                .get("task")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: raw
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            creator: raw
                .get("creator")
                .and_then(|v| v.get("login"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at: raw
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            production: raw
                .get("production_environment")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalize_deployment(raw: &serde_json::Value) -> DeploymentSummary {
        DeploymentSummary {
            id: raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            r#ref: raw
                .get("ref")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            environment: raw
                .get("environment")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            task: raw
                .get("task")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: raw
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            creator: raw
                .get("creator")
                .and_then(|v| v.get("login"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at: raw
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            production: raw
                .get("production_environment")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
    }

    #[test]
    fn test_normalize_deployment_full() {
        let json = serde_json::json!({
            "id": 42,
            "ref": "main",
            "environment": "production",
            "task": "deploy",
            "description": "Deploy main",
            "creator": { "login": "octocat" },
            "created_at": "2024-01-01T00:00:00Z",
            "production_environment": true
        });

        let result = normalize_deployment(&json);

        assert_eq!(result.id, 42);

        assert_eq!(result.r#ref, "main");
        assert_eq!(result.environment, "production");

        assert_eq!(result.task, "deploy");
        assert_eq!(result.description, Some("Deploy main".to_string()));

        assert_eq!(result.creator, Some("octocat".to_string()));
        assert!(result.production);
    }

    #[test]
    fn test_normalize_deployment_minimal() {
        let json = serde_json::json!({
            "id": 1,
            "ref": "dev",
            "environment": "staging",
            "task": "deploy",
            "created_at": "2024-01-01T00:00:00Z"
        });

        let result = normalize_deployment(&json);

        assert_eq!(result.id, 1);

        assert_eq!(result.r#ref, "dev");
        assert_eq!(result.environment, "staging");

        assert!(result.description.is_none());
        assert!(result.creator.is_none());

        assert!(!result.production);
    }
}
