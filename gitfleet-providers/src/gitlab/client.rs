use gitfleet_core::constants::{
    GITLAB_API_BASE_URL, HTTP_TIMEOUT_SECONDS, MAX_HTTP_RESPONSE_BYTES, MAX_PAGINATION_ITEMS,
    MAX_PAGINATION_PAGES, STATUS_FORBIDDEN, STATUS_NOT_FOUND, STATUS_OK_MAX, STATUS_OK_MIN,
    STATUS_RATE_LIMITED, STATUS_UNAUTHORIZED, STATUS_UNPROCESSABLE,
};
use gitfleet_core::errors::{
    AuthError, GitfleetError, NotFoundError, RateLimitError, TokenRequiredError,
    UnprocessableError, UnsupportedCapabilityError,
};
use gitfleet_core::provider::{ProviderCapability, ProviderId};
use reqwest::Client;

const USER_AGENT: &str = "gitfleet/0.1.0";

pub struct ProviderClient {
    http: Client,
    base_url_override: Option<String>,
    configured_token: Option<String>,
}

impl Default for ProviderClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECONDS))
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECONDS))
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url_override: None,
            configured_token: None,
        }
    }

    pub fn with_base_url(base_url: &str) -> Self {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECONDS))
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECONDS))
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url_override: Some(base_url.to_string()),
            configured_token: None,
        }
    }

    pub fn with_host(host: &str) -> Self {
        Self::with_base_url(&format!("https://{host}/api/v4"))
    }

    pub fn with_context(host: &str, token: Option<String>) -> Self {
        let mut client = Self::with_host(host);
        client.configured_token = token;
        client
    }

    fn build_headers(
        &self,
        token: Option<&str>,
        content_type: Option<&str>,
    ) -> Result<reqwest::header::HeaderMap, GitfleetError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_str(content_type.unwrap_or("application/json"))
                .map_err(|e| GitfleetError::from(AuthError::new(format!("Invalid header: {e}"))))?,
        );

        if let Some(t) = token {
            headers.insert(
                "PRIVATE-TOKEN",
                reqwest::header::HeaderValue::from_str(t).map_err(|e| {
                    GitfleetError::from(AuthError::new(format!("Invalid token: {e}")))
                })?,
            );
        }

        Ok(headers)
    }

    fn api_base_url(&self, host: Option<&str>) -> String {
        if let Some(ref base) = self.base_url_override {
            return base.clone();
        }

        match host {
            Some(h) if h != "gitlab.com" => format!("https://{h}/api/v4"),
            _ => GITLAB_API_BASE_URL.to_string(),
        }
    }

    pub async fn request(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<serde_json::Value>,
        token: Option<&str>,
        host: Option<&str>,
    ) -> Result<reqwest::Response, GitfleetError> {
        crate::validate_relative_endpoint(endpoint)?;

        let url = format!("{}{endpoint}", self.api_base_url(host));

        self.request_url(method, &url, body, token).await
    }

    pub async fn request_url(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<serde_json::Value>,
        token: Option<&str>,
    ) -> Result<reqwest::Response, GitfleetError> {
        let mut attempt = 1;

        loop {
            let headers = self.build_headers(token, None)?;
            let mut req = self.http.request(method.clone(), url).headers(headers);

            if let Some(body) = body.as_ref() {
                req = req.json(body);
            }

            let response = match req.send().await {
                Ok(response) => response,
                Err(error) => {
                    if let Some(delay) = crate::retry::delay_for(&method, None, None, attempt) {
                        tracing::warn!(
                            provider = "gitlab",
                            method = %method,
                            attempt,
                            ?delay,
                            "retrying transient transport failure"
                        );

                        tokio::time::sleep(delay).await;
                        attempt += 1;
                        continue;
                    }

                    return Err(GitfleetError::new(format!(
                        "Network request failed: {error}"
                    )));
                }
            };

            let status = response.status().as_u16();

            if is_successful_response(&method, url, status) {
                validate_response_size(&response)?;

                return Ok(response);
            }

            if let Some(delay) =
                crate::retry::delay_for(&method, Some(status), Some(response.headers()), attempt)
            {
                tracing::warn!(
                    provider = "gitlab",
                    method = %method,
                    status,
                    attempt,
                    ?delay,
                    "retrying transient provider response"
                );

                tokio::time::sleep(delay).await;
                attempt += 1;
                continue;
            }

            return Err(handle_error(
                status,
                &response,
                token.is_some()
                    || gitfleet_core::config::get_provider_token_optional(
                        gitfleet_core::provider::ProviderId::GitLab,
                    )
                    .is_some(),
            ));
        }
    }

    pub async fn request_token_required(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<serde_json::Value>,
        token: Option<&str>,
        host: Option<&str>,
    ) -> Result<reqwest::Response, GitfleetError> {
        let effective_token = token
            .map(|token| token.to_string())
            .or_else(|| self.configured_token.clone());

        let effective_token = effective_token.or_else(|| {
            gitfleet_core::config::get_provider_token_optional(
                gitfleet_core::provider::ProviderId::GitLab,
            )
        });

        if effective_token.is_none() {
            return Err(GitfleetError::from(TokenRequiredError::new(
                "This operation requires a token with appropriate scopes.",
                vec![],
            )));
        }

        self.request(method, endpoint, body, effective_token.as_deref(), host)
            .await
    }

    pub async fn request_optional_token(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<serde_json::Value>,
        token: Option<&str>,
        host: Option<&str>,
    ) -> Result<reqwest::Response, GitfleetError> {
        let effective_token = token
            .map(|token| token.to_string())
            .or_else(|| self.configured_token.clone());

        let effective_token = effective_token.or_else(|| {
            gitfleet_core::config::get_provider_token_optional(
                gitfleet_core::provider::ProviderId::GitLab,
            )
        });

        self.request(method, endpoint, body, effective_token.as_deref(), host)
            .await
    }

    pub async fn get_paginated<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        token: Option<&str>,
        host: Option<&str>,
    ) -> Result<Vec<T>, GitfleetError> {
        let effective_token = token
            .map(|token| token.to_string())
            .or_else(|| self.configured_token.clone())
            .or_else(|| {
                gitfleet_core::config::get_provider_token_optional(
                    gitfleet_core::provider::ProviderId::GitLab,
                )
            });

        let base = self.api_base_url(host);
        let endpoint = with_default_per_page(endpoint);
        let mut url = format!("{base}{endpoint}");

        let mut all_items: Vec<T> = Vec::new();
        let mut pages = 0;

        loop {
            pages += 1;
            if pages > MAX_PAGINATION_PAGES {
                return Err(GitfleetError::new(
                    "Pagination exceeded the configured page limit.",
                ));
            }

            let response = self
                .request_url(reqwest::Method::GET, &url, None, effective_token.as_deref())
                .await?;

            let next_page = response
                .headers()
                .get("x-next-page")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| {
                    value.parse::<u32>().map_err(|_| {
                        GitfleetError::new(format!(
                            "Malformed GitLab pagination X-Next-Page header: {value}."
                        ))
                    })
                })
                .transpose()?;

            let items: Vec<T> = crate::parse_json(response).await.map_err(|e| {
                GitfleetError::new(format!("Failed to parse paginated response: {e}"))
            })?;

            if all_items.len().saturating_add(items.len()) > MAX_PAGINATION_ITEMS {
                return Err(GitfleetError::new(
                    "Pagination exceeded the configured item limit.",
                ));
            }

            all_items.extend(items);

            match next_page {
                Some(page) => {
                    url = replace_page_parameter(&url, page);
                }

                None => break,
            }
        }

        Ok(all_items)
    }

    pub fn is_ok(&self, status: u16) -> bool {
        (STATUS_OK_MIN..=STATUS_OK_MAX).contains(&status)
    }

    pub fn is_not_found(&self, status: u16) -> bool {
        status == STATUS_NOT_FOUND
    }
}

fn with_default_per_page(endpoint: &str) -> String {
    let has_per_page = endpoint
        .split('?')
        .nth(1)
        .map(|query| query.split('&').any(|part| part.starts_with("per_page=")))
        .unwrap_or(false);

    if has_per_page {
        endpoint.to_string()
    } else if endpoint.contains('?') {
        format!("{endpoint}&per_page=100")
    } else {
        format!("{endpoint}?per_page=100")
    }
}

fn validate_response_size(response: &reqwest::Response) -> Result<(), GitfleetError> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_HTTP_RESPONSE_BYTES as u64)
    {
        return Err(GitfleetError::new(
            "Provider response exceeded the configured size limit.",
        ));
    }

    Ok(())
}

fn is_successful_response(method: &reqwest::Method, url: &str, status: u16) -> bool {
    (STATUS_OK_MIN..=STATUS_OK_MAX).contains(&status)
        || (status == reqwest::StatusCode::NOT_MODIFIED.as_u16()
            && *method == reqwest::Method::POST
            && url.ends_with("/unstar"))
}

fn replace_page_parameter(url: &str, page: u32) -> String {
    let (base, query) = url.split_once('?').unwrap_or((url, ""));
    let mut found = false;

    let mut params: Vec<String> = query
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            if part.starts_with("page=") {
                found = true;
                format!("page={page}")
            } else {
                part.to_string()
            }
        })
        .collect();

    if !found {
        params.push(format!("page={page}"));
    }

    format!("{base}?{}", params.join("&"))
}

fn handle_error(status: u16, response: &reqwest::Response, has_token: bool) -> GitfleetError {
    if status == STATUS_UNAUTHORIZED && !has_token {
        return GitfleetError::from(TokenRequiredError::new(
            "This operation requires a token with appropriate scopes.",
            vec![],
        ));
    }

    if status == STATUS_RATE_LIMITED {
        let remaining = response
            .headers()
            .get("ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let limit = response
            .headers()
            .get("ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let reset = response
            .headers()
            .get("ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);

        let reset_at = time::OffsetDateTime::from_unix_timestamp(reset)
            .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);

        let message = if has_token {
            gitfleet_core::constants::ERROR_RATE_LIMIT_AUTHENTICATED
        } else {
            gitfleet_core::constants::ERROR_RATE_LIMIT_UNAUTHENTICATED
        };

        return GitfleetError::from(RateLimitError::new(message, reset_at, remaining, limit));
    }

    if status == STATUS_FORBIDDEN {
        let remaining = response
            .headers()
            .get("ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);

        if remaining == 0 {
            let limit = response
                .headers()
                .get("ratelimit-limit")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0);

            let reset = response
                .headers()
                .get("ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(0);

            let reset_at = time::OffsetDateTime::from_unix_timestamp(reset)
                .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);

            let message = if has_token {
                gitfleet_core::constants::ERROR_RATE_LIMIT_AUTHENTICATED
            } else {
                gitfleet_core::constants::ERROR_RATE_LIMIT_UNAUTHENTICATED
            };

            return GitfleetError::from(RateLimitError::new(message, reset_at, remaining, limit));
        }
    }

    match status {
        STATUS_UNAUTHORIZED => {
            GitfleetError::from(AuthError::new(gitfleet_core::constants::ERROR_UNAUTHORIZED))
        }

        STATUS_NOT_FOUND => GitfleetError::from(NotFoundError::new(
            gitfleet_core::constants::ERROR_NOT_FOUND,
        )),
        STATUS_UNPROCESSABLE => GitfleetError::from(UnprocessableError::new(
            gitfleet_core::constants::ERROR_UNPROCESSABLE,
        )),
        _ => GitfleetError::new(format!(
            "{}: {status}",
            gitfleet_core::constants::ERROR_UNEXPECTED
        )),
    }
}

macro_rules! impl_empty_ops {
    ($($trait:ident),*) => {
        $(
            impl gitfleet_core::provider::$trait for ProviderClient {}
        )*
    };
}

impl_empty_ops!();

#[async_trait::async_trait]
impl gitfleet_core::provider::DiscussionOps for ProviderClient {
    async fn list_discussions(
        &self,
        owner: &str,
        name: &str,
        category_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<gitfleet_core::types::Discussion>, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::DiscussionsApi::list(self, owner, name, category_id, limit).await
    }

    async fn get_discussion(
        &self,
        owner: &str,
        name: &str,
        discussion_number: u64,
    ) -> Result<gitfleet_core::types::Discussion, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::DiscussionsApi::get(self, owner, name, discussion_number).await
    }

    async fn create_discussion(
        &self,
        owner: &str,
        name: &str,
        title: &str,
        body: &str,
        category_id: Option<&str>,
    ) -> Result<gitfleet_core::types::Discussion, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::DiscussionsApi::create(self, owner, name, title, body, category_id)
            .await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::RepoOps for ProviderClient {
    async fn list_org_repos(
        &self,
        group: &str,
    ) -> Result<Vec<gitfleet_core::types::RepoSummary>, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::list_group(self, group).await
    }

    async fn list_user_repos(
        &self,
    ) -> Result<Vec<gitfleet_core::types::RepoSummary>, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::list_user(self).await
    }

    async fn list_user_named_repos(
        &self,
        username: &str,
    ) -> Result<Vec<gitfleet_core::types::RepoSummary>, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::list_user_named(self, username).await
    }

    async fn get_repo(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::get(self, repo).await
    }

    async fn create_repo(
        &self,
        name: &str,
        visibility: &str,
        owner: Option<&str>,
        owner_type: Option<&str>,
        description: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::create(
            self,
            name,
            visibility,
            owner,
            owner_type,
            description,
        )
        .await
    }

    async fn update_repo(
        &self,
        repo: &str,
        options: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::update(self, repo, options).await
    }

    async fn delete_repo(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::delete(self, repo).await
    }

    async fn star_repo(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::star(self, repo).await
    }

    async fn unstar_repo(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::unstar(self, repo).await
    }

    async fn fork_repo(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::fork(self, repo).await
    }

    async fn archive_repo(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::archive(self, repo).await
    }

    async fn unarchive_repo(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ProjectsApi::unarchive(self, repo).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::ChangeOps for ProviderClient {
    async fn list_changes(
        &self,
        repo: &str,
        state: &str,
        limit: u32,
        base: Option<&str>,
        head: Option<&str>,
    ) -> Result<Vec<gitfleet_core::types::PullRequest>, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MergeRequestsApi::list(self, repo, state, limit, base, head).await
    }

    async fn get_change(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<gitfleet_core::types::PullRequest, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MergeRequestsApi::get(self, repo, number).await
    }

    async fn create_change(
        &self,
        repo: &str,
        title: &str,
        head: &str,
        base: &str,
        body: Option<&str>,
        draft: bool,
    ) -> Result<gitfleet_core::types::PullRequest, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MergeRequestsApi::create(self, repo, title, head, base, body, draft)
            .await
    }

    async fn update_change(
        &self,
        repo: &str,
        number: u64,
        options: serde_json::Value,
    ) -> Result<gitfleet_core::types::PullRequest, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MergeRequestsApi::update(self, repo, number, options).await
    }

    async fn merge_change(
        &self,
        repo: &str,
        number: u64,
        method: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MergeRequestsApi::merge(self, repo, number, method).await
    }

    async fn comment_on_change(
        &self,
        repo: &str,
        number: u64,
        body: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MergeRequestsApi::comment(self, repo, number, body).await
    }

    async fn lock_change(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MergeRequestsApi::lock(self, repo, number).await
    }

    async fn unlock_change(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MergeRequestsApi::unlock(self, repo, number).await
    }

    async fn list_change_comments(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MergeRequestsApi::list_comments(self, repo, number).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::IssueOps for ProviderClient {
    async fn get_issue(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::IssuesApi::get(self, repo, number).await
    }

    async fn create_issue(
        &self,
        repo: &str,
        title: &str,
        body: Option<&str>,
        labels: &[String],
        assignees: &[String],
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::IssuesApi::create(self, repo, title, body, labels, assignees).await
    }

    async fn list_issues(
        &self,
        repo: &str,
        state: &str,
        limit: u32,
        labels: &[String],
        assignees: &[String],
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::IssuesApi::list(self, repo, state, limit, labels, assignees).await
    }

    async fn update_issue(
        &self,
        repo: &str,
        number: u64,
        options: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::IssuesApi::update(self, repo, number, options).await
    }

    async fn comment_on_issue(
        &self,
        repo: &str,
        number: u64,
        body: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::IssuesApi::comment(self, repo, number, body).await
    }

    async fn list_issue_comments(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::IssuesApi::list_comments(self, repo, number).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::PipelineOps for ProviderClient {
    async fn list_workflows(
        &self,
        repo: &str,
        limit: u32,
        page: Option<u32>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PipelinesApi::list_workflows(self, repo, limit, page).await
    }

    async fn get_workflow(
        &self,
        repo: &str,
        workflow_id: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PipelinesApi::get_workflow(self, repo, workflow_id).await
    }

    async fn dispatch_workflow(
        &self,
        repo: &str,
        workflow_id: &str,
        r#ref: &str,
        inputs: Option<serde_json::Value>,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PipelinesApi::dispatch_pipeline(self, repo, workflow_id, r#ref, inputs)
            .await
    }

    async fn list_runs(
        &self,
        repo: &str,
        filters: &str,
        limit: u32,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PipelinesApi::list_pipelines(self, repo, filters, limit).await
    }

    async fn get_run(
        &self,
        repo: &str,
        run_id: u64,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PipelinesApi::get_pipeline(self, repo, run_id).await
    }

    async fn cancel_run(
        &self,
        repo: &str,
        run_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PipelinesApi::cancel_pipeline(self, repo, run_id).await
    }

    async fn rerun(
        &self,
        repo: &str,
        run_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PipelinesApi::retry_pipeline(self, repo, run_id).await
    }

    async fn delete_run(
        &self,
        repo: &str,
        run_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PipelinesApi::delete_pipeline(self, repo, run_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::ReleaseOps for ProviderClient {
    async fn list_releases(
        &self,
        repo: &str,
        limit: u32,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ReleasesApi::list(self, repo, limit).await
    }

    async fn fetch_release_by_tag(
        &self,
        repo: &str,
        tag: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ReleasesApi::fetch_by_tag(self, repo, tag).await
    }

    async fn create_release(
        &self,
        repo: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ReleasesApi::create(self, repo, body).await
    }

    async fn update_release(
        &self,
        repo: &str,
        release: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ReleasesApi::update(self, repo, release, body).await
    }

    async fn delete_release(
        &self,
        repo: &str,
        release: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ReleasesApi::delete(self, repo, release).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::WikiOps for ProviderClient {
    async fn list_wiki_pages(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::WikiPage>, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::WikisApi::list(self, repo).await
    }

    async fn get_wiki_page(
        &self,
        repo: &str,
        page: &str,
    ) -> Result<gitfleet_core::types::WikiPageContent, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::WikisApi::get_page(self, repo, page).await
    }

    async fn create_wiki_page(
        &self,
        repo: &str,
        title: &str,
        content: &str,
    ) -> Result<gitfleet_core::types::WikiPageContent, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::WikisApi::create_page(self, repo, title, content).await
    }

    async fn update_wiki_page(
        &self,
        repo: &str,
        page: &str,
        content: &str,
    ) -> Result<gitfleet_core::types::WikiPageContent, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::WikisApi::update_page(self, repo, page, content).await
    }

    async fn delete_wiki_page(
        &self,
        repo: &str,
        page: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::WikisApi::delete_page(self, repo, page).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::LabelOps for ProviderClient {
    async fn list_labels(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::Label>, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::LabelsApi::list(self, repo).await
    }

    async fn create_label(
        &self,
        label: &gitfleet_core::types::Label,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::LabelsApi::create(self, label, repo).await
    }

    async fn delete_label(
        &self,
        name: &str,
        repo: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::LabelsApi::delete(self, name, repo).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::NotificationOps for ProviderClient {
    async fn list_notifications(
        &self,
        all: bool,
        participating: bool,
        repo: Option<&str>,
    ) -> Result<Vec<gitfleet_core::types::Notification>, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::NotificationsApi::list(self, all, participating, repo).await
    }

    async fn mark_notifications_read(&self) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::NotificationsApi::mark_read(self).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::EnvironmentOps for ProviderClient {
    async fn list_environments(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<gitfleet_core::types::EnvironmentListResponse, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::EnvironmentsApi::list(self, owner, repo).await
    }

    async fn create_environment(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        wait_timer: Option<u32>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::EnvironmentsApi::create(self, owner, repo, name, wait_timer).await
    }

    async fn delete_environment(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::EnvironmentsApi::delete(self, owner, repo, name).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::RunnerOps for ProviderClient {
    async fn list_runners(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::RunnerSummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::RunnersApi::list(self, repo).await
    }

    async fn remove_runner(
        &self,
        repo: &str,
        runner_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::RunnersApi::remove(self, repo, runner_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::WebhookOps for ProviderClient {
    async fn list_webhooks(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::WebhookSummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::WebhooksApi::list(self, repo).await
    }

    async fn create_webhook(
        &self,
        repo: &str,
        input: serde_json::Value,
    ) -> Result<gitfleet_core::types::WebhookSummary, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::WebhooksApi::create(self, repo, input).await
    }

    async fn remove_webhook(
        &self,
        repo: &str,
        hook_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::WebhooksApi::remove(self, repo, hook_id).await
    }

    async fn test_webhook(
        &self,
        repo: &str,
        hook_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::WebhooksApi::test(self, repo, hook_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::AccessOps for ProviderClient {
    async fn invite_org_member(
        &self,
        org: &str,
        username: &str,
        role: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AccessApi::invite_group_member(self, org, username, role).await
    }

    async fn invite_collaborator(
        &self,
        owner: &str,
        repo: &str,
        username: &str,
        permission: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AccessApi::invite_member(self, owner, repo, username, permission).await
    }

    async fn list_org_members(
        &self,
        group: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AccessApi::list_group_members(self, group).await
    }

    async fn remove_org_member(
        &self,
        org: &str,
        username: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AccessApi::remove_member(self, org, username).await
    }

    async fn list_teams(
        &self,
        org: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AccessApi::list_teams(self, org).await
    }

    async fn create_team(
        &self,
        org: &str,
        name: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AccessApi::create_team(self, org, name, "", "closed").await
    }

    async fn list_team_members(
        &self,
        org: &str,
        team_slug: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AccessApi::list_team_members(self, org, team_slug).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::IdentityOps for ProviderClient {
    async fn list_ssh_keys(
        &self,
    ) -> Result<Vec<gitfleet_core::types::SshKeySummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::IdentityApi::list_ssh_keys(self).await
    }

    async fn add_ssh_key(
        &self,
        title: &str,
        key: &str,
    ) -> Result<gitfleet_core::types::SshKeySummary, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::IdentityApi::add_ssh_key(self, title, key).await
    }

    async fn delete_ssh_key(
        &self,
        key_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::IdentityApi::delete_ssh_key(self, key_id).await
    }

    async fn list_gpg_keys(
        &self,
    ) -> Result<Vec<gitfleet_core::types::GpgKeySummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::IdentityApi::list_gpg_keys(self).await
    }

    async fn add_gpg_key(
        &self,
        armored_key: &str,
    ) -> Result<gitfleet_core::types::GpgKeySummary, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::IdentityApi::add_gpg_key(self, armored_key).await
    }

    async fn delete_gpg_key(
        &self,
        key_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::IdentityApi::delete_gpg_key(self, key_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::SearchOps for ProviderClient {
    async fn search_issues(
        &self,
        query: &str,
        sort: Option<&str>,
        order: Option<&str>,
        limit: u32,
    ) -> Result<
        gitfleet_core::types::SearchResult<serde_json::Value>,
        gitfleet_core::errors::GitfleetError,
    > {
        crate::gitlab::api::SearchApi::search_issues(self, query, sort, order, limit).await
    }

    async fn search_repos(
        &self,
        query: &str,
        sort: Option<&str>,
        order: Option<&str>,
        limit: u32,
    ) -> Result<
        gitfleet_core::types::SearchResult<serde_json::Value>,
        gitfleet_core::errors::GitfleetError,
    > {
        crate::gitlab::api::SearchApi::search_projects(self, query, sort, order, limit).await
    }

    async fn search_code(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<
        gitfleet_core::types::SearchResult<serde_json::Value>,
        gitfleet_core::errors::GitfleetError,
    > {
        crate::gitlab::api::SearchApi::search_code(self, query, limit).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::CodeOps for ProviderClient {
    async fn get_file_contents(
        &self,
        repo: &str,
        path: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::CodeApi::file_contents(self, repo, path, r#ref).await
    }

    async fn search_code(
        &self,
        query: &str,
        repo: Option<&str>,
        language: Option<&str>,
        limit: u32,
    ) -> Result<Vec<gitfleet_core::types::CodeSearchResult>, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::CodeApi::search(self, query, repo, language, limit).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::TemplateOps for ProviderClient {
    async fn list_issue_templates(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::IssueTemplate>, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::TemplatesApi::list(self, repo).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::VariableOps for ProviderClient {
    async fn list_repo_variables(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<
        gitfleet_core::types::VariableListResponse<gitfleet_core::types::RepoVariable>,
        gitfleet_core::errors::GitfleetError,
    > {
        crate::gitlab::api::VariablesApi::list(self, owner, repo).await
    }

    async fn set_repo_variable(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        value: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::VariablesApi::set(self, owner, repo, name, value).await
    }

    async fn delete_repo_variable(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::VariablesApi::delete(self, owner, repo, name).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::BrowseOps for ProviderClient {
    async fn list_contents(
        &self,
        repo: &str,
        path: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::BrowsingApi::list_contents(self, repo, path).await
    }

    async fn file_contents(
        &self,
        repo: &str,
        path: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::BrowsingApi::file_contents(self, repo, path, r#ref).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::RawApiOps for ProviderClient {
    async fn raw_get(
        &self,
        endpoint: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::RawApi::get(self, endpoint).await
    }

    async fn raw_post(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::RawApi::post(self, endpoint, body).await
    }

    async fn raw_delete(
        &self,
        endpoint: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::RawApi::delete(self, endpoint).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::DeployOps for ProviderClient {
    async fn list_deployments(
        &self,
        repo: &str,
        environment: Option<&str>,
        limit: u32,
    ) -> Result<Vec<gitfleet_core::types::DeploymentSummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::DeployApi::list(self, repo, environment, limit).await
    }

    async fn create_deployment(
        &self,
        repo: &str,
        input: serde_json::Value,
    ) -> Result<gitfleet_core::types::DeploymentSummary, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::DeployApi::create(self, repo, input).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::AnalyticsOps for ProviderClient {
    async fn get_traffic_views(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AnalyticsApi::get_traffic_views(self, repo).await
    }

    async fn get_traffic_clones(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AnalyticsApi::get_traffic_clones(self, repo).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::GovernanceOps for ProviderClient {
    async fn list_rulesets(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::GovernanceApi::list_rulesets(self, repo).await
    }

    async fn create_ruleset(
        &self,
        repo: &str,
        input: &gitfleet_core::types::RulesetInput,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::GovernanceApi::create_ruleset(self, repo, input).await
    }

    async fn delete_ruleset(
        &self,
        repo: &str,
        ruleset_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::GovernanceApi::delete_ruleset(self, repo, ruleset_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::SecretOps for ProviderClient {
    async fn list_repo_secrets(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<
        gitfleet_core::types::SecretListResponse<gitfleet_core::types::RepoSecret>,
        gitfleet_core::errors::GitfleetError,
    > {
        crate::gitlab::api::SecretsApi::list_repo(self, owner, repo).await
    }

    async fn get_repo_public_key(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<gitfleet_core::types::PublicKeyResponse, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::SecretsApi::get_repo_public_key(self, owner, repo).await
    }

    async fn set_repo_secret(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        encrypted_value: &str,
        key_id: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::SecretsApi::set_repo(self, owner, repo, name, encrypted_value, key_id)
            .await
    }

    async fn delete_repo_secret(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::SecretsApi::delete_repo(self, owner, repo, name).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::LicenseOps for ProviderClient {
    async fn list_licenses(
        &self,
    ) -> Result<Vec<gitfleet_core::types::LicenseSummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::LicensesApi::list(self).await
    }

    async fn get_license(
        &self,
        key: &str,
    ) -> Result<gitfleet_core::types::LicenseDetail, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::LicensesApi::get(self, key).await
    }

    async fn repo_license(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::LicensesApi::repo_license(self, repo).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::DependencyOps for ProviderClient {
    async fn sbom(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::DependenciesApi::sbom(self, repo).await
    }

    async fn review_dependencies(
        &self,
        repo: &str,
        base: &str,
        head: &str,
    ) -> Result<
        Vec<gitfleet_core::types::DependencyReviewChange>,
        gitfleet_core::errors::GitfleetError,
    > {
        crate::gitlab::api::DependenciesApi::review(self, repo, base, head).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::AdvisoryOps for ProviderClient {
    async fn list_dependabot_alerts(
        &self,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AdvisoriesApi::list_dependabot_alerts(self, repo, state).await
    }

    async fn list_codeql_alerts(
        &self,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AdvisoriesApi::list_codeql_alerts(self, repo, state).await
    }

    async fn list_secret_scanning_alerts(
        &self,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AdvisoriesApi::list_secret_scanning_alerts(self, repo, state).await
    }

    async fn get_dependabot_alert(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AdvisoriesApi::get_alert(self, repo, number).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::AttestationOps for ProviderClient {
    async fn list_attestations(
        &self,
        repo: &str,
        subject_digest: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::AttestationsApi::list(self, repo, subject_digest).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::ReviewOps for ProviderClient {
    async fn list_reactions_for_issue(
        &self,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<gitfleet_core::types::ReactionSummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::ReviewApi::list(self, repo, issue_number).await
    }

    async fn create_reaction_for_issue(
        &self,
        repo: &str,
        issue_number: u64,
        content: &str,
    ) -> Result<gitfleet_core::types::ReactionSummary, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ReviewApi::create(self, repo, issue_number, content).await
    }

    async fn delete_reaction_for_issue(
        &self,
        repo: &str,
        issue_number: u64,
        reaction_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::ReviewApi::delete(self, repo, issue_number, reaction_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::PlanningOps for ProviderClient {
    async fn list_milestones(
        &self,
        repo: &str,
        state: Option<&str>,
        limit: u32,
    ) -> Result<Vec<gitfleet_core::types::Milestone>, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MilestonesApi::list(self, repo, state, limit).await
    }

    async fn create_milestone(
        &self,
        repo: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<gitfleet_core::types::Milestone, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MilestonesApi::create(self, repo, title, description).await
    }

    async fn get_milestone(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<gitfleet_core::types::Milestone, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MilestonesApi::get(self, repo, number).await
    }

    async fn update_milestone(
        &self,
        repo: &str,
        number: u64,
        input: serde_json::Value,
    ) -> Result<gitfleet_core::types::Milestone, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MilestonesApi::update(self, repo, number, input).await
    }

    async fn delete_milestone(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::MilestonesApi::delete(self, repo, number).await
    }

    async fn list_projects(
        &self,
        _owner: &str,
        _limit: u32,
    ) -> Result<Vec<gitfleet_core::types::ProjectSummary>, gitfleet_core::errors::GitfleetError>
    {
        Err(GitfleetError::from(UnsupportedCapabilityError::new(
            ProviderId::GitLab,
            ProviderCapability::Projects,
        )))
    }

    async fn get_project(
        &self,
        _project_id: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        Err(GitfleetError::from(UnsupportedCapabilityError::new(
            ProviderId::GitLab,
            ProviderCapability::Projects,
        )))
    }

    async fn create_project(
        &self,
        _owner: &str,
        _title: &str,
        _body: Option<&str>,
    ) -> Result<gitfleet_core::types::ProjectSummary, gitfleet_core::errors::GitfleetError> {
        Err(GitfleetError::from(UnsupportedCapabilityError::new(
            ProviderId::GitLab,
            ProviderCapability::Projects,
        )))
    }

    async fn delete_project(
        &self,
        _project_id: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        Err(GitfleetError::from(UnsupportedCapabilityError::new(
            ProviderId::GitLab,
            ProviderCapability::Projects,
        )))
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::SiteOps for ProviderClient {
    async fn get_pages(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::SiteApi::get_pages(self, repo).await
    }

    async fn create_pages(
        &self,
        repo: &str,
        source: &str,
        build_type: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::SiteApi::create_pages(self, repo, source, build_type).await
    }

    async fn remove_pages(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::SiteApi::remove_pages(self, repo).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::SnippetOps for ProviderClient {
    async fn list_snippets(
        &self,
        owner: &str,
    ) -> Result<Vec<gitfleet_core::types::GistSummary>, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::SnippetsApi::list(self, owner).await
    }

    async fn get_snippet(
        &self,
        gist_id: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::SnippetsApi::get(self, gist_id).await
    }

    async fn create_snippet(
        &self,
        description: &str,
        public: bool,
        files: serde_json::Value,
    ) -> Result<gitfleet_core::types::GistSummary, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::SnippetsApi::create(self, description, public, files).await
    }

    async fn delete_snippet(
        &self,
        gist_id: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::SnippetsApi::delete(self, gist_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::PolicyOps for ProviderClient {
    async fn get_branch_protection(
        &self,
        repo: &str,
        branch: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PolicyApi::get_branch_protection(self, repo, branch).await
    }

    async fn protect_branch(
        &self,
        repo: &str,
        branch: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PolicyApi::protect_branch(self, repo, branch, input).await
    }

    async fn unprotect_branch(
        &self,
        repo: &str,
        branch: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PolicyApi::unprotect_branch(self, repo, branch).await
    }

    async fn list_tag_protection(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::TagProtection>, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::PolicyApi::list_tag_protection(self, repo).await
    }

    async fn create_tag_protection(
        &self,
        repo: &str,
        pattern: &str,
    ) -> Result<gitfleet_core::types::TagProtection, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PolicyApi::create_tag_protection(self, repo, pattern).await
    }

    async fn delete_tag_protection(
        &self,
        repo: &str,
        identifier: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::PolicyApi::delete_tag_protection(self, repo, identifier).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::RegistryOps for ProviderClient {
    async fn list_packages(
        &self,
        owner: &str,
        package_type: Option<&str>,
        limit: u32,
    ) -> Result<Vec<gitfleet_core::types::PackageSummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::gitlab::api::RegistryApi::list(self, owner, package_type, limit).await
    }

    async fn get_package(
        &self,
        owner: &str,
        package_type: &str,
        package_name: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::gitlab::api::RegistryApi::get(self, owner, package_type, package_name).await
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::constants::GITLAB_API_BASE_URL;

    use super::*;

    #[test]
    fn test_gitlab_api_base_url_default() {
        let client = ProviderClient::new();

        let url = client.api_base_url(None);

        assert_eq!(url, GITLAB_API_BASE_URL);
    }

    #[test]
    fn test_gitlab_api_base_url_override() {
        let client = ProviderClient::with_base_url("http://localhost:8080");

        let url = client.api_base_url(None);

        assert_eq!(url, "http://localhost:8080");
    }

    #[test]
    fn test_gitlab_api_base_url_profile_host() {
        let client = ProviderClient::with_host("git.example.com");

        let url = client.api_base_url(None);

        assert_eq!(url, "https://git.example.com/api/v4");
    }

    #[test]
    fn test_gitlab_api_base_url_custom_host() {
        let client = ProviderClient::new();

        let url = client.api_base_url(Some("self-hosted.example.com"));

        assert_eq!(url, "https://self-hosted.example.com/api/v4");
    }

    #[test]
    fn test_gitlab_api_base_url_gitlab_com() {
        let client = ProviderClient::new();

        let url = client.api_base_url(Some("gitlab.com"));

        assert_eq!(url, GITLAB_API_BASE_URL);
    }

    #[test]
    fn test_gitlab_is_ok_true() {
        let client = ProviderClient::new();

        assert!(client.is_ok(200));

        assert!(client.is_ok(201));
        assert!(client.is_ok(204));

        assert!(client.is_ok(299));
    }

    #[test]
    fn test_gitlab_is_ok_false() {
        let client = ProviderClient::new();

        assert!(!client.is_ok(100));

        assert!(!client.is_ok(301));
        assert!(!client.is_ok(404));
    }

    #[test]
    fn test_gitlab_unstar_accepts_not_modified() {
        assert!(is_successful_response(
            &reqwest::Method::POST,
            "https://gitlab.example/api/v4/projects/org%2Frepo/unstar",
            304,
        ));
    }

    #[test]
    fn test_gitlab_does_not_accept_not_modified_for_other_requests() {
        assert!(!is_successful_response(
            &reqwest::Method::GET,
            "https://gitlab.example/api/v4/projects/org%2Frepo",
            304,
        ));
    }

    #[test]
    fn test_gitlab_is_not_found() {
        let client = ProviderClient::new();

        assert!(client.is_not_found(404));

        assert!(!client.is_not_found(200));
        assert!(!client.is_not_found(403));
    }

    #[test]
    fn test_gitlab_build_headers_with_token() {
        let client = ProviderClient::new();

        let headers = client.build_headers(Some("mytoken"), None).unwrap();

        assert_eq!(headers.get("PRIVATE-TOKEN").unwrap(), "mytoken");
    }

    #[test]
    fn test_gitlab_build_headers_without_token() {
        let client = ProviderClient::new();

        let headers = client.build_headers(None, None).unwrap();

        assert!(headers.get("PRIVATE-TOKEN").is_none());
    }

    #[test]
    fn test_gitlab_build_headers_custom_content_type() {
        let client = ProviderClient::new();

        let headers = client.build_headers(None, Some("text/plain")).unwrap();

        assert_eq!(
            headers.get(reqwest::header::CONTENT_TYPE).unwrap(),
            "text/plain"
        );
    }

    #[test]
    fn test_gitlab_build_headers_default_content_type() {
        let client = ProviderClient::new();

        let headers = client.build_headers(None, None).unwrap();

        assert_eq!(
            headers.get(reqwest::header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_gitlab_build_headers_accept() {
        let client = ProviderClient::new();

        let headers = client.build_headers(None, None).unwrap();

        assert_eq!(
            headers.get(reqwest::header::ACCEPT).unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_gitlab_handle_error_unauthorized_no_token() {
        let err = handle_error(
            STATUS_UNAUTHORIZED,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(401)
                    .body(String::new())
                    .unwrap(),
            ),
            false,
        );

        assert!(err.to_string().contains("token") || err.to_string().contains("Token"));
    }

    #[test]
    fn test_gitlab_handle_error_unauthorized_with_token() {
        let err = handle_error(
            STATUS_UNAUTHORIZED,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(401)
                    .body(String::new())
                    .unwrap(),
            ),
            true,
        );

        assert!(err.to_string().contains("nauthorized"));
    }

    #[test]
    fn test_gitlab_handle_error_not_found() {
        let err = handle_error(
            STATUS_NOT_FOUND,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(404)
                    .body(String::new())
                    .unwrap(),
            ),
            true,
        );

        assert!(err.to_string().to_lowercase().contains("not found"));
    }

    #[test]
    fn test_gitlab_handle_error_unprocessable() {
        let err = handle_error(
            STATUS_UNPROCESSABLE,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(422)
                    .body(String::new())
                    .unwrap(),
            ),
            true,
        );

        assert!(err.to_string().to_lowercase().contains("processable"));
    }

    #[test]
    fn test_gitlab_handle_error_rate_limited() {
        let err = handle_error(
            STATUS_RATE_LIMITED,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(429)
                    .header("ratelimit-remaining", "0")
                    .header("ratelimit-limit", "5000")
                    .header("ratelimit-reset", "0")
                    .body(String::new())
                    .unwrap(),
            ),
            true,
        );

        assert!(err.to_string().contains("Rate limit"));
    }

    #[test]
    fn test_gitlab_handle_error_forbidden_rate_limited() {
        let err = handle_error(
            STATUS_FORBIDDEN,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(403)
                    .header("ratelimit-remaining", "0")
                    .header("ratelimit-limit", "5000")
                    .header("ratelimit-reset", "0")
                    .body(String::new())
                    .unwrap(),
            ),
            true,
        );

        assert!(err.to_string().contains("Rate limit"));
    }

    #[test]
    fn test_gitlab_handle_error_forbidden_not_rate_limited() {
        let err = handle_error(
            STATUS_FORBIDDEN,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(403)
                    .body(String::new())
                    .unwrap(),
            ),
            true,
        );

        assert!(err.to_string().contains("Unexpected"));
    }

    #[test]
    fn test_gitlab_handle_error_unknown_status() {
        let err = handle_error(
            500,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(500)
                    .body(String::new())
                    .unwrap(),
            ),
            true,
        );

        assert!(err.to_string().contains("500"));
    }

    #[test]
    fn test_gitlab_default_trait() {
        let client = ProviderClient::default();

        let url = client.api_base_url(None);

        assert_eq!(url, GITLAB_API_BASE_URL);
    }

    #[test]
    fn test_gitlab_with_default_per_page_preserves_existing_value() {
        assert_eq!(
            with_default_per_page("/projects?per_page=25"),
            "/projects?per_page=25"
        );
    }

    #[test]
    fn test_gitlab_replace_page_parameter() {
        assert_eq!(
            replace_page_parameter("https://gitlab.example/projects?per_page=25&page=1", 2),
            "https://gitlab.example/projects?per_page=25&page=2"
        );
    }

    #[test]
    fn test_gitlab_replace_page_parameter_adds_page() {
        assert_eq!(
            replace_page_parameter("https://gitlab.example/projects?per_page=25", 2),
            "https://gitlab.example/projects?per_page=25&page=2"
        );
    }
}
