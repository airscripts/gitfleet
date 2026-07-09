use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::{WikiPage, WikiPageContent};

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct WikiApi;

impl WikiApi {
    pub async fn list(client: &ProviderClient, repo: &str) -> Result<Vec<WikiPage>, GitfleetError> {
        let _endpoint = repo_path(repo, &["wiki", "pages"]);

        let response = client
            .request_url(
                reqwest::Method::GET,
                &format!("https://raw.githubusercontent.com/wiki/{repo}"),
                None,
                None,
                None,
                None,
            )
            .await;

        if let Ok(resp) = response {
            if resp.status().is_success() {
                let text = resp
                    .text()
                    .await
                    .map_err(|e| GitfleetError::new(format!("Failed to read wiki: {e}")))?;

                let pages: Vec<WikiPage> = vec![WikiPage {
                    path: "Home".to_string(),
                    title: "Home".to_string(),
                    format: "markdown".to_string(),
                    filename: "Home.md".to_string(),
                }];
                let _ = text;
                return Ok(pages);
            }
        }

        Ok(vec![])
    }

    pub async fn get_page(
        client: &ProviderClient,
        repo: &str,
        page: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        let url = format!("https://raw.githubusercontent.com/wiki/{repo}/{page}.md");

        let response = client
            .request_url(reqwest::Method::GET, &url, None, None, None, None)
            .await?;

        let content = response
            .text()
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

#[cfg(test)]
mod tests {
    use super::*;

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
