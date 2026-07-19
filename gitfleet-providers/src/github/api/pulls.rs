use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::PullRequest;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct PullsApi;

impl PullsApi {
    pub async fn fetch(
        client: &ProviderClient,
        number: u64,
        repo: &str,
    ) -> Result<PullRequest, GitfleetError> {
        let endpoint = repo_path(repo, &["pulls", &number.to_string()]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let pr: PullRequest = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to fetch pull request: {e}")))?;

        Ok(pr)
    }

    pub async fn create_pr(
        client: &ProviderClient,
        repo: &str,
        title: &str,
        head: &str,
        base: &str,
        body: Option<&str>,
        draft: bool,
    ) -> Result<PullRequest, GitfleetError> {
        let endpoint = repo_path(repo, &["pulls"]);

        let mut json = serde_json::json!({
            "title": title,
            "head": head,
            "base": base,
            "draft": draft,
        });

        if let Some(b) = body {
            json["body"] = serde_json::Value::String(b.to_string());
        }

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(json), None, None)
            .await?;

        let pr: PullRequest = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create pull request: {e}")))?;

        Ok(pr)
    }

    pub async fn list(
        client: &ProviderClient,
        repo: &str,
        state: &str,
        limit: u32,
        page: Option<u32>,
        base: Option<&str>,
        head: Option<&str>,
    ) -> Result<Vec<PullRequest>, GitfleetError> {
        let enc_state = urlencoding::encode(state);
        let page = page.unwrap_or(1);

        let mut endpoint = format!(
            "{}?state={enc_state}&per_page={limit}&page={page}",
            repo_path(repo, &["pulls"])
        );

        if let Some(b) = base {
            let enc = urlencoding::encode(b);
            endpoint.push_str(&format!("&base={enc}"));
        }

        if let Some(h) = head {
            let enc = urlencoding::encode(h);
            endpoint.push_str(&format!("&head={enc}"));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let prs: Vec<PullRequest> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list pull requests: {e}")))?;

        Ok(prs)
    }

    pub async fn update_pr(
        client: &ProviderClient,
        repo: &str,
        number: u64,
        options: serde_json::Value,
    ) -> Result<PullRequest, GitfleetError> {
        let endpoint = repo_path(repo, &["pulls", &number.to_string()]);

        let response = client
            .request_token_required(reqwest::Method::PATCH, &endpoint, Some(options), None, None)
            .await?;

        let pr: PullRequest = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update pull request: {e}")))?;

        Ok(pr)
    }

    pub async fn merge(
        client: &ProviderClient,
        repo: &str,
        number: u64,
        method: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["pulls", &number.to_string(), "merge"]);

        let merge_method = match method {
            "squash" => "squash",
            "rebase" => "rebase",
            _ => "merge",
        };

        let body = serde_json::json!({ "merge_method": merge_method });

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to merge pull request: {e}")))?;

        Ok(data)
    }

    pub async fn comment(
        client: &ProviderClient,
        repo: &str,
        number: u64,
        body: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["issues", &number.to_string(), "comments"]);

        let json = serde_json::json!({ "body": body });
        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(json), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to comment: {e}")))?;

        Ok(data)
    }

    pub async fn lock(
        client: &ProviderClient,
        repo: &str,
        number: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["issues", &number.to_string(), "lock"]);

        client
            .request_token_required(
                reqwest::Method::PUT,
                &endpoint,
                Some(serde_json::json!({})),
                None,
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn unlock(
        client: &ProviderClient,
        repo: &str,
        number: u64,
    ) -> Result<(), GitfleetError> {
        let endpoint = repo_path(repo, &["issues", &number.to_string(), "lock"]);

        client
            .request_token_required(reqwest::Method::DELETE, &endpoint, None, None, None)
            .await?;

        Ok(())
    }
}
