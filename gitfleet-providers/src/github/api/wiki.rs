use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};
use gitfleet_core::types::{WikiPage, WikiPageContent};

use crate::github::api::path::{encode_path, repo_path};
use crate::github::client::ProviderClient;

pub struct WikiApi;

impl WikiApi {
    pub async fn list(client: &ProviderClient, repo: &str) -> Result<Vec<WikiPage>, GitfleetError> {
        let endpoint = repo_path(repo, &["wikis"]);
        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;
        let pages: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list wiki pages: {e}")))?;

        Ok(pages.iter().map(normalize_page).collect())
    }

    pub async fn get_page(
        client: &ProviderClient,
        repo: &str,
        page: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        ensure_public_wiki_reads(client)?;

        let url = format!(
            "https://raw.githubusercontent.com/wiki/{}/{}.md",
            encode_path(repo),
            encode_path(page)
        );

        let response = client
            .request_url_optional_token(
                reqwest::Method::GET,
                &url,
                None,
                None,
                Some("text/plain"),
                Some("text/plain"),
            )
            .await?;

        let content = crate::read_response_text(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to read wiki page: {e}")))?;

        Ok(WikiPageContent {
            page: WikiPage {
                path: page.to_string(),
                title: page.to_string(),
                format: "markdown".to_string(),
                filename: format!("{page}.md"),
            },
            content,
        })
    }

    pub async fn create_page(
        client: &ProviderClient,
        repo: &str,
        title: &str,
        content: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        let endpoint = repo_path(repo, &["wikis"]);

        let payload = serde_json::json!({
            "title": title,
            "content": content,
        });

        client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(payload), None, None)
            .await?;

        Ok(WikiPageContent {
            page: WikiPage {
                path: title.to_string(),
                title: title.to_string(),
                format: "markdown".to_string(),
                filename: format!("{title}.md"),
            },
            content: content.to_string(),
        })
    }

    pub async fn update_page(
        client: &ProviderClient,
        repo: &str,
        page: &str,
        content: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        let endpoint = repo_path(repo, &["wikis", page]);

        let payload = serde_json::json!({
            "content": content,
        });

        client
            .request_token_required(reqwest::Method::PATCH, &endpoint, Some(payload), None, None)
            .await?;

        Ok(WikiPageContent {
            page: WikiPage {
                path: page.to_string(),
                title: page.to_string(),
                format: "markdown".to_string(),
                filename: format!("{page}.md"),
            },
            content: content.to_string(),
        })
    }

    pub async fn delete_page(
        client: &ProviderClient,
        repo: &str,
        page: &str,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["wikis", page]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}

fn normalize_page(raw: &serde_json::Value) -> WikiPage {
    let title = raw
        .get("title")
        .or_else(|| raw.get("name"))
        .or_else(|| raw.get("path"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let path = raw
        .get("path")
        .or_else(|| raw.get("slug"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&title)
        .to_string();
    let format = raw
        .get("format")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("markdown")
        .to_string();
    let filename = raw
        .get("filename")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{path}.md"));

    WikiPage {
        path,
        title,
        format,
        filename,
    }
}

fn ensure_public_wiki_reads(client: &ProviderClient) -> Result<(), GitfleetError> {
    if client.supports_public_wiki_reads() {
        return Ok(());
    }

    Err(GitfleetError::from(UnsupportedCapabilityError::new(
        ProviderId::GitHub,
        ProviderCapability::Wiki,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_raw_wiki_reads_reject_non_public_host() {
        let client = ProviderClient::with_host("github.example.com");

        let result = WikiApi::get_page(&client, "org/repo", "Home").await;

        assert!(matches!(
            result,
            Err(GitfleetError::UnsupportedCapability(_))
        ));
    }

    #[test]
    fn test_normalize_page_supplies_defaults() {
        let page = normalize_page(&serde_json::json!({"title": "Getting Started"}));

        assert_eq!(page.title, "Getting Started");
        assert_eq!(page.path, "Getting Started");
        assert_eq!(page.format, "markdown");
        assert_eq!(page.filename, "Getting Started.md");
    }

    #[test]
    fn test_wiki_page_round_trip() {
        let page = WikiPage {
            path: "Home".to_string(),
            title: "Home".to_string(),
            format: "markdown".to_string(),
            filename: "Home.md".to_string(),
        };

        let json = serde_json::to_value(&page).unwrap();

        let deserialized: WikiPage = serde_json::from_value(json).unwrap();

        assert_eq!(deserialized.path, "Home");

        assert_eq!(deserialized.title, "Home");
        assert_eq!(deserialized.format, "markdown");

        assert_eq!(deserialized.filename, "Home.md");
    }

    #[test]
    fn test_wiki_page_content_round_trip() {
        let page = WikiPage {
            path: "GettingStarted".to_string(),
            title: "Getting Started".to_string(),
            format: "markdown".to_string(),
            filename: "GettingStarted.md".to_string(),
        };

        let content = WikiPageContent {
            page: page.clone(),
            content: "# Getting Started\nWelcome!".to_string(),
        };

        let json = serde_json::to_value(&content).unwrap();

        let deserialized: WikiPageContent = serde_json::from_value(json).unwrap();

        assert_eq!(deserialized.page.path, "GettingStarted");

        assert_eq!(deserialized.content, "# Getting Started\nWelcome!");
    }
}
