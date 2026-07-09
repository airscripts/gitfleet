use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::Label;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct LabelsApi;

impl LabelsApi {
    pub async fn fetch(client: &ProviderClient, repo: &str) -> Result<Vec<Label>, GitfleetError> {
        let endpoint = repo_path(repo, &["labels"]);

        let response = client
            .request_optional_token(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to fetch labels: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| Label {
                name: raw
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                color: raw
                    .get("color")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                description: raw
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                new_name: None,
            })
            .collect())
    }

    pub async fn create(
        client: &ProviderClient,
        label: &Label,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["labels"]);

        let json = serde_json::json!({
            "name": label.name,
            "color": label.color,
            "description": label.description,
        });

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(json), None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create label: {e}")))?;

        Ok(data)
    }

    pub async fn delete(
        client: &ProviderClient,
        name: &str,
        repo: &str,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["labels", name]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalize_label(raw: &serde_json::Value) -> Label {
        Label {
            name: raw
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            color: raw
                .get("color")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: raw
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            new_name: None,
        }
    }

    #[test]
    fn test_normalize_label_full() {
        let json = serde_json::json!({
            "name": "bug",
            "color": "ff0000",
            "description": "Bug reports"
        });

        let result = normalize_label(&json);

        assert_eq!(result.name, "bug");

        assert_eq!(result.color, "ff0000");
        assert_eq!(result.description, "Bug reports");

        assert!(result.new_name.is_none());
    }

    #[test]
    fn test_normalize_label_minimal() {
        let json = serde_json::json!({});

        let result = normalize_label(&json);

        assert_eq!(result.name, "");

        assert_eq!(result.color, "");
        assert_eq!(result.description, "");
    }
}
