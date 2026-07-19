use gitfleet_core::errors::GitfleetError;

use crate::github::api::path::repo_path;
use crate::github::client::ProviderClient;

pub struct IssuesApi;

impl IssuesApi {
    pub async fn get(
        client: &ProviderClient,
        issue_number: u64,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["issues", &issue_number.to_string()]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to fetch issue: {e}")))?;

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        repo: &str,
        title: &str,
        body: Option<&str>,
        labels: &[String],
        assignees: &[String],
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["issues"]);

        let mut json = serde_json::json!({ "title": title });

        if let Some(b) = body {
            json["body"] = serde_json::Value::String(b.to_string());
        }

        if !labels.is_empty() {
            json["labels"] = serde_json::Value::Array(
                labels
                    .iter()
                    .map(|l| serde_json::Value::String(l.clone()))
                    .collect(),
            );
        }

        if !assignees.is_empty() {
            json["assignees"] = serde_json::Value::Array(
                assignees
                    .iter()
                    .map(|a| serde_json::Value::String(a.clone()))
                    .collect(),
            );
        }

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(json), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create issue: {e}")))?;

        Ok(data)
    }

    pub async fn list(
        client: &ProviderClient,
        repo: &str,
        state: &str,
        limit: u32,
        page: Option<u32>,
        labels: &[String],
        assignees: &[String],
    ) -> Result<serde_json::Value, GitfleetError> {
        let mut qualifiers = vec![format!("repo:{repo}"), "type:issue".to_string()];

        if state != "all" {
            qualifiers.push(format!("state:{state}"));
        }

        for label in labels {
            qualifiers.push(format!("label:\"{label}\""));
        }

        for assignee in assignees {
            qualifiers.push(format!("assignee:\"{assignee}\""));
        }

        let query = qualifiers.join(" ");

        let page = page.unwrap_or(1);
        let endpoint = format!(
            "/search/issues?q={}&sort=updated&order=desc&per_page={limit}&page={page}",
            urlencoding::encode(&query)
        );

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list issues: {e}")))?;

        Ok(data)
    }

    pub async fn update(
        client: &ProviderClient,
        issue_number: u64,
        repo: &str,
        options: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["issues", &issue_number.to_string()]);

        let response = client
            .request_token_required(reqwest::Method::PATCH, &endpoint, Some(options), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update issue: {e}")))?;

        Ok(data)
    }

    pub async fn comment(
        client: &ProviderClient,
        issue_number: u64,
        repo: &str,
        body: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["issues", &issue_number.to_string(), "comments"]);

        let json = serde_json::json!({ "body": body });
        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(json), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to comment on issue: {e}")))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    fn build_issue_json(
        title: &str,
        body: Option<&str>,
        labels: &[String],
        assignees: &[String],
    ) -> serde_json::Value {
        let mut json = serde_json::json!({ "title": title });

        if let Some(b) = body {
            json["body"] = serde_json::Value::String(b.to_string());
        }

        if !labels.is_empty() {
            json["labels"] = serde_json::Value::Array(
                labels
                    .iter()
                    .map(|l| serde_json::Value::String(l.clone()))
                    .collect(),
            );
        }

        if !assignees.is_empty() {
            json["assignees"] = serde_json::Value::Array(
                assignees
                    .iter()
                    .map(|a| serde_json::Value::String(a.clone()))
                    .collect(),
            );
        }

        json
    }

    #[test]
    fn test_build_issue_json_title_only() {
        let json = build_issue_json("Bug report", None, &[], &[]);

        assert_eq!(json["title"], "Bug report");

        assert!(json.get("body").is_none());
        assert!(json.get("labels").is_none());
    }

    #[test]
    fn test_build_issue_json_full() {
        let json = build_issue_json(
            "Fix",
            Some("Description"),
            &["bug".to_string()],
            &["user1".to_string()],
        );

        assert_eq!(json["title"], "Fix");

        assert_eq!(json["body"], "Description");
        assert_eq!(json["labels"][0], "bug");

        assert_eq!(json["assignees"][0], "user1");
    }
}
