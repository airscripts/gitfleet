use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::PullRequest;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

fn normalize_mr(raw: &serde_json::Value) -> PullRequest {
    let author = raw.get("author").and_then(|v| v.as_object());
    PullRequest {
        title: raw
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        state: raw
            .get("state")
            .and_then(|v| v.as_str())
            .map(|state| if state == "opened" { "open" } else { state })
            .unwrap_or("")
            .to_string(),
        number: raw.get("iid").and_then(|v| v.as_u64()).unwrap_or(0),
        merged: raw
            .get("state")
            .and_then(|v| v.as_str())
            .map(|s| s == "merged")
            .unwrap_or(false),
        draft: Some(raw.get("draft").and_then(|v| v.as_bool()).unwrap_or(false)),
        html_url: raw
            .get("web_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        created_at: raw
            .get("created_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        updated_at: raw
            .get("updated_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        body: raw
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        mergeable_state: raw
            .get("detailed_merge_status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        merged_at: raw
            .get("merged_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        mergeable: None,
        user: author.map(|a| gitfleet_core::types::PullRequestUser {
            login: a
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        maintainer_can_modify: false,
        merge_commit_sha: raw
            .get("merge_commit_sha")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        labels: raw.get("labels").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    if v.is_string() {
                        Some(gitfleet_core::types::LabelEntry {
                            name: v.as_str().map(|s| s.to_string()),
                            color: None,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }),
        requested_reviewers: None,
        head: gitfleet_core::types::PullRequestHead {
            r#ref: raw
                .get("source_branch")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            sha: raw
                .get("sha")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            repo: None,
        },
        base: gitfleet_core::types::PullRequestBase {
            r#ref: raw
                .get("target_branch")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            repo: None,
        },
    }
}

fn merge_body(method: &str, sha: Option<&str>) -> serde_json::Value {
    let mut body = serde_json::json!({});

    if let Some(sha) = sha {
        body["sha"] = serde_json::Value::String(sha.to_string());
    }

    match method {
        "squash" => {
            body["squash"] = serde_json::Value::Bool(true);
        }
        "rebase" => {
            body["squash"] = serde_json::Value::Bool(false);
        }

        _ => {}
    }

    body
}

pub struct MergeRequestsApi;

impl MergeRequestsApi {
    pub async fn list(
        client: &ProviderClient,
        project: &str,
        state: &str,
        limit: u32,
        base: Option<&str>,
        head: Option<&str>,
    ) -> Result<Vec<PullRequest>, GitfleetError> {
        let encoded = encode_path(project);

        let state = match state {
            "open" => "opened",
            state => state,
        };

        let mut endpoint = format!(
            "/projects/{encoded}/merge_requests?state={}&per_page={limit}",
            urlencoding::encode(state)
        );

        if let Some(b) = base {
            endpoint.push_str(&format!("&target_branch={}", urlencoding::encode(b)));
        }

        if let Some(h) = head {
            endpoint.push_str(&format!("&source_branch={}", urlencoding::encode(h)));
        }

        let data: Vec<serde_json::Value> = client.get_paginated(&endpoint, None, None).await?;

        Ok(data.iter().map(normalize_mr).collect())
    }

    pub async fn get(
        client: &ProviderClient,
        project: &str,
        iid: u64,
    ) -> Result<PullRequest, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/merge_requests/{iid}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get merge request: {e}")))?;

        Ok(normalize_mr(&raw))
    }

    pub async fn create(
        client: &ProviderClient,
        project: &str,
        title: &str,
        head: &str,
        base: &str,
        body: Option<&str>,
        draft: bool,
    ) -> Result<PullRequest, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/merge_requests");
        let mut json = serde_json::json!({
            "title": title,
            "source_branch": head,
            "target_branch": base,
        });

        if let Some(b) = body {
            json["description"] = serde_json::Value::String(b.to_string());
        }

        if draft {
            json["draft"] = serde_json::Value::Bool(true);
        }

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(json), None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create merge request: {e}")))?;

        Ok(normalize_mr(&raw))
    }

    pub async fn update(
        client: &ProviderClient,
        project: &str,
        iid: u64,
        options: serde_json::Value,
    ) -> Result<PullRequest, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/merge_requests/{iid}");

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(options), None, None)
            .await?;

        let raw: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update merge request: {e}")))?;

        Ok(normalize_mr(&raw))
    }

    pub async fn merge(
        client: &ProviderClient,
        project: &str,
        iid: u64,
        method: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/merge_requests/{iid}/merge");
        let merge_request = Self::get(client, project, iid).await?;
        let body = merge_body(method, merge_request.head.sha.as_deref());

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to merge: {e}")))?;

        Ok(data)
    }

    pub async fn comment(
        client: &ProviderClient,
        project: &str,
        iid: u64,
        body: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/merge_requests/{iid}/notes");
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
        project: &str,
        iid: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/merge_requests/{iid}");

        let body = serde_json::json!({ "discussion_locked": true });

        client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn unlock(
        client: &ProviderClient,
        project: &str,
        iid: u64,
    ) -> Result<(), GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/merge_requests/{iid}");

        let body = serde_json::json!({ "discussion_locked": false });

        client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(body), None, None)
            .await?;

        Ok(())
    }

    pub async fn list_comments(
        client: &ProviderClient,
        project: &str,
        iid: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/merge_requests/{iid}/notes?per_page=100");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list MR comments: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_mr_full() {
        let json = serde_json::json!({
            "title": "Fix login bug",
            "state": "merged",
            "iid": 7,
            "draft": false,
            "web_url": "https://gitlab.com/group/project/-/merge_requests/7",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "description": "This fixes the login bug",
            "detailed_merge_status": "mergeable",
            "merged_at": "2024-01-03T00:00:00Z",
            "author": { "username": "dev1" },
            "merge_commit_sha": "abc123",
            "labels": ["bug", "urgent"],
            "source_branch": "fix-login",
            "target_branch": "main",
            "sha": "def456"
        });

        let result = normalize_mr(&json);

        assert_eq!(result.title, "Fix login bug");

        assert_eq!(result.state, "merged");
        assert_eq!(result.number, 7);

        assert!(result.merged);
        assert!(!result.draft.unwrap());

        assert_eq!(
            result.html_url,
            Some("https://gitlab.com/group/project/-/merge_requests/7".to_string())
        );

        assert_eq!(result.body, Some("This fixes the login bug".to_string()));

        assert!(result.merged_at.is_some());
        assert_eq!(result.user.unwrap().login, "dev1");

        assert_eq!(result.head.r#ref, "fix-login");
        assert_eq!(result.head.sha, Some("def456".to_string()));
        assert_eq!(result.base.r#ref, "main");

        assert_eq!(result.labels.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_normalize_mr_minimal() {
        let json = serde_json::json!({
            "title": "MR",
            "state": "opened",
            "iid": 1,
            "source_branch": "feature",
            "target_branch": "develop"
        });

        let result = normalize_mr(&json);

        assert_eq!(result.title, "MR");

        assert_eq!(result.state, "open");
        assert!(!result.merged);

        assert_eq!(result.number, 1);
        assert_eq!(result.head.r#ref, "feature");

        assert_eq!(result.base.r#ref, "develop");
    }

    #[test]
    fn test_merge_body_includes_sha() {
        let body = merge_body("merge", Some("abc123"));

        assert_eq!(body["sha"], "abc123");
        assert!(body.get("squash").is_none());
    }

    #[test]
    fn test_merge_body_preserves_squash_method() {
        let body = merge_body("squash", Some("abc123"));

        assert_eq!(body["sha"], "abc123");
        assert_eq!(body["squash"], true);
    }
}
