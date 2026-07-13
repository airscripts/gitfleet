use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{WikiPage, WikiPageContent};

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct WikisApi;

impl WikisApi {
    pub async fn list(
        client: &ProviderClient,
        project: &str,
    ) -> Result<Vec<WikiPage>, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/wikis");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list wiki pages: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| WikiPage {
                title: raw
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                path: raw
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                format: raw
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("markdown")
                    .to_string(),
                filename: raw
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }

    pub async fn get_page(
        client: &ProviderClient,
        project: &str,
        slug: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        let encoded = encode_path(project);

        let enc_slug = urlencoding::encode(slug);
        let endpoint = format!("/projects/{encoded}/wikis/{enc_slug}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get wiki page: {e}")))?;

        Ok(WikiPageContent {
            page: WikiPage {
                title: raw
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                path: raw
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                format: raw
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("markdown")
                    .to_string(),
                filename: raw
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            },
            content: raw
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    pub async fn create_page(
        client: &ProviderClient,
        project: &str,
        title: &str,
        content: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/wikis");
        let payload = serde_json::json!({
            "title": title,
            "content": content,
        });

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(payload), None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create wiki page: {e}")))?;

        Ok(WikiPageContent {
            page: WikiPage {
                title: raw
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                path: raw
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                format: raw
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("markdown")
                    .to_string(),
                filename: raw
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            },
            content: raw
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    pub async fn update_page(
        client: &ProviderClient,
        project: &str,
        slug: &str,
        content: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        let encoded = encode_path(project);

        let enc_slug = urlencoding::encode(slug);
        let endpoint = format!("/projects/{encoded}/wikis/{enc_slug}");

        let payload = serde_json::json!({
            "content": content,
        });

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(payload), None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update wiki page: {e}")))?;

        Ok(WikiPageContent {
            page: WikiPage {
                title: raw
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                path: raw
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                format: raw
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("markdown")
                    .to_string(),
                filename: raw
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            },
            content: raw
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    pub async fn delete_page(
        client: &ProviderClient,
        project: &str,
        slug: &str,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let enc_slug = urlencoding::encode(slug);
        let endpoint = format!("/projects/{encoded}/wikis/{enc_slug}");

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::WikiPage;

    fn normalize_wiki_page(raw: &serde_json::Value) -> WikiPage {
        WikiPage {
            title: raw
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            path: raw
                .get("slug")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            format: raw
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("markdown")
                .to_string(),
            filename: raw
                .get("slug")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    #[test]
    fn test_normalize_gitlab_wiki_page_full() {
        let json = serde_json::json!({
            "title": "Home",
            "slug": "home",
            "format": "markdown"
        });

        let result = normalize_wiki_page(&json);

        assert_eq!(result.title, "Home");

        assert_eq!(result.path, "home");
        assert_eq!(result.format, "markdown");

        assert_eq!(result.filename, "home");
    }

    #[test]
    fn test_normalize_gitlab_wiki_page_minimal() {
        let json = serde_json::json!({});

        let result = normalize_wiki_page(&json);

        assert_eq!(result.title, "");

        assert_eq!(result.path, "");
        assert_eq!(result.format, "markdown");

        assert_eq!(result.filename, "");
    }
}
