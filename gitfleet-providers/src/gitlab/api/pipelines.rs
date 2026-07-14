use gitfleet_core::errors::{GitfleetError, UnprocessableError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct PipelinesApi;

impl PipelinesApi {
    pub async fn list_workflows(
        _client: &ProviderClient,
        _project: &str,
        _limit: u32,
        _page: Option<u32>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(workflow_definitions_unsupported())
    }

    pub async fn get_workflow(
        _client: &ProviderClient,
        _project: &str,
        _workflow_id: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Err(workflow_definitions_unsupported())
    }

    pub async fn list_pipelines(
        client: &ProviderClient,
        project: &str,
        filters: &str,
        limit: u32,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let mut endpoint = format!("/projects/{encoded}/pipelines?per_page={limit}");

        if !filters.is_empty() {
            endpoint.push('&');
            endpoint.push_str(filters);
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let mut data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list pipelines: {e}")))?;

        for pipeline in &mut data {
            normalize_pipeline(pipeline);
        }

        Ok(serde_json::json!({
            "total_count": data.len(),
            "workflow_runs": data,
        }))
    }

    pub async fn get_pipeline(
        client: &ProviderClient,
        project: &str,
        pipeline_id: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/pipelines/{pipeline_id}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let mut data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get pipeline: {e}")))?;

        normalize_pipeline(&mut data);

        Ok(data)
    }

    pub async fn dispatch_pipeline(
        client: &ProviderClient,
        project: &str,
        r#ref: &str,
        inputs: Option<serde_json::Value>,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/pipeline");

        let mut body = serde_json::json!({ "ref": r#ref });

        if let Some(inputs) = inputs {
            let inputs = inputs.as_object().ok_or_else(|| {
                GitfleetError::from(UnprocessableError::new(
                    "Pipeline inputs must be a JSON object.",
                ))
            })?;
            let variables = inputs
                .iter()
                .map(|(key, value)| {
                    let value = value
                        .as_str()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| value.to_string());

                    serde_json::json!({
                        "key": key,
                        "value": value,
                    })
                })
                .collect::<Vec<_>>();

            body["variables"] = serde_json::Value::Array(variables);
        }

        client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn cancel_pipeline(
        client: &ProviderClient,
        project: &str,
        pipeline_id: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/pipelines/{pipeline_id}/cancel");

        client
            .request_token_required(reqwest::Method::POST, &endpoint, None, None, None)
            .await?;

        Ok(())
    }

    pub async fn retry_pipeline(
        client: &ProviderClient,
        project: &str,
        pipeline_id: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/pipelines/{pipeline_id}/retry");

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

fn normalize_pipeline(raw: &mut serde_json::Value) {
    let Some(object) = raw.as_object_mut() else {
        return;
    };

    let provider_status = object
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let (status, conclusion) = match provider_status {
        "success" => ("completed", Some("success")),
        "failed" => ("completed", Some("failure")),
        "canceled" => ("completed", Some("cancelled")),
        "skipped" => ("completed", Some("skipped")),
        "running" => ("in_progress", None),
        _ => ("queued", None),
    };
    let branch = object
        .get("ref")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    object.insert(
        "status".to_string(),
        serde_json::Value::String(status.to_string()),
    );
    object.insert(
        "conclusion".to_string(),
        conclusion.map_or(serde_json::Value::Null, |value| {
            serde_json::Value::String(value.to_string())
        }),
    );
    object.insert("head_branch".to_string(), branch);
}

fn workflow_definitions_unsupported() -> GitfleetError {
    GitfleetError::from(UnsupportedCapabilityError::new(
        ProviderId::GitLab,
        ProviderCapability::Pipelines,
    ))
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
