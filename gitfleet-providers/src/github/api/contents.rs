use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct ContentsApi;

impl ContentsApi {
    pub async fn list(
        client: &ProviderClient,
        repo: &str,
        path: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = match path {
            Some(p) => repo_path(repo, &["contents", p]),
            None => repo_path(repo, &["contents"]),
        };

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list contents: {e}")))?;

        Ok(data)
    }

    pub async fn get(
        client: &ProviderClient,
        repo: &str,
        path: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let mut endpoint = repo_path(repo, &["contents", path]);

        if let Some(r) = r#ref {
            endpoint.push_str(&format!("?ref={}", urlencoding::encode(r)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get file contents: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contents_list_endpoint() {
        let repo = "owner/repo";
        let endpoint = repo_path(repo, &["contents"]);

        assert_eq!(endpoint, "/repos/owner/repo/contents");
    }

    #[test]
    fn test_contents_list_with_path_endpoint() {
        let repo = "owner/repo";
        let endpoint = repo_path(repo, &["contents", "src"]);

        assert_eq!(endpoint, "/repos/owner/repo/contents/src");
    }
}
