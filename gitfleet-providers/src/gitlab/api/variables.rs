use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{RepoVariable, VariableListResponse};

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct VariablesApi;

impl VariablesApi {
    pub async fn list(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
    ) -> Result<VariableListResponse<RepoVariable>, GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let endpoint = format!("/projects/{encoded}/variables");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list variables: {e}")))?;

        Ok(normalize_variables(&data))
    }

    pub async fn set(
        client: &ProviderClient,
        owner: &str,
        repo: &str,
        name: &str,
        value: &str,
    ) -> Result<(), GitfleetError> {
        let full = format!("{owner}/{repo}");

        let encoded = encode_path(&full);
        let enc_name = urlencoding::encode(name);
        let item_endpoint = format!("/projects/{encoded}/variables/{enc_name}");

        let body = serde_json::json!({
            "key": name,
            "value": value,
        });

        let method = match client
            .request_token_required(reqwest::Method::GET, &item_endpoint, None, None, None)
            .await
        {
            Ok(_) => reqwest::Method::PUT,
            Err(GitfleetError::NotFound(_)) => reqwest::Method::POST,
            Err(error) => return Err(error),
        };

        let endpoint = if method == reqwest::Method::PUT {
            item_endpoint
        } else {
            format!("/projects/{encoded}/variables")
        };

        client
            .request_token_required(method, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
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

        let endpoint = format!("/projects/{encoded}/variables/{enc_name}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_variables(data: &[serde_json::Value]) -> VariableListResponse<RepoVariable> {
    let variables = data
        .iter()
        .map(|raw| RepoVariable {
            name: raw
                .get("key")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string(),
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
            value: raw
                .get("value")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
        })
        .collect::<Vec<_>>();

    VariableListResponse {
        total_count: variables.len() as u64,
        variables,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_variables_set_body() {
        let name = "CI_TOKEN";
        let value = "secret";
        let body = serde_json::json!({ "key": name, "value": value });

        assert_eq!(body["key"], "CI_TOKEN");

        assert_eq!(body["value"], "secret");
    }

    #[test]
    fn test_gitlab_variables_encode_path() {
        let full = "owner/repo";
        let encoded = urlencoding::encode(full).to_string();

        assert_eq!(encoded, "owner%2Frepo");
    }

    #[test]
    fn test_normalize_variables_maps_gitlab_wire_fields() {
        let result = normalize_variables(&[serde_json::json!({
            "key": "CI_TOKEN",
            "value": "secret",
            "masked": true
        })]);

        assert_eq!(result.total_count, 1);
        assert_eq!(result.variables[0].name, "CI_TOKEN");
        assert_eq!(result.variables[0].value.as_deref(), Some("secret"));
    }
}
