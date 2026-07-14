use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::Label;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct LabelsApi;

impl LabelsApi {
    pub async fn list(client: &ProviderClient, project: &str) -> Result<Vec<Label>, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/labels?per_page=100");

        let response = client
            .request_optional_token(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list labels: {e}")))?;

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
        project: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/labels");
        let json = serde_json::json!({
            "name": label.name,
            "color": label.color,
            "description": label.description,
        });

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(json), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create label: {e}")))?;

        Ok(data)
    }

    pub async fn delete(
        client: &ProviderClient,
        name: &str,
        project: &str,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let enc_name = urlencoding::encode(name);
        let endpoint = format!("/projects/{encoded}/labels/{enc_name}");

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
    fn test_normalize_gitlab_label_full() {
        let json = serde_json::json!({
            "name": "bug",
            "color": "#ff0000",
            "description": "Bug reports"
        });

        let result = normalize_label(&json);

        assert_eq!(result.name, "bug");

        assert_eq!(result.color, "#ff0000");
        assert_eq!(result.description, "Bug reports");
    }

    #[test]
    fn test_normalize_gitlab_label_minimal() {
        let json = serde_json::json!({});

        let result = normalize_label(&json);

        assert_eq!(result.name, "");

        assert_eq!(result.color, "");
        assert_eq!(result.description, "");
    }

    #[test]
    fn test_encode_path() {
        assert_eq!(encode_path("org/repo"), "org%2Frepo");

        assert_eq!(encode_path("simple"), "simple");
    }
}
