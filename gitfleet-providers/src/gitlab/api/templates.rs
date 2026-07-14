use gitfleet_core::errors::GitfleetError;
use gitfleet_core::types::IssueTemplate;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct TemplatesApi;

impl TemplatesApi {
    pub async fn list(
        client: &ProviderClient,
        project: &str,
    ) -> Result<Vec<IssueTemplate>, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/templates/issues");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await;

        let response = match response {
            Ok(response) => response,
            Err(GitfleetError::NotFound(_)) => return Ok(vec![]),
            Err(error) => return Err(error),
        };

        let data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list issue templates: {e}")))?;

        Ok(data
            .iter()
            .map(|raw| {
                let key = raw
                    .get("key")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                let filename = format!("{key}.md");

                IssueTemplate {
                    name: raw
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    filename: filename.clone(),
                    path: format!(".gitlab/issue_templates/{filename}"),
                    body: raw
                        .get("content")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    about: None,
                    title: None,
                    labels: None,
                    assignees: None,
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::types::IssueTemplate;

    fn normalize_gitlab_template(raw: &serde_json::Value) -> IssueTemplate {
        IssueTemplate {
            name: raw
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            filename: raw
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            path: raw
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            body: raw
                .get("content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            about: None,
            title: None,
            labels: None,
            assignees: None,
        }
    }

    #[test]
    fn test_normalize_gitlab_template_full() {
        let json = serde_json::json!({
            "name": "Bug Report",
            "filename": "bug_report.md",
            "path": ".gitlab/issue_templates/bug_report.md",
            "content": "## Description\n\n## Steps"
        });

        let result = normalize_gitlab_template(&json);

        assert_eq!(result.name, "Bug Report");

        assert_eq!(result.filename, "bug_report.md");
        assert_eq!(result.path, ".gitlab/issue_templates/bug_report.md");

        assert_eq!(result.body, Some("## Description\n\n## Steps".to_string()));
    }

    #[test]
    fn test_normalize_gitlab_template_minimal() {
        let json = serde_json::json!({});

        let result = normalize_gitlab_template(&json);

        assert_eq!(result.name, "");

        assert_eq!(result.filename, "");
        assert!(result.body.is_none());
    }
}
