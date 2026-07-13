use gitfleet_core::errors::GitfleetError;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct PipelinesApi;

impl PipelinesApi {
    pub async fn list_pipelines(
        client: &ProviderClient,
        project: &str,
        limit: u32,
        page: Option<u32>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let mut endpoint = format!("/projects/{encoded}/pipelines?per_page={limit}");

        if let Some(p) = page {
            endpoint.push_str(&format!("&page={p}"));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list pipelines: {e}")))?;

        Ok(data)
    }

    pub async fn get_pipeline(
        client: &ProviderClient,
        project: &str,
        pipeline_id: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/pipelines/{pipeline_id}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get pipeline: {e}")))?;

        Ok(data)
    }

    pub async fn dispatch_pipeline(
        client: &ProviderClient,
        project: &str,
        r#ref: &str,
        _workflow_id: &str,
        _inputs: Option<serde_json::Value>,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/pipeline");

        let body = serde_json::json!({ "ref": r#ref });

        client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn list_jobs(
        client: &ProviderClient,
        project: &str,
        pipeline_id: &str,
        limit: u32,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/pipelines/{pipeline_id}/jobs?per_page={limit}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list jobs: {e}")))?;

        Ok(data)
    }

    pub async fn get_job(
        client: &ProviderClient,
        project: &str,
        job_id: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/jobs/{job_id}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get job: {e}")))?;

        Ok(data)
    }

    pub async fn cancel_job(
        client: &ProviderClient,
        project: &str,
        job_id: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/jobs/{job_id}/cancel");

        client
            .request_token_required(reqwest::Method::POST, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn retry_job(
        client: &ProviderClient,
        project: &str,
        job_id: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/jobs/{job_id}/retry");

        client
            .request_token_required(reqwest::Method::POST, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn delete_pipeline(
        client: &ProviderClient,
        project: &str,
        pipeline_id: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/pipelines/{pipeline_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipelines_dispatch_body() {
        let r#ref = "main";
        let body = serde_json::json!({ "ref": r#ref });

        assert_eq!(body["ref"], "main");
    }

    #[test]
    fn test_encode_path_pipeline() {
        assert_eq!(encode_path("org/repo"), "org%2Frepo");

        assert_eq!(encode_path("simple"), "simple");
    }
}
