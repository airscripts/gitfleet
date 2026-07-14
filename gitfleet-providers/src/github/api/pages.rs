use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct PagesApi;

impl PagesApi {
    pub async fn get(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["pages"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get pages: {e}")))?;

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        repo: &str,
        source: &str,
        build_type: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["pages"]);

        let (branch, path) = source
            .split_once('/')
            .map_or((source, "/".to_string()), |(branch, path)| {
                (branch, format!("/{path}"))
            });
        let mut body = serde_json::json!({
            "source": { "branch": branch, "path": path }
        });

        if let Some(bt) = build_type {
            body["build_type"] = serde_json::Value::String(bt.to_string());
        }

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create pages: {e}")))?;

        Ok(data)
    }

    pub async fn remove(client: &ProviderClient, repo: &str) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["pages"]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pages_create_body() {
        let source = "main";
        let body = serde_json::json!({
            "source": { "branch": source, "path": "/" }
        });

        assert_eq!(body["source"]["branch"], "main");

        assert_eq!(body["source"]["path"], "/");
    }

    #[test]
    fn test_pages_create_body_with_build_type() {
        let source = "gh-pages";
        let build_type = "workflow";
        let mut body = serde_json::json!({
            "source": { "branch": source, "path": "/" }
        });

        body["build_type"] = serde_json::Value::String(build_type.to_string());
        assert_eq!(body["build_type"], "workflow");
    }

    #[test]
    fn test_pages_source_splits_branch_and_directory() {
        let source = "main/docs/api";
        let (branch, path) = source
            .split_once('/')
            .map_or((source, "/".to_string()), |(branch, path)| {
                (branch, format!("/{path}"))
            });

        assert_eq!(branch, "main");
        assert_eq!(path, "/docs/api");
    }
}
