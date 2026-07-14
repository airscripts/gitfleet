use gitfleet_core::errors::{GitfleetError, UnprocessableError};
use gitfleet_core::types::{GistFile, GistSummary};

use crate::gitlab::client::ProviderClient;

pub struct SnippetsApi;

impl SnippetsApi {
    pub async fn list(
        client: &ProviderClient,
        owner: &str,
    ) -> Result<Vec<GistSummary>, GitfleetError> {
        if !owner.is_empty() {
            return Err(GitfleetError::from(UnprocessableError::new(
                "GitLab can only list snippets for the authenticated user; omit --owner.",
            )));
        }

        let endpoint = "/snippets?per_page=100";

        let response = client
            .request_token_required(reqwest::Method::GET, endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list snippets: {e}")))?;

        Ok(data.iter().map(normalize_snippet).collect())
    }

    pub async fn get(
        client: &ProviderClient,
        snippet_id: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!("/snippets/{snippet_id}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get snippet: {e}")))?;

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        description: &str,
        public: bool,
        files: serde_json::Value,
    ) -> Result<GistSummary, GitfleetError> {
        let snippet_files = normalize_files(&files);
        let visibility = if public { "public" } else { "private" };
        let body = serde_json::json!({
            "title": description,
            "files": snippet_files,
            "visibility": visibility,
        });

        let response = client
            .request_token_required(reqwest::Method::POST, "/snippets", Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create snippet: {e}")))?;

        Ok(normalize_snippet(&data))
    }

    pub async fn delete(client: &ProviderClient, snippet_id: &str) -> Result<(), GitfleetError> {
        let endpoint = format!("/snippets/{snippet_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_files(files: &serde_json::Value) -> Vec<serde_json::Value> {
    files
        .as_object()
        .into_iter()
        .flat_map(|files| files.iter())
        .filter_map(|(file_path, value)| {
            value
                .get("content")
                .and_then(serde_json::Value::as_str)
                .map(|content| {
                    serde_json::json!({
                        "file_path": file_path,
                        "content": content,
                    })
                })
        })
        .collect()
}

fn normalize_snippet(raw: &serde_json::Value) -> GistSummary {
    let id = raw
        .get("id")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .to_string();

    let visibility = raw
        .get("visibility")
        .and_then(|v| v.as_str())
        .unwrap_or("private");

    let web_url = raw
        .get("web_url")
        .or_else(|| raw.get("raw_url"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    GistSummary {
        id,
        description: raw
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        public: visibility != "private",
        html_url: web_url.clone(),
        git_pull_url: web_url,
        created_at: raw
            .get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        updated_at: raw
            .get("updated_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        owner: raw
            .get("author")
            .and_then(|a| a.get("username"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        files: vec![GistFile {
            filename: raw
                .get("file_name")
                .and_then(|v| v.as_str())
                .unwrap_or("snippet.txt")
                .to_string(),
            r#type: None,
            language: None,
            raw_url: raw
                .get("raw_url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            size: 0,
            content: None,
            truncated: None,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_snippet_full() {
        let json = serde_json::json!({
            "id": 42,
            "title": "My Snippet",
            "visibility": "internal",
            "web_url": "https://gitlab.com/snippets/42",
            "raw_url": "https://gitlab.com/snippets/42/raw",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "author": { "username": "alice" },
            "file_name": "main.py"
        });

        let result = normalize_snippet(&json);

        assert_eq!(result.id, "42");

        assert_eq!(result.description, Some("My Snippet".to_string()));
        assert!(result.public);

        assert_eq!(result.owner, Some("alice".to_string()));
        assert_eq!(result.files[0].filename, "main.py");
    }

    #[test]
    fn test_normalize_snippet_minimal() {
        let json = serde_json::json!({});

        let result = normalize_snippet(&json);

        assert_eq!(result.id, "0");

        assert!(!result.public);
    }

    #[test]
    fn test_normalize_files() {
        let files = serde_json::json!({"hello.txt": {"content": "hello world"}});
        let normalized = normalize_files(&files);

        assert_eq!(normalized[0]["file_path"], "hello.txt");
        assert_eq!(normalized[0]["content"], "hello world");
    }

    #[test]
    fn test_normalize_files_preserves_filename() {
        let files = serde_json::json!({"main.py": {"content": "print('hi')"}});
        let normalized = normalize_files(&files);

        assert_eq!(normalized[0]["file_path"], "main.py");
    }

    #[test]
    fn test_normalize_files_empty() {
        let files = serde_json::json!({});

        assert!(normalize_files(&files).is_empty());
    }
}
