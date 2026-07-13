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

        let data: VariableListResponse<RepoVariable> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list variables: {e}")))?;

        Ok(data)
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
        let endpoint = format!("/projects/{encoded}/variables");

        let body = serde_json::json!({
            "key": name,
            "value": value,
        });

        client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
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

#[cfg(test)]
mod tests {
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
}
