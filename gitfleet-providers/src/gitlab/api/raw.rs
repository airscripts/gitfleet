use gitfleet_core::errors::GitfleetError;

use crate::gitlab::client::ProviderClient;

pub struct RawApi;

impl RawApi {
    pub async fn get(
        client: &ProviderClient,
        endpoint: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let resp = client
            .request_token_required(reqwest::Method::GET, endpoint, None, None, None)
            .await?;

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse response: {e}")))
    }

    pub async fn post(
        client: &ProviderClient,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let resp = client
            .request_token_required(reqwest::Method::POST, endpoint, Some(body), None, None)
            .await?;

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse response: {e}")))
    }

    pub async fn delete(
        client: &ProviderClient,
        endpoint: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let resp = client
            .request_token_required(reqwest::Method::DELETE, endpoint, None, None, None)
            .await?;

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse response: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_api_methods_exist() {
        let _ = RawApi;
    }
}
