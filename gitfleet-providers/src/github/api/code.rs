use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::CodeSearchResult;

use crate::github::api::path::{encode_path, repo_path};
use crate::github::client::ProviderClient;

pub struct CodeApi;

impl CodeApi {
    pub async fn file_contents(
        client: &ProviderClient,
        repo: &str,
        path: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let mut endpoint = repo_path(repo, &["contents"]);
        endpoint.push('/');
        endpoint.push_str(&encode_path(path));

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

    pub async fn search(
        client: &ProviderClient,
        query: &str,
        repo: Option<&str>,
        language: Option<&str>,
        limit: u32,
    ) -> Result<Vec<CodeSearchResult>, GitfleetError> {
        let mut q = query.to_string();

        if let Some(r) = repo {
            q.push_str(&format!(" repo:{r}"));
        }

        if let Some(l) = language {
            q.push_str(&format!(" language:{l}"));
        }

        let endpoint = format!(
            "/search/code?q={}&per_page={limit}",
            urlencoding::encode(&q)
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to search code: {e}")))?;

        let items = raw
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(items
            .iter()
            .map(|item| CodeSearchResult {
                file: item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                repo: item
                    .get("repository")
                    .and_then(|r| r.get("full_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                url: item
                    .get("html_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalize_code_search_result(item: &serde_json::Value) -> CodeSearchResult {
        CodeSearchResult {
            file: item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            repo: item
                .get("repository")
                .and_then(|r| r.get("full_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            url: item
                .get("html_url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    #[test]
    fn test_normalize_code_search_result_full() {
        let json = serde_json::json!({
            "name": "main.rs",
            "repository": { "full_name": "org/repo" },
            "html_url": "https://github.com/org/repo/blob/main/main.rs"
        });

        let result = normalize_code_search_result(&json);

        assert_eq!(result.file, "main.rs");

        assert_eq!(result.repo, "org/repo");
        assert_eq!(result.url, "https://github.com/org/repo/blob/main/main.rs");
    }

    #[test]
    fn test_normalize_code_search_result_minimal() {
        let json = serde_json::json!({});

        let result = normalize_code_search_result(&json);

        assert_eq!(result.file, "");

        assert_eq!(result.repo, "");
        assert_eq!(result.url, "");
    }
}
