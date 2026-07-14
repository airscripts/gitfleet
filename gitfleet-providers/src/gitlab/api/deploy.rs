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
            endpoint.push_str(&format!("&environment={}", urlencoding::encode(env)));
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
                environment: environment_name(raw),
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
                production: environment_name(raw) == "production",
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

        let body = normalize_create_input(client, project, &input).await?;

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
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
            environment: environment_name(&raw),
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
            production: environment_name(&raw) == "production",
        })
    }
}

async fn normalize_create_input(
    client: &ProviderClient,
    project: &str,
    input: &serde_json::Value,
) -> Result<serde_json::Value, GitfleetError> {
    let reference = input
        .get("ref")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| GitfleetError::new("Deployment ref is required."))?;
    let environment = input
        .get("environment")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| GitfleetError::new("Deployment environment is required."))?;
    let encoded = encode_path(project);
    let encoded_ref = urlencoding::encode(reference);
    let commit_endpoint = format!("/projects/{encoded}/repository/commits/{encoded_ref}");
    let response = client
        .request_token_required(reqwest::Method::GET, &commit_endpoint, None, None, None)
        .await?;
    let commit: serde_json::Value = crate::parse_json(response)
        .await
        .map_err(|e| GitfleetError::new(format!("Failed to resolve deployment ref: {e}")))?;
    let sha = commit
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| GitfleetError::new("GitLab commit response did not include a SHA."))?;

    let tag_endpoint = format!("/projects/{encoded}/repository/tags/{encoded_ref}");
    let tag = match client
        .request_token_required(reqwest::Method::GET, &tag_endpoint, None, None, None)
        .await
    {
        Ok(_) => true,
        Err(GitfleetError::NotFound(_)) => false,
        Err(error) => return Err(error),
    };

    Ok(serde_json::json!({
        "environment": environment,
        "sha": sha,
        "ref": reference,
        "tag": tag,
        "status": "running",
    }))
}

fn environment_name(raw: &serde_json::Value) -> String {
    raw.get("environment")
        .and_then(|environment| {
            environment
                .as_str()
                .or_else(|| environment.get("name").and_then(serde_json::Value::as_str))
        })
        .unwrap_or_default()
        .to_string()
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
