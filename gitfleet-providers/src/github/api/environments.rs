use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::EnvironmentListResponse;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct EnvironmentsApi;

impl EnvironmentsApi {
    pub async fn list(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
    ) -> Result<EnvironmentListResponse, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let endpoint = repo_path(&full, &["environments"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: EnvironmentListResponse = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list environments: {e}")))?;

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
        wait_timer: Option<u32>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let endpoint = repo_path(&full, &["environments", name]);
        let mut body = serde_json::json!({});

        if let Some(t) = wait_timer {
            body["wait_timer"] = serde_json::Value::Number(t.into());
        }

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(body), None, None)
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

        let endpoint = repo_path(&full, &["environments", name]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_environments_create_body_with_wait() {
        let wait_timer: u32 = 30;
        let mut body = serde_json::json!({});
        body["wait_timer"] = serde_json::Value::Number(wait_timer.into());
        assert_eq!(body["wait_timer"], 30);
    }

    #[test]
    fn test_environments_create_body_without_wait() {
        let body = serde_json::json!({});

        assert!(body.get("wait_timer").is_none());
    }
}
