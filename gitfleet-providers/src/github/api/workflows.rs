use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct WorkflowsApi;

impl WorkflowsApi {
    pub async fn list_workflows(
        client: &ProviderClient,
        repo: &str,
        limit: u32,
        page: Option<u32>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let page_param = page.map_or(1, |p| p);

        let endpoint = repo_path(repo, &["actions", "workflows"]);
        let endpoint = format!("{endpoint}?per_page={limit}&page={page_param}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list workflows: {e}")))?;

        Ok(data)
    }

    pub async fn get_workflow(
        client: &ProviderClient,
        repo: &str,
        workflow_id: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["actions", "workflows", workflow_id]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get workflow: {e}")))?;

        Ok(data)
    }

    pub async fn dispatch_workflow(
        client: &ProviderClient,
        repo: &str,
        workflow_id: &str,
        r#ref: &str,
        inputs: Option<serde_json::Value>,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["actions", "workflows", workflow_id, "dispatches"]);

        let mut body = serde_json::json!({ "ref": r#ref });

        if let Some(inp) = inputs {
            body["inputs"] = inp;
        }

        client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn list_runs(
        client: &ProviderClient,
        repo: &str,
        filters: &str,
        limit: u32,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["actions", "runs"]);

        let endpoint = format!("{endpoint}?{filters}&per_page={limit}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list runs: {e}")))?;

        Ok(data)
    }

    pub async fn get_run(
        client: &ProviderClient,
        repo: &str,
        run_id: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["actions", "runs", &run_id.to_string()]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get run: {e}")))?;

        Ok(data)
    }

    pub async fn list_run_jobs(
        client: &ProviderClient,
        repo: &str,
        run_id: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["actions", "runs", &run_id.to_string(), "jobs"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list run jobs: {e}")))?;

        Ok(data)
    }

    pub async fn cancel_run(
        client: &ProviderClient,
        repo: &str,
        run_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["actions", "runs", &run_id.to_string(), "cancel"]);

        client
            .request_token_required(reqwest::Method::POST, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn rerun(
        client: &ProviderClient,
        repo: &str,
        run_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["actions", "runs", &run_id.to_string(), "rerun"]);

        client
            .request_token_required(reqwest::Method::POST, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn delete_run(
        client: &ProviderClient,
        repo: &str,
        run_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["actions", "runs", &run_id.to_string()]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn download_run_logs(
        client: &ProviderClient,
        repo: &str,
        run_id: u64,
    ) -> Result<reqwest::Response, GitfleetError> {
        let endpoint = repo_path(repo, &["actions", "runs", &run_id.to_string(), "logs"]);

        client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await
    }

    pub async fn set_workflow_enabled(
        client: &ProviderClient,
        repo: &str,
        workflow_id: &str,
        enabled: bool,
    ) -> Result<(), GitfleetError> {
        let action = if enabled { "enable" } else { "disable" };
        let endpoint = repo_path(repo, &["actions", "workflows", workflow_id, action]);

        client
            .request_token_required(reqwest::Method::PUT, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn list_caches(
        client: &ProviderClient,
        repo: &str,
        key: Option<&str>,
        limit: u32,
    ) -> Result<serde_json::Value, GitfleetError> {
        let mut endpoint = format!(
            "{}?per_page={limit}",
            repo_path(repo, &["actions", "caches"])
        );

        if let Some(k) = key {
            endpoint.push_str(&format!("&key={k}"));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list caches: {e}")))?;

        Ok(data)
    }

    pub async fn delete_cache(
        client: &ProviderClient,
        repo: &str,
        cache_id: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["actions", "caches", &cache_id.to_string()]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}
