use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::CodeSearchResult;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct CodeApi;

impl CodeApi {
    pub async fn file_contents(
        client: &ProviderClient,
        project: &str,
        path: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let mut endpoint = format!(
            "/projects/{encoded}/repository/files/{}",
            urlencoding::encode(path)
        );

        endpoint.push_str("?raw=false");

        if let Some(r) = r#ref {
            endpoint.push_str(&format!("&ref={}", urlencoding::encode(r)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get file contents: {e}")))?;

        Ok(data)
    }

    pub async fn search(
        client: &ProviderClient,
        query: &str,
        repo: Option<&str>,
        _language: Option<&str>,
        limit: u32,
    ) -> Result<Vec<CodeSearchResult>, GitfleetError> {
        let mut endpoint = format!(
            "/search?scope=blobs&search={}&per_page={limit}",
            urlencoding::encode(query)
        );

        if let Some(r) = repo {
            endpoint.push_str(&format!("&project_id={}", urlencoding::encode(r)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to search code: {e}")))?;

        Ok(raw
            .iter()
            .map(|item| CodeSearchResult {
                file: item
                    .get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                repo: item
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                url: item
                    .get("ref")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::CodeSearchResult;

    fn normalize_gitlab_code_search(item: &serde_json::Value) -> CodeSearchResult {
        CodeSearchResult {
            file: item
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            repo: item
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            url: item
                .get("ref")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    #[test]
    fn test_normalize_gitlab_code_search_full() {
        let json = serde_json::json!({
            "filename": "main.rs",
            "path": "src/main.rs",
            "ref": "main"
        });

        let result = normalize_gitlab_code_search(&json);

        assert_eq!(result.file, "main.rs");

        assert_eq!(result.repo, "src/main.rs");
        assert_eq!(result.url, "main");
    }

    #[test]
    fn test_normalize_gitlab_code_search_minimal() {
        let json = serde_json::json!({});

        let result = normalize_gitlab_code_search(&json);

        assert_eq!(result.file, "");

        assert_eq!(result.repo, "");
        assert_eq!(result.url, "");
    }
}
