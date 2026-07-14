use gitfleet_core::errors::{GitfleetError, NotFoundError};
use gitfleet_core::types::{Environment, EnvironmentListResponse};

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct EnvironmentsApi;

impl EnvironmentsApi {
    pub async fn list(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
    ) -> Result<EnvironmentListResponse, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let endpoint = format!("/projects/{encoded}/environments");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list environments: {e}")))?;

        Ok(normalize_environments(&data))
    }

    pub async fn create(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
        _wait_timer: Option<u32>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let endpoint = format!("/projects/{encoded}/environments");

        let body = serde_json::json!({ "name": name });
        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create environment: {e}")))?;

        Ok(data)
    }

    pub async fn delete(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let enc_name = urlencoding::encode(name);
        let lookup_endpoint = format!("/projects/{encoded}/environments?name={enc_name}");

        let response = client
            .request_token_required(reqwest::Method::GET, &lookup_endpoint, None, None, None)
            .await?;

        let environments: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to resolve environment: {e}")))?;

        let environment_id = environments
            .iter()
            .find(|environment| {
                environment.get("name").and_then(serde_json::Value::as_str) == Some(name)
            })
            .and_then(|environment| environment.get("id"))
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                GitfleetError::from(NotFoundError::new(format!(
                    "Environment '{name}' was not found."
                )))
            })?;

        let endpoint = format!("/projects/{encoded}/environments/{environment_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_environments(data: &[serde_json::Value]) -> EnvironmentListResponse {
    let environments = data
        .iter()
        .map(|raw| {
            let external_url = raw
                .get("external_url")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string);

            Environment {
                id: raw
                    .get("id")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
                name: raw
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                url: external_url.clone(),
                html_url: external_url.unwrap_or_default(),
                created_at: raw
                    .get("created_at")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                updated_at: raw
                    .get("updated_at")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                wait_timer: None,
                protection_rules: None,
            }
        })
        .collect::<Vec<_>>();

    EnvironmentListResponse {
        total_count: environments.len() as u64,
        environments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environments_create_body() {
        let name = "production";
        let body = serde_json::json!({ "name": name });

        assert_eq!(body["name"], "production");
    }

    #[test]
    fn test_environments_encode_path() {
        let full = "owner/repo";
        let encoded = urlencoding::encode(full).to_string();

        assert_eq!(encoded, "owner%2Frepo");
    }

    #[test]
    fn test_normalize_environments_wraps_gitlab_array() {
        let result = normalize_environments(&[serde_json::json!({
            "id": 7,
            "name": "production",
            "external_url": "https://production.example.com",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z"
        })]);

        assert_eq!(result.total_count, 1);
        assert_eq!(result.environments[0].id, 7);
        assert_eq!(result.environments[0].name, "production");
        assert_eq!(
            result.environments[0].url.as_deref(),
            Some("https://production.example.com")
        );
    }
}
