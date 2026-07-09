use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::Discussion;

use crate::gitlab::client::ProviderClient;

pub struct DiscussionsApi;

impl DiscussionsApi {
    pub async fn list(
        client: &ProviderClient,
        owner: &str,
        name: &str,
        _category_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Discussion>, GitfleetError> {
        let full = format!("{owner}/{name}");

        let encoded = urlencoding::encode(&full).to_string();
        let endpoint = format!("/projects/{encoded}/issues?per_page={limit}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list discussions: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| Discussion {
                id: raw
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .map(|i| i.to_string())
                    .unwrap_or_default(),
                url: raw
                    .get("web_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                body: raw
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                title: raw
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                number: raw.get("iid").and_then(|v| v.as_u64()).unwrap_or(0),
                author: raw
                    .get("author")
                    .and_then(|v| v.as_object())
                    .and_then(|o| o.get("username"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                closed: raw
                    .get("state")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "closed")
                    .unwrap_or(false),
                category: String::new(),
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
                comments_count: raw
                    .get("user_notes_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
            })
            .collect())
    }

    pub async fn get(
        client: &ProviderClient,
        owner: &str,
        name: &str,
        discussion_number: u64,
    ) -> Result<Discussion, GitfleetError> {
        let full = format!("{owner}/{name}");

        let encoded = urlencoding::encode(&full).to_string();
        let endpoint = format!("/projects/{encoded}/issues/{discussion_number}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get discussion: {e}")))?;

        Ok(Discussion {
            id: raw
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|i| i.to_string())
                .unwrap_or_default(),
            url: raw
                .get("web_url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            body: raw
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            title: raw
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            number: raw.get("iid").and_then(|v| v.as_u64()).unwrap_or(0),
            author: raw
                .get("author")
                .and_then(|v| v.as_object())
                .and_then(|o| o.get("username"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            closed: raw
                .get("state")
                .and_then(|v| v.as_str())
                .map(|s| s == "closed")
                .unwrap_or(false),
            category: String::new(),
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
            comments_count: raw
                .get("user_notes_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        })
    }

    pub async fn create(
        client: &ProviderClient,
        owner: &str,
        name: &str,
        title: &str,
        body: &str,
        _category_id: Option<&str>,
    ) -> Result<Discussion, GitfleetError> {
        let full = format!("{owner}/{name}");

        let encoded = urlencoding::encode(&full).to_string();
        let endpoint = format!("/projects/{encoded}/issues");

        let payload = serde_json::json!({
            "title": title,
            "description": body,
        });

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(payload), None, None)
            .await?;

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create discussion: {e}")))?;

        Ok(Discussion {
            id: raw
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|i| i.to_string())
                .unwrap_or_default(),
            url: raw
                .get("web_url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            body: raw
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            title: raw
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            number: raw.get("iid").and_then(|v| v.as_u64()).unwrap_or(0),
            author: raw
                .get("author")
                .and_then(|v| v.as_object())
                .and_then(|o| o.get("username"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            closed: raw
                .get("state")
                .and_then(|v| v.as_str())
                .map(|s| s == "closed")
                .unwrap_or(false),
            category: String::new(),
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
            comments_count: raw
                .get("user_notes_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalize_gitlab_discussion(raw: &serde_json::Value) -> Discussion {
        let author_obj = raw.get("author").and_then(|v| v.as_object());
        Discussion {
            id: raw
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|i| i.to_string())
                .unwrap_or_default(),
            url: raw
                .get("web_url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            body: raw
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            title: raw
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            number: raw.get("iid").and_then(|v| v.as_u64()).unwrap_or(0),
            author: author_obj
                .and_then(|o| o.get("username"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            closed: raw
                .get("state")
                .and_then(|v| v.as_str())
                .map(|s| s == "closed")
                .unwrap_or(false),
            category: String::new(),
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
            comments_count: raw
                .get("user_notes_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        }
    }

    #[test]
    fn test_normalize_gitlab_discussion_full() {
        let json = serde_json::json!({
            "id": 123,
            "web_url": "https://gitlab.com/org/proj/-/issues/1",
            "description": "Bug description",
            "title": "Bug report",
            "iid": 1,
            "author": { "username": "dev1" },
            "state": "closed",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "user_notes_count": 3
        });

        let result = normalize_gitlab_discussion(&json);

        assert_eq!(result.id, "123");

        assert_eq!(result.url, "https://gitlab.com/org/proj/-/issues/1");
        assert_eq!(result.body, "Bug description");

        assert_eq!(result.title, "Bug report");
        assert_eq!(result.number, 1);

        assert_eq!(result.author, "dev1");
        assert!(result.closed);

        assert_eq!(result.comments_count, 3);
    }

    #[test]
    fn test_normalize_gitlab_discussion_minimal() {
        let json = serde_json::json!({});

        let result = normalize_gitlab_discussion(&json);

        assert_eq!(result.id, "");

        assert_eq!(result.title, "");
        assert!(!result.closed);

        assert_eq!(result.comments_count, 0);
    }
}
