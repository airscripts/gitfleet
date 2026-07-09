use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::DeploymentSummary;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct DeployApi;

impl DeployApi {
    pub async fn list(
        client: &ProviderClient,
        project: &str,
        environment: Option<&str>,
        limit: u32,
    ) -> Result<Vec<DeploymentSummary>, GitfleetError> {
        let encoded = encode_path(project);

        let mut endpoint = format!("/projects/{encoded}/deployments?per_page={limit}");

        if let Some(env) = environment {
            endpoint.push_str(&format!("&environment={env}"));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = response
            .json()
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
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                description: raw
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                creator: raw
                    .get("user")
                    .and_then(|v| v.get("username"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                created_at: raw
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                production: raw
                    .get("environment")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "production")
                    .unwrap_or(false),
            })
            .collect())
    }

    pub async fn create(
        client: &ProviderClient,
        project: &str,
        input: serde_json::Value,
    ) -> Result<DeploymentSummary, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/deployments");

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(input), None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
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
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: raw
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            creator: raw
                .get("user")
                .and_then(|v| v.get("username"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at: raw
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            production: raw
                .get("environment")
                .and_then(|v| v.as_str())
                .map(|s| s == "production")
                .unwrap_or(false),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_deploy_list_endpoint() {
        let project = "org/repo";
        let encoded = urlencoding::encode(project).to_string();

        let endpoint = format!("/projects/{encoded}/deployments?per_page=30");

        assert!(endpoint.contains("org%2Frepo"));
    }
}
