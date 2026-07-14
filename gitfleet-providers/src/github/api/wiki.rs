use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::{ProviderCapability, ProviderId};
use gitfleet_core::types::{WikiPage, WikiPageContent};

use crate::github::client::ProviderClient;

pub struct WikiApi;

impl WikiApi {
    pub async fn list(
        _client: &ProviderClient,
        _repo: &str,
    ) -> Result<Vec<WikiPage>, GitfleetError> {
        Err(unsupported())
    }

    pub async fn get_page(
        _client: &ProviderClient,
        _repo: &str,
        _page: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        Err(unsupported())
    }

    pub async fn create_page(
        _client: &ProviderClient,
        _repo: &str,
        _title: &str,
        _content: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        Err(unsupported())
    }

    pub async fn update_page(
        _client: &ProviderClient,
        _repo: &str,
        _page: &str,
        _content: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        Err(unsupported())
    }

    pub async fn delete_page(
        _client: &ProviderClient,
        _repo: &str,
        _page: &str,
    ) -> Result<(), GitfleetError> {
        Err(unsupported())
    }
}

#[cfg(test)]
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

fn unsupported() -> GitfleetError {
    GitfleetError::from(UnsupportedCapabilityError::new(
        ProviderId::GitHub,
        ProviderCapability::Wiki,
    ))
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
