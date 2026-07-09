use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::GistSummary;

use crate::github::client::ProviderClient;

pub struct GistsApi;

impl GistsApi {
    pub async fn list(
        client: &ProviderClient,
        is_public: bool,
        limit: u32,
    ) -> Result<Vec<GistSummary>, GitfleetError> {
        let endpoint = if is_public {
            format!("/gists/public?per_page={limit}")
        } else {
            format!("/gists?per_page={limit}")
        };

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list gists: {e}")))?;

        Ok(raw.iter().map(normalize_gist).collect())
    }

    pub async fn list_for_user(
        client: &ProviderClient,
        owner: &str,
    ) -> Result<Vec<GistSummary>, GitfleetError> {
        let endpoint = format!("/users/{owner}/gists?per_page=100");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list gists: {e}")))?;

        Ok(raw.iter().map(normalize_gist).collect())
    }

    pub async fn get(client: &ProviderClient, gist_id: &str) -> Result<GistSummary, GitfleetError> {
        let endpoint = format!("/gists/{gist_id}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get gist: {e}")))?;

        Ok(normalize_gist(&raw))
    }

    pub async fn get_json(
        client: &ProviderClient,
        gist_id: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = format!("/gists/{gist_id}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get gist: {e}")))?;

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        description: Option<&str>,
        public: bool,
        files: serde_json::Value,
    ) -> Result<GistSummary, GitfleetError> {
        let mut payload = serde_json::json!({
            "public": public,
            "files": files,
        });

        if let Some(desc) = description {
            payload["description"] = serde_json::Value::String(desc.to_string());
        }

        let response = client
            .request_token_required(reqwest::Method::POST, "/gists", Some(payload), None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create gist: {e}")))?;

        Ok(normalize_gist(&raw))
    }

    pub async fn delete(client: &ProviderClient, gist_id: &str) -> Result<(), GitfleetError> {
        let endpoint = format!("/gists/{gist_id}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_gist(raw: &serde_json::Value) -> GistSummary {
    use gitfleet_core::types::GistFile;

    let files = raw
        .get("files")
        .and_then(|f| f.as_object())
        .map(|obj| {
            obj.values()
                .map(|v| GistFile {
                    filename: v
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    r#type: v
                        .get("type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    language: v
                        .get("language")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    raw_url: v
                        .get("raw_url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    size: v.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
                    content: v
                        .get("content")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    truncated: v.get("truncated").and_then(|v| v.as_bool()),
                })
                .collect()
        })
        .unwrap_or_default();

    GistSummary {
        id: raw
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        description: raw
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        public: raw.get("public").and_then(|v| v.as_bool()).unwrap_or(true),
        html_url: raw
            .get("html_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        git_pull_url: raw
            .get("git_pull_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
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
            .get("owner")
            .and_then(|o| o.get("login"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        files,
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_gist;

    #[test]
    fn test_normalize_gist_full() {
        let json = serde_json::json!({
            "id": "abc123",
            "description": "My gist",
            "public": true,
            "html_url": "https://gist.github.com/abc123",
            "git_pull_url": "https://gist.github.com/abc123.git",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "owner": { "login": "octocat" },
            "files": {
                "hello.rb": {
                    "filename": "hello.rb",
                    "type": "application/x-ruby",
                    "language": "Ruby",
                    "raw_url": "https://gist.github.com/raw/abc123/hello.rb",
                    "size": 167,
                    "content": "puts 'hello'",
                    "truncated": false
                }
            }
        });

        let result = normalize_gist(&json);

        assert_eq!(result.id, "abc123");

        assert_eq!(result.description, Some("My gist".to_string()));
        assert!(result.public);

        assert_eq!(result.html_url, "https://gist.github.com/abc123");
        assert_eq!(result.owner, Some("octocat".to_string()));

        assert_eq!(result.files.len(), 1);

        let file = &result.files[0];

        assert_eq!(file.filename, "hello.rb");

        assert_eq!(file.r#type, Some("application/x-ruby".to_string()));
        assert_eq!(file.language, Some("Ruby".to_string()));

        assert_eq!(file.size, 167);
        assert_eq!(file.content, Some("puts 'hello'".to_string()));
    }

    #[test]
    fn test_normalize_gist_minimal() {
        let json = serde_json::json!({
            "id": "xyz",
            "public": false,
            "html_url": "https://gist.github.com/xyz",
            "git_pull_url": "",
            "created_at": "",
            "updated_at": "",
            "files": {}
        });

        let result = normalize_gist(&json);

        assert_eq!(result.id, "xyz");

        assert!(result.description.is_none());
        assert!(!result.public);

        assert!(result.files.is_empty());
    }
}
