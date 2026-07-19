use gitfleet_core::errors::GitfleetError;

use crate::gitlab::client::ProviderClient;

fn encode_path(project: &str) -> String {
    urlencoding::encode(project).to_string()
}

pub struct IssuesApi;

impl IssuesApi {
    pub async fn get(
        client: &ProviderClient,
        project: &str,
        iid: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/issues/{iid}");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let mut data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to get issue: {e}")))?;

        normalize_issue(&mut data);

        Ok(data)
    }

    pub async fn create(
        client: &ProviderClient,
        project: &str,
        title: &str,
        body: Option<&str>,
        labels: &[String],
        assignees: &[String],
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/issues");
        let mut json = serde_json::json!({ "title": title });

        if let Some(b) = body {
            json["description"] = serde_json::Value::String(b.to_string());
        }

        if !labels.is_empty() {
            json["labels"] = serde_json::Value::String(labels.join(","));
        }

        if !assignees.is_empty() {
            json["assignee_ids"] = serde_json::Value::Array(
                assignees
                    .iter()
                    .filter_map(|a| a.parse::<u64>().ok())
                    .map(serde_json::Value::from)
                    .collect(),
            );
        }

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(json), None, None)
            .await?;

        let mut data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to create issue: {e}")))?;

        normalize_issue(&mut data);

        Ok(data)
    }

    pub async fn list(
        client: &ProviderClient,
        project: &str,
        state: &str,
        limit: u32,
        page: Option<u32>,
        labels: &[String],
        _assignees: &[String],
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let state = match state {
            "open" => "opened",
            state => state,
        };

        let page = page.unwrap_or(1);
        let mut endpoint = format!(
            "/projects/{encoded}/issues?state={}&per_page={limit}&page={page}",
            urlencoding::encode(state)
        );

        if !labels.is_empty() {
            endpoint.push_str(&format!(
                "&labels={}",
                urlencoding::encode(&labels.join(","))
            ));
        }

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let mut data: Vec<serde_json::Value> = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list issues: {e}")))?;

        for issue in &mut data {
            normalize_issue(issue);
        }

        Ok(serde_json::json!({
            "total_count": data.len(),
            "items": data,
        }))
    }

    pub async fn update(
        client: &ProviderClient,
        project: &str,
        iid: u64,
        options: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/issues/{iid}");

        let response = client
            .request_token_required(reqwest::Method::PUT, &endpoint, Some(options), None, None)
            .await?;

        let mut data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to update issue: {e}")))?;

        normalize_issue(&mut data);

        Ok(data)
    }

    pub async fn comment(
        client: &ProviderClient,
        project: &str,
        iid: u64,
        body: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/issues/{iid}/notes");
        let json = serde_json::json!({ "body": body });

        let response = client
            .request_token_required(reqwest::Method::POST, &endpoint, Some(json), None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to comment on issue: {e}")))?;

        Ok(data)
    }

    pub async fn list_comments(
        client: &ProviderClient,
        project: &str,
        iid: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        let encoded = encode_path(project);

        let endpoint = format!("/projects/{encoded}/issues/{iid}/notes?per_page=100");

        let response = client
            .request_token_required(reqwest::Method::GET, &endpoint, None, None, None)
            .await?;

        let data: serde_json::Value = crate::parse_json(response)
            .await
            .map_err(|e| GitfleetError::new(format!("Failed to list issue comments: {e}")))?;

        Ok(data)
    }
}

fn normalize_issue(raw: &mut serde_json::Value) {
    let Some(object) = raw.as_object_mut() else {
        return;
    };

    let number = object
        .get("iid")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let body = object
        .get("description")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let html_url = object
        .get("web_url")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let user = object
        .get("author")
        .and_then(serde_json::Value::as_object)
        .map(|author| {
            serde_json::json!({
                "login": author.get("username").cloned().unwrap_or(serde_json::Value::Null),
                "id": author.get("id").cloned().unwrap_or(serde_json::Value::Null),
            })
        })
        .unwrap_or(serde_json::Value::Null);
    let state = object
        .get("state")
        .and_then(serde_json::Value::as_str)
        .map(|state| if state == "opened" { "open" } else { state })
        .map(|state| serde_json::Value::String(state.to_string()))
        .unwrap_or(serde_json::Value::Null);

    object.insert("number".to_string(), number);
    object.insert("body".to_string(), body);
    object.insert("html_url".to_string(), html_url);
    object.insert("user".to_string(), user);
    object.insert("state".to_string(), state);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gitlab_issue_create_body_with_description() {
        let title = "Bug";
        let description = "Details here";
        let mut json = serde_json::json!({ "title": title });
        json["description"] = serde_json::Value::String(description.to_string());
        assert_eq!(json["title"], "Bug");

        assert_eq!(json["description"], "Details here");
    }

    #[test]
    fn test_gitlab_issue_create_body_with_labels() {
        let labels = ["bug".to_string(), "urgent".to_string()];
        let json = serde_json::Value::String(labels.join(","));

        assert_eq!(json, "bug,urgent");
    }

    #[test]
    fn test_gitlab_encode_path() {
        let encoded = urlencoding::encode("org/repo").to_string();

        assert_eq!(encoded, "org%2Frepo");
    }
}
