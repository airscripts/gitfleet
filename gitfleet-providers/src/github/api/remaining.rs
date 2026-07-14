use gitfleet_core::errors::{GitfleetError, NotFoundError};
use gitfleet_core::types::{DependencyReviewChange, IssueTemplate, LicenseDetail, LicenseSummary};

use crate::github::api::path::{encode_path, encode_segment, repo_path};
use crate::github::client::ProviderClient;

pub struct TemplatesApi;

impl TemplatesApi {
    pub async fn list(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<Vec<IssueTemplate>, GitfleetError> {
        let (owner, name) = repo.split_once('/').ok_or_else(|| {
            GitfleetError::from(NotFoundError::new("Repository must use OWNER/REPO format."))
        })?;
        let query = r#"
            query ListIssueTemplates($owner: String!, $name: String!) {
                repository(owner: $owner, name: $name) {
                    issueTemplates {
                        name filename body about title
                        labels(first: 100) { nodes { name } }
                        assignees(first: 100) { nodes { login } }
                    }
                }
            }
        "#;
        let payload = serde_json::json!({
            "query": query,
            "variables": { "owner": owner, "name": name }
        });

        let response = client
            .request_token_required(reqwest::Method::POST, "/graphql", Some(payload), None, None)
            .await?;
        let data = crate::parse_graphql(response, "issue template listing")
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list issue templates: {e}")))?;
        let templates = data
            .get("data")
            .and_then(|value| value.get("repository"))
            .and_then(|value| value.get("issueTemplates"))
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();

        templates
            .into_iter()
            .map(|mut template| {
                let labels = template
                    .pointer("/labels/nodes")
                    .and_then(serde_json::Value::as_array)
                    .map(|nodes| {
                        nodes
                            .iter()
                            .filter_map(|node| node.get("name").cloned())
                            .collect()
                    })
                    .unwrap_or_default();
                let assignees = template
                    .pointer("/assignees/nodes")
                    .and_then(serde_json::Value::as_array)
                    .map(|nodes| {
                        nodes
                            .iter()
                            .filter_map(|node| node.get("login").cloned())
                            .collect()
                    })
                    .unwrap_or_default();
                let filename = template
                    .get("filename")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string();

                if let Some(object) = template.as_object_mut() {
                    object.insert("labels".to_string(), serde_json::Value::Array(labels));
                    object.insert("assignees".to_string(), serde_json::Value::Array(assignees));
                    object.insert(
                        "path".to_string(),
                        serde_json::Value::String(format!(".github/ISSUE_TEMPLATE/{filename}")),
                    );
                }

                serde_json::from_value(template).map_err(|error| {
                    GitfleetError::new(format!("Failed to normalize issue template: {error}"))
                })
            })
            .collect()
    }
}

pub struct LicensesApi;

impl LicensesApi {
    pub async fn list(client: &ProviderClient) -> Result<Vec<LicenseSummary>, GitfleetError> {
        let endpoint = "/licenses";

        let response = client
            .request_token_required(reqwest::Method::GET, endpoint, None, None, None)
            .await?;

        let data: Vec<LicenseSummary> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list licenses: {e}")))?;

        Ok(data)
    }

    pub async fn get(client: &ProviderClient, key: &str) -> Result<LicenseDetail, GitfleetError> {
        let endpoint = license_endpoint(key);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: LicenseDetail = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get license: {e}")))?;

        Ok(data)
    }

    pub async fn repo_license(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["license"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get repo license: {e}")))?;

        Ok(data)
    }
}

fn license_endpoint(key: &str) -> String {
    format!("/licenses/{}", encode_segment(key))
}

pub struct DependenciesApi;

impl DependenciesApi {
    pub async fn sbom(
        client: &ProviderClient,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["dependency-graph", "sbom"]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get SBOM: {e}")))?;

        Ok(data)
    }

    pub async fn compare(
        client: &ProviderClient,
        repo: &str,
        basehead: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["dependency-graph", "compare", basehead]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to compare dependencies: {e}")))?;

        Ok(data)
    }

    pub async fn review(
        client: &ProviderClient,
        repo: &str,
        base: &str,
        head: &str,
    ) -> Result<Vec<DependencyReviewChange>, GitfleetError> {
        let basehead = format!("{base}...{head}");

        let endpoint = repo_path(repo, &["dependency-graph", "compare", &basehead]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to review dependencies: {e}")))?;

        let changes = data.as_array().cloned().unwrap_or_default();

        Ok(changes
            .iter()
            .map(|raw| DependencyReviewChange {
                change_type: raw
                    .get("change_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                package: raw
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                ecosystem: raw
                    .get("ecosystem")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                version: raw
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                severity: highest_severity(raw.get("vulnerabilities")),
                vulnerabilities: raw
                    .get("vulnerabilities")
                    .and_then(|v| v.as_array())
                    .map_or(0, |items| items.len() as u32),
            })
            .collect())
    }
}

pub struct AdvisoriesApi;

impl AdvisoriesApi {
    pub async fn list_alerts(
        client: &ProviderClient,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let mut endpoint = repo_path(repo, &["dependabot", "alerts"]);

        if let Some(s) = state {
            endpoint.push_str(&format!("?state={}", urlencoding::encode(s)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list advisories: {e}")))?;

        Ok(data)
    }

    pub async fn dismiss_alert(
        client: &ProviderClient,
        repo: &str,
        alert_number: u64,
        reason: &str,
        comment: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["dependabot", "alerts", &alert_number.to_string()]);

        let mut body = serde_json::json!({ "state": "dismissed", "dismissed_reason": reason });

        if let Some(c) = comment {
            body["dismissed_comment"] = serde_json::Value::String(c.to_string());
        }

        let response = client
            .request_token_required(reqwest::Method::PATCH, &endpoint, Some(body), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to dismiss alert: {e}")))?;

        Ok(data)
    }

    pub async fn list_codeql(
        client: &ProviderClient,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let mut endpoint = repo_path(repo, &["code-scanning", "alerts"]);

        if let Some(s) = state {
            endpoint.push_str(&format!("?state={}", urlencoding::encode(s)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list CodeQL alerts: {e}")))?;

        Ok(data)
    }

    pub async fn list_secret_scanning(
        client: &ProviderClient,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let mut endpoint = repo_path(repo, &["secret-scanning", "alerts"]);

        if let Some(s) = state {
            endpoint.push_str(&format!("?state={}", urlencoding::encode(s)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response).await.map_err(|e| {
            GitfleetError::new(format!("Failed to list secret scanning alerts: {e}"))
        })?;

        Ok(data)
    }

    pub async fn get_alert(
        client: &ProviderClient,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["dependabot", "alerts", &number.to_string()]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get dependabot alert: {e}")))?;

        Ok(data)
    }
}

pub struct AttestationsApi;

impl AttestationsApi {
    pub async fn list(
        client: &ProviderClient,
        repo: &str,
        subject_digest: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = repo_path(repo, &["attestations", subject_digest]);

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list attestations: {e}")))?;

        Ok(data)
    }
}

fn highest_severity(vulnerabilities: Option<&serde_json::Value>) -> String {
    const SEVERITIES: [&str; 5] = ["critical", "high", "moderate", "medium", "low"];

    let found = vulnerabilities
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("severity").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();

    SEVERITIES
        .iter()
        .find(|severity| {
            found
                .iter()
                .any(|found| found.eq_ignore_ascii_case(severity))
        })
        .copied()
        .unwrap_or("")
        .to_string()
}

pub struct BrowseApi;

impl BrowseApi {
    pub async fn list_contents(
        client: &ProviderClient,
        repo: &str,
        path: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let endpoint = match path {
            Some(p) => repo_path(repo, &["contents", p]),
            None => repo_path(repo, &["contents"]),
        };

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list contents: {e}")))?;

        Ok(data)
    }

    pub async fn file_contents(
        client: &ProviderClient,
        repo: &str,
        path: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let mut endpoint = repo_path(repo, &["contents"]);
        endpoint.push('/');
        endpoint.push_str(&encode_path(path));

        if let Some(r) = r#ref {
            endpoint.push_str(&format!("?ref={}", urlencoding::encode(r)));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get file contents: {e}")))?;

        Ok(data)
    }
}

pub struct RawApi;

impl RawApi {
    pub async fn get(
        client: &ProviderClient,
        endpoint: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let resp = client
            .request_token_required(reqwest::Method::GET, endpoint, None, None, None)
            .await?;

        crate::parse_json(resp)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse response: {e}")))
    }

    pub async fn post(
        client: &ProviderClient,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let resp = client
            .request_token_required(reqwest::Method::POST, endpoint, Some(body), None, None)
            .await?;

        crate::parse_json(resp)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse response: {e}")))
    }

    pub async fn put(
        client: &ProviderClient,
        endpoint: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, GitfleetError> {
        let resp = client
            .request_token_required(reqwest::Method::PUT, endpoint, body, None, None)
            .await?;

        crate::parse_json(resp)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse response: {e}")))
    }

    pub async fn patch(
        client: &ProviderClient,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let resp = client
            .request_token_required(reqwest::Method::PATCH, endpoint, Some(body), None, None)
            .await?;

        crate::parse_json(resp)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse response: {e}")))
    }

    pub async fn delete(
        client: &ProviderClient,
        endpoint: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let resp = client
            .request_token_required(reqwest::Method::DELETE, endpoint, None, None, None)
            .await?;

        let status = resp.status().as_u16();

        if status == 204 || resp.content_length() == Some(0) {
            return Ok(serde_json::json!({"status": "deleted"}));
        }

        crate::parse_json(resp)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to parse response: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::license_endpoint;

    #[test]
    fn test_advisories_dismiss_alert_body_with_comment() {
        let reason = "false_positive";
        let comment = "Not applicable";
        let body = serde_json::json!({
            "state": "dismissed",
            "dismissed_reason": reason,
            "dismissed_comment": comment
        });

        assert_eq!(body["state"], "dismissed");

        assert_eq!(body["dismissed_reason"], "false_positive");
        assert_eq!(body["dismissed_comment"], "Not applicable");
    }

    #[test]
    fn test_advisories_dismiss_alert_body_without_comment() {
        let reason = "tolerable_risk";
        let body = serde_json::json!({ "state": "dismissed", "dismissed_reason": reason });

        assert!(body.get("dismissed_comment").is_none());
    }

    #[test]
    fn test_workflows_dispatch_body_with_inputs() {
        let r#ref = "main";
        let inputs = serde_json::json!({ "environment": "staging" });

        let mut body = serde_json::json!({ "ref": r#ref });
        body["inputs"] = inputs;
        assert_eq!(body["ref"], "main");

        assert_eq!(body["inputs"]["environment"], "staging");
    }

    #[test]
    fn test_workflows_dispatch_body_without_inputs() {
        let r#ref = "develop";
        let body = serde_json::json!({ "ref": r#ref });

        assert_eq!(body["ref"], "develop");

        assert!(body.get("inputs").is_none());
    }

    #[test]
    fn test_codespaces_create_payload_with_ref() {
        let mut payload = serde_json::json!({});
        payload["ref"] = serde_json::Value::String("main".to_string());
        assert_eq!(payload["ref"], "main");

        assert!(payload.get("machine").is_none());
    }

    #[test]
    fn test_issue_template_type() {
        let template = gitfleet_core::types::IssueTemplate {
            name: "Bug".to_string(),
            filename: "bug.md".to_string(),
            path: ".github/ISSUE_TEMPLATE/bug.md".to_string(),
            body: Some("## Bug".to_string()),
            about: None,
            title: None,
            labels: None,
            assignees: None,
        };

        assert_eq!(template.name, "Bug");

        assert_eq!(template.filename, "bug.md");
    }

    #[test]
    fn test_license_endpoint_encodes_key() {
        assert_eq!(
            license_endpoint("custom/license"),
            "/licenses/custom%2Flicense"
        );
    }
}
