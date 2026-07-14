use gitfleet_core::constants::{
    GITHUB_API_ACCEPT, GITHUB_API_BASE_URL, GITHUB_API_VERSION, HTTP_TIMEOUT_SECONDS,
    MAX_HTTP_RESPONSE_BYTES, MAX_PAGINATION_ITEMS, MAX_PAGINATION_PAGES, STATUS_FORBIDDEN,
    STATUS_NOT_FOUND, STATUS_OK_MAX, STATUS_OK_MIN, STATUS_RATE_LIMITED, STATUS_UNAUTHORIZED,
    STATUS_UNPROCESSABLE,
};
use gitfleet_core::errors::{
    AuthError, GitfleetError, NotFoundError, RateLimitError, TokenRequiredError, UnprocessableError,
};
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
        let base_url = if host == "github.com" {
            GITHUB_API_BASE_URL.to_string()
        } else {
            format!("https://{host}/api/v3")
        };

        Self::with_base_url(&base_url)
    }

    pub fn with_context(host: &str, token: Option<String>) -> Self {
        let mut client = Self::with_host(host);
        client.configured_token = token;
        client
    }

    fn effective_optional_token(&self, token: Option<&str>) -> Option<String> {
        token
            .map(|token| token.to_string())
            .or_else(|| self.configured_token.clone())
            .or_else(|| {
                gitfleet_core::config::get_provider_token_optional(
                    gitfleet_core::provider::ProviderId::GitHub,
                )
            })
    }

    fn build_headers(
        &self,
        token: Option<&str>,
        accept: Option<&str>,
        content_type: Option<&str>,
    ) -> Result<reqwest::header::HeaderMap, GitfleetError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_str(accept.unwrap_or(GITHUB_API_ACCEPT))
                .map_err(|e| GitfleetError::from(AuthError::new(format!("Invalid header: {e}"))))?,
        );

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_str(content_type.unwrap_or("application/json"))
                .map_err(|e| GitfleetError::from(AuthError::new(format!("Invalid header: {e}"))))?,
        );

        headers.insert(
            "X-GitHub-Api-Version",
            reqwest::header::HeaderValue::from_str(GITHUB_API_VERSION)
                .map_err(|e| GitfleetError::from(AuthError::new(format!("Invalid header: {e}"))))?,
        );

        if let Some(t) = token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {t}")).map_err(|e| {
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
            Some(h) if h != "github.com" => format!("https://{h}/api/v3"),
            _ => GITHUB_API_BASE_URL.to_string(),
        }
    }

    fn graphql_base_url(&self, host: Option<&str>) -> String {
        let api_base = self.api_base_url(host);

        api_base
            .strip_suffix("/api/v3")
            .map(|base| format!("{base}/api"))
            .unwrap_or(api_base)
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

        let base_url = if endpoint == "/graphql" {
            self.graphql_base_url(host)
        } else {
            self.api_base_url(host)
        };
        let url = format!("{base_url}{endpoint}");

        self.request_url(method, &url, body, token, None, None)
            .await
    }

    pub async fn request_url(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<serde_json::Value>,
        token: Option<&str>,
        accept: Option<&str>,
        content_type: Option<&str>,
    ) -> Result<reqwest::Response, GitfleetError> {
        let mut attempt = 1;

        loop {
            let headers = self.build_headers(token, accept, content_type)?;
            let mut req = self.http.request(method.clone(), url).headers(headers);

            if let Some(body) = body.as_ref() {
                req = req.json(body);
            }

            let response = match req.send().await {
                Ok(response) => response,
                Err(error) => {
                    if let Some(delay) = crate::retry::delay_for(&method, None, None, attempt) {
                        tracing::warn!(
                            provider = "github",
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

            if (STATUS_OK_MIN..=STATUS_OK_MAX).contains(&status) {
                validate_response_size(&response)?;

                return Ok(response);
            }

            if let Some(delay) =
                crate::retry::delay_for(&method, Some(status), Some(response.headers()), attempt)
            {
                tracing::warn!(
                    provider = "github",
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
                        gitfleet_core::provider::ProviderId::GitHub,
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
        let effective_token = self.effective_optional_token(token);

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
        let effective_token = self.effective_optional_token(token);

        self.request(method, endpoint, body, effective_token.as_deref(), host)
            .await
    }

    pub async fn get_paginated<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        token: Option<&str>,
        host: Option<&str>,
    ) -> Result<Vec<T>, GitfleetError> {
        let effective_token = self.effective_optional_token(token);

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
                .request_url(
                    reqwest::Method::GET,
                    &url,
                    None,
                    effective_token.as_deref(),
                    None,
                    None,
                )
                .await?;

            let link_header = response
                .headers()
                .get("link")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let items: Vec<T> = crate::parse_json(response).await.map_err(|e| {
                GitfleetError::new(format!("Failed to parse paginated response: {e}"))
            })?;

            if all_items.len().saturating_add(items.len()) > MAX_PAGINATION_ITEMS {
                return Err(GitfleetError::new(
                    "Pagination exceeded the configured item limit.",
                ));
            }

            all_items.extend(items);

            match link_header {
                Some(link_header) => match parse_next_link(&link_header)? {
                    Some(next) => url = validate_next_url(&base, &next)?,
                    None => break,
                },

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
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let limit = response
            .headers()
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let reset = response
            .headers()
            .get("x-ratelimit-reset")
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
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);

        if remaining == 0 {
            let limit = response
                .headers()
                .get("x-ratelimit-limit")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0);

            let reset = response
                .headers()
                .get("x-ratelimit-reset")
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

impl_empty_ops!(MergeAutomationOps);

#[async_trait::async_trait]
impl gitfleet_core::provider::RepoOps for ProviderClient {
    async fn list_org_repos(
        &self,
        org: &str,
    ) -> Result<Vec<gitfleet_core::types::RepoSummary>, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReposApi::fetch_org(self, org).await
    }

    async fn list_user_repos(
        &self,
    ) -> Result<Vec<gitfleet_core::types::RepoSummary>, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReposApi::fetch_user_repos(self).await
    }

    async fn list_user_named_repos(
        &self,
        username: &str,
    ) -> Result<Vec<gitfleet_core::types::RepoSummary>, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReposApi::fetch_user(self, username).await
    }

    async fn get_repo(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        let gh_repo = crate::github::api::ReposApi::get(self, repo).await?;
        serde_json::to_value(gh_repo).map_err(|e| {
            gitfleet_core::errors::GitfleetError::new(format!("Failed to serialize repo: {e}"))
        })
    }

    async fn create_repo(
        &self,
        name: &str,
        visibility: &str,
        owner: Option<&str>,
        owner_type: Option<&str>,
        description: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        let gh_repo = crate::github::api::ReposApi::create(
            self,
            name,
            visibility,
            owner,
            owner_type,
            description,
        )
        .await?;

        serde_json::to_value(gh_repo).map_err(|e| {
            gitfleet_core::errors::GitfleetError::new(format!("Failed to serialize repo: {e}"))
        })
    }

    async fn update_repo(
        &self,
        repo: &str,
        options: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        let gh_repo = crate::github::api::ReposApi::update(self, repo, options).await?;
        serde_json::to_value(gh_repo).map_err(|e| {
            gitfleet_core::errors::GitfleetError::new(format!("Failed to serialize repo: {e}"))
        })
    }

    async fn delete_repo(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReposApi::delete(self, repo).await
    }

    async fn star_repo(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReposApi::star(self, repo).await
    }

    async fn unstar_repo(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReposApi::unstar(self, repo).await
    }

    async fn fork_repo(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        let gh_repo = crate::github::api::ReposApi::fork(self, repo).await?;
        serde_json::to_value(gh_repo).map_err(|e| {
            gitfleet_core::errors::GitfleetError::new(format!("Failed to serialize repo: {e}"))
        })
    }

    async fn archive_repo(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReposApi::archive(self, repo).await
    }

    async fn unarchive_repo(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReposApi::unarchive(self, repo).await
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
        crate::github::api::PullsApi::list(self, repo, state, limit, base, head).await
    }

    async fn get_change(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<gitfleet_core::types::PullRequest, gitfleet_core::errors::GitfleetError> {
        crate::github::api::PullsApi::fetch(self, number, repo).await
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
        crate::github::api::PullsApi::create_pr(self, repo, title, head, base, body, draft).await
    }

    async fn update_change(
        &self,
        repo: &str,
        number: u64,
        options: serde_json::Value,
    ) -> Result<gitfleet_core::types::PullRequest, gitfleet_core::errors::GitfleetError> {
        crate::github::api::PullsApi::update_pr(self, repo, number, options).await
    }

    async fn merge_change(
        &self,
        repo: &str,
        number: u64,
        method: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::PullsApi::merge(self, repo, number, method).await
    }

    async fn comment_on_change(
        &self,
        repo: &str,
        number: u64,
        body: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::PullsApi::comment(self, repo, number, body).await
    }

    async fn lock_change(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::PullsApi::lock(self, repo, number).await
    }

    async fn unlock_change(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::PullsApi::unlock(self, repo, number).await
    }

    async fn list_change_comments(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::CommentsApi::list_pr_comments(self, repo, number)
            .await
            .map(|v| serde_json::to_value(v).unwrap_or_default())
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
        crate::github::api::ReactionsApi::list_for_issue(self, repo, issue_number).await
    }

    async fn create_reaction_for_issue(
        &self,
        repo: &str,
        issue_number: u64,
        content: &str,
    ) -> Result<gitfleet_core::types::ReactionSummary, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReactionsApi::create_for_issue(self, repo, issue_number, content).await
    }

    async fn delete_reaction_for_issue(
        &self,
        repo: &str,
        issue_number: u64,
        reaction_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReactionsApi::delete_for_issue(self, repo, issue_number, reaction_id)
            .await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::IssueOps for ProviderClient {
    async fn get_issue(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::IssuesApi::get(self, number, repo).await
    }

    async fn create_issue(
        &self,
        repo: &str,
        title: &str,
        body: Option<&str>,
        labels: &[String],
        assignees: &[String],
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::IssuesApi::create(self, repo, title, body, labels, assignees).await
    }

    async fn list_issues(
        &self,
        repo: &str,
        state: &str,
        limit: u32,
        labels: &[String],
        assignees: &[String],
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::IssuesApi::list(self, repo, state, limit, labels, assignees).await
    }

    async fn update_issue(
        &self,
        repo: &str,
        number: u64,
        options: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::IssuesApi::update(self, number, repo, options).await
    }

    async fn comment_on_issue(
        &self,
        repo: &str,
        number: u64,
        body: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::IssuesApi::comment(self, number, repo, body).await
    }

    async fn list_issue_comments(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::CommentsApi::list_issue_comments(self, repo, number)
            .await
            .map(|v| serde_json::to_value(v).unwrap_or_default())
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::LabelOps for ProviderClient {
    async fn list_labels(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::Label>, gitfleet_core::errors::GitfleetError> {
        crate::github::api::LabelsApi::fetch(self, repo).await
    }

    async fn create_label(
        &self,
        label: &gitfleet_core::types::Label,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::LabelsApi::create(self, label, repo).await
    }

    async fn delete_label(
        &self,
        name: &str,
        repo: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::LabelsApi::delete(self, name, repo).await
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
        crate::github::api::NotificationsApi::fetch(self, all, participating, repo).await
    }

    async fn mark_notifications_read(&self) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::NotificationsApi::mark_read(self).await
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
        crate::github::api::WorkflowsApi::list_workflows(self, repo, limit, page).await
    }

    async fn get_workflow(
        &self,
        repo: &str,
        workflow_id: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::WorkflowsApi::get_workflow(self, repo, workflow_id).await
    }

    async fn dispatch_workflow(
        &self,
        repo: &str,
        workflow_id: &str,
        r#ref: &str,
        inputs: Option<serde_json::Value>,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::WorkflowsApi::dispatch_workflow(self, repo, workflow_id, r#ref, inputs)
            .await
    }

    async fn list_runs(
        &self,
        repo: &str,
        filters: &str,
        limit: u32,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::WorkflowsApi::list_runs(self, repo, filters, limit).await
    }

    async fn get_run(
        &self,
        repo: &str,
        run_id: u64,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::WorkflowsApi::get_run(self, repo, run_id).await
    }

    async fn cancel_run(
        &self,
        repo: &str,
        run_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::WorkflowsApi::cancel_run(self, repo, run_id).await
    }

    async fn rerun(
        &self,
        repo: &str,
        run_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::WorkflowsApi::rerun(self, repo, run_id).await
    }

    async fn delete_run(
        &self,
        repo: &str,
        run_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::WorkflowsApi::delete_run(self, repo, run_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::ReleaseOps for ProviderClient {
    async fn list_releases(
        &self,
        repo: &str,
        limit: u32,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReleasesApi::list(self, repo, limit).await
    }

    async fn fetch_release_by_tag(
        &self,
        repo: &str,
        tag: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReleasesApi::fetch_by_tag(self, repo, tag).await
    }

    async fn create_release(
        &self,
        repo: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ReleasesApi::create(self, repo, body).await
    }

    async fn update_release(
        &self,
        repo: &str,
        release: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        let release_id = resolve_release_id(self, repo, release).await?;

        crate::github::api::ReleasesApi::update(self, repo, release_id, body).await
    }

    async fn delete_release(
        &self,
        repo: &str,
        release: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        let release_id = resolve_release_id(self, repo, release).await?;

        crate::github::api::ReleasesApi::delete(self, repo, release_id).await
    }
}

async fn resolve_release_id(
    client: &ProviderClient,
    repo: &str,
    release: &str,
) -> Result<u64, GitfleetError> {
    if let Ok(release_id) = release.parse::<u64>() {
        return Ok(release_id);
    }

    let data = crate::github::api::ReleasesApi::fetch_by_tag(client, repo, release).await?;

    data.get("id")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            GitfleetError::from(UnprocessableError::new(
                "GitHub release response did not include a numeric ID.",
            ))
        })
}

#[async_trait::async_trait]
impl gitfleet_core::provider::PlanningOps for ProviderClient {
    async fn list_milestones(
        &self,
        repo: &str,
        state: Option<&str>,
        limit: u32,
    ) -> Result<Vec<gitfleet_core::types::Milestone>, gitfleet_core::errors::GitfleetError> {
        crate::github::api::MilestonesApi::list(self, repo, state.unwrap_or("open"), limit).await
    }

    async fn create_milestone(
        &self,
        repo: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<gitfleet_core::types::Milestone, gitfleet_core::errors::GitfleetError> {
        crate::github::api::MilestonesApi::create(self, repo, title, description).await
    }

    async fn get_milestone(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<gitfleet_core::types::Milestone, gitfleet_core::errors::GitfleetError> {
        crate::github::api::MilestonesApi::get(self, repo, number).await
    }

    async fn update_milestone(
        &self,
        repo: &str,
        number: u64,
        input: serde_json::Value,
    ) -> Result<gitfleet_core::types::Milestone, gitfleet_core::errors::GitfleetError> {
        crate::github::api::MilestonesApi::update(self, repo, number, input).await
    }

    async fn delete_milestone(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::MilestonesApi::delete(self, repo, number).await
    }

    async fn list_projects(
        &self,
        owner: &str,
        limit: u32,
    ) -> Result<Vec<gitfleet_core::types::ProjectSummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::github::api::ProjectsApi::list(self, owner, limit).await
    }

    async fn get_project(
        &self,
        project_id: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ProjectsApi::get_by_id(self, project_id).await
    }

    async fn create_project(
        &self,
        owner: &str,
        title: &str,
        _body: Option<&str>,
    ) -> Result<gitfleet_core::types::ProjectSummary, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ProjectsApi::create(self, owner, title).await
    }

    async fn delete_project(
        &self,
        project_id: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::ProjectsApi::delete(self, project_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::WikiOps for ProviderClient {
    async fn list_wiki_pages(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::WikiPage>, gitfleet_core::errors::GitfleetError> {
        crate::github::api::WikiApi::list(self, repo).await
    }

    async fn get_wiki_page(
        &self,
        repo: &str,
        page: &str,
    ) -> Result<gitfleet_core::types::WikiPageContent, gitfleet_core::errors::GitfleetError> {
        crate::github::api::WikiApi::get_page(self, repo, page).await
    }

    async fn create_wiki_page(
        &self,
        repo: &str,
        title: &str,
        content: &str,
    ) -> Result<gitfleet_core::types::WikiPageContent, gitfleet_core::errors::GitfleetError> {
        crate::github::api::WikiApi::create_page(self, repo, title, content).await
    }

    async fn update_wiki_page(
        &self,
        repo: &str,
        page: &str,
        content: &str,
    ) -> Result<gitfleet_core::types::WikiPageContent, gitfleet_core::errors::GitfleetError> {
        crate::github::api::WikiApi::update_page(self, repo, page, content).await
    }

    async fn delete_wiki_page(
        &self,
        repo: &str,
        page: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::WikiApi::delete_page(self, repo, page).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::SiteOps for ProviderClient {
    async fn get_pages(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::PagesApi::get(self, repo).await
    }

    async fn create_pages(
        &self,
        repo: &str,
        source: &str,
        build_type: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::PagesApi::create(self, repo, source, build_type).await
    }

    async fn remove_pages(&self, repo: &str) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::PagesApi::remove(self, repo).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::DiscussionOps for ProviderClient {
    async fn list_discussions(
        &self,
        owner: &str,
        name: &str,
        category_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<gitfleet_core::types::Discussion>, gitfleet_core::errors::GitfleetError> {
        crate::github::api::DiscussionsApi::list(self, owner, name, category_id, limit).await
    }

    async fn get_discussion(
        &self,
        owner: &str,
        name: &str,
        discussion_number: u64,
    ) -> Result<gitfleet_core::types::Discussion, gitfleet_core::errors::GitfleetError> {
        crate::github::api::DiscussionsApi::get(self, owner, name, discussion_number).await
    }

    async fn create_discussion(
        &self,
        owner: &str,
        name: &str,
        title: &str,
        body: &str,
        category_id: Option<&str>,
    ) -> Result<gitfleet_core::types::Discussion, gitfleet_core::errors::GitfleetError> {
        crate::github::api::DiscussionsApi::create(self, owner, name, title, body, category_id)
            .await
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
        crate::github::api::DeploymentsApi::list(self, repo, environment, limit).await
    }

    async fn create_deployment(
        &self,
        repo: &str,
        input: serde_json::Value,
    ) -> Result<gitfleet_core::types::DeploymentSummary, gitfleet_core::errors::GitfleetError> {
        crate::github::api::DeploymentsApi::create(self, repo, input).await
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
        crate::github::api::EnvironmentsApi::list(self, owner, repo).await
    }

    async fn create_environment(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        wait_timer: Option<u32>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::EnvironmentsApi::create(self, owner, repo, name, wait_timer).await
    }

    async fn delete_environment(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::EnvironmentsApi::delete(self, owner, repo, name).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::RunnerOps for ProviderClient {
    async fn list_runners(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::RunnerSummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::github::api::RunnersApi::list_repo(self, repo).await
    }

    async fn remove_runner(
        &self,
        repo: &str,
        runner_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::RunnersApi::remove_repo(self, repo, runner_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::WebhookOps for ProviderClient {
    async fn list_webhooks(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::WebhookSummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::github::api::WebhooksApi::list(self, repo).await
    }

    async fn create_webhook(
        &self,
        repo: &str,
        input: serde_json::Value,
    ) -> Result<gitfleet_core::types::WebhookSummary, gitfleet_core::errors::GitfleetError> {
        crate::github::api::WebhooksApi::create(self, repo, input).await
    }

    async fn remove_webhook(
        &self,
        repo: &str,
        hook_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::WebhooksApi::remove(self, repo, hook_id).await
    }

    async fn test_webhook(
        &self,
        repo: &str,
        hook_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::WebhooksApi::test(self, repo, hook_id).await
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
        crate::github::api::OrgsApi::invite_member(self, org, username, role).await
    }

    async fn invite_collaborator(
        &self,
        owner: &str,
        repo: &str,
        username: &str,
        permission: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::AccessApi::invite_collaborator(self, owner, repo, username, permission)
            .await
    }

    async fn list_org_members(
        &self,
        org: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::AccessApi::list_org_members(self, org).await
    }

    async fn remove_org_member(
        &self,
        org: &str,
        username: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::OrgsApi::remove_member(self, org, username).await
    }

    async fn list_teams(
        &self,
        org: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::TeamsApi::list(self, org).await
    }

    async fn create_team(
        &self,
        org: &str,
        name: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::TeamsApi::create(self, org, name, "", "closed").await
    }

    async fn list_team_members(
        &self,
        org: &str,
        team_slug: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::TeamsApi::list_members(self, org, team_slug).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::IdentityOps for ProviderClient {
    async fn list_ssh_keys(
        &self,
    ) -> Result<Vec<gitfleet_core::types::SshKeySummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::github::api::IdentityApi::list_ssh_keys(self).await
    }

    async fn add_ssh_key(
        &self,
        title: &str,
        key: &str,
    ) -> Result<gitfleet_core::types::SshKeySummary, gitfleet_core::errors::GitfleetError> {
        crate::github::api::IdentityApi::add_ssh_key(self, title, key).await
    }

    async fn delete_ssh_key(
        &self,
        key_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::IdentityApi::delete_ssh_key(self, key_id).await
    }

    async fn list_gpg_keys(
        &self,
    ) -> Result<Vec<gitfleet_core::types::GpgKeySummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::github::api::IdentityApi::list_gpg_keys(self).await
    }

    async fn add_gpg_key(
        &self,
        armored_key: &str,
    ) -> Result<gitfleet_core::types::GpgKeySummary, gitfleet_core::errors::GitfleetError> {
        crate::github::api::IdentityApi::add_gpg_key(self, armored_key).await
    }

    async fn delete_gpg_key(
        &self,
        key_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::IdentityApi::delete_gpg_key(self, key_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::AnalyticsOps for ProviderClient {
    async fn get_traffic_views(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::AnalyticsApi::get_traffic_views(self, repo).await
    }

    async fn get_traffic_clones(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::AnalyticsApi::get_traffic_clones(self, repo).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::SnippetOps for ProviderClient {
    async fn list_snippets(
        &self,
        owner: &str,
    ) -> Result<Vec<gitfleet_core::types::GistSummary>, gitfleet_core::errors::GitfleetError> {
        if owner.is_empty() {
            crate::github::api::GistsApi::list(self, false, 100).await
        } else {
            crate::github::api::GistsApi::list_for_user(self, owner).await
        }
    }

    async fn get_snippet(
        &self,
        gist_id: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::GistsApi::get_json(self, gist_id).await
    }

    async fn create_snippet(
        &self,
        description: &str,
        public: bool,
        files: serde_json::Value,
    ) -> Result<gitfleet_core::types::GistSummary, gitfleet_core::errors::GitfleetError> {
        crate::github::api::GistsApi::create(self, Some(description), public, files).await
    }

    async fn delete_snippet(
        &self,
        gist_id: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::GistsApi::delete(self, gist_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::GovernanceOps for ProviderClient {
    async fn list_rulesets(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::GovernanceApi::list_rulesets(self, repo).await
    }

    async fn create_ruleset(
        &self,
        repo: &str,
        input: &gitfleet_core::types::RulesetInput,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::GovernanceApi::create_ruleset(self, repo, input).await
    }

    async fn delete_ruleset(
        &self,
        repo: &str,
        ruleset_id: u64,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::GovernanceApi::delete_ruleset(self, repo, ruleset_id).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::PolicyOps for ProviderClient {
    async fn get_branch_protection(
        &self,
        repo: &str,
        branch: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ProtectionApi::get_branch_protection(self, repo, branch).await
    }

    async fn protect_branch(
        &self,
        repo: &str,
        branch: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ProtectionApi::protect_branch(self, repo, branch, input).await
    }

    async fn unprotect_branch(
        &self,
        repo: &str,
        branch: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::ProtectionApi::unprotect_branch(self, repo, branch).await
    }

    async fn list_tag_protection(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::TagProtection>, gitfleet_core::errors::GitfleetError>
    {
        crate::github::api::ProtectionApi::list_tag_protection(self, repo).await
    }

    async fn create_tag_protection(
        &self,
        repo: &str,
        pattern: &str,
    ) -> Result<gitfleet_core::types::TagProtection, gitfleet_core::errors::GitfleetError> {
        crate::github::api::ProtectionApi::create_tag_protection(self, repo, pattern).await
    }

    async fn delete_tag_protection(
        &self,
        repo: &str,
        identifier: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::ProtectionApi::delete_tag_protection(self, repo, identifier).await
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
        crate::github::api::SearchApi::issues(self, query, sort, order, limit).await
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
        crate::github::api::SearchApi::repos(self, query, sort, order, limit).await
    }

    async fn search_code(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<
        gitfleet_core::types::SearchResult<serde_json::Value>,
        gitfleet_core::errors::GitfleetError,
    > {
        crate::github::api::SearchApi::code(self, query, limit).await
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
        crate::github::api::CodeApi::file_contents(self, repo, path, r#ref).await
    }

    async fn search_code(
        &self,
        query: &str,
        repo: Option<&str>,
        language: Option<&str>,
        limit: u32,
    ) -> Result<Vec<gitfleet_core::types::CodeSearchResult>, gitfleet_core::errors::GitfleetError>
    {
        crate::github::api::CodeApi::search(self, query, repo, language, limit).await
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
        crate::github::api::SecretsApi::list_repo(self, owner, repo).await
    }

    async fn get_repo_public_key(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<gitfleet_core::types::PublicKeyResponse, gitfleet_core::errors::GitfleetError> {
        crate::github::api::SecretsApi::get_repo_public_key(self, owner, repo).await
    }

    async fn set_repo_secret(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        encrypted_value: &str,
        key_id: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::SecretsApi::set_repo(self, owner, repo, name, encrypted_value, key_id)
            .await
    }

    async fn delete_repo_secret(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::SecretsApi::delete_repo(self, owner, repo, name).await
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
        crate::github::api::VariablesApi::list_repo(self, owner, repo).await
    }

    async fn set_repo_variable(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        value: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::VariablesApi::set_repo(self, owner, repo, name, value).await
    }

    async fn delete_repo_variable(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::VariablesApi::delete_repo(self, owner, repo, name).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::LicenseOps for ProviderClient {
    async fn list_licenses(
        &self,
    ) -> Result<Vec<gitfleet_core::types::LicenseSummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::github::api::LicensesApi::list(self).await
    }

    async fn get_license(
        &self,
        key: &str,
    ) -> Result<gitfleet_core::types::LicenseDetail, gitfleet_core::errors::GitfleetError> {
        crate::github::api::LicensesApi::get(self, key).await
    }

    async fn repo_license(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::LicensesApi::repo_license(self, repo).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::DependencyOps for ProviderClient {
    async fn sbom(
        &self,
        repo: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::DependenciesApi::sbom(self, repo).await
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
        crate::github::api::DependenciesApi::review(self, repo, base, head).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::AdvisoryOps for ProviderClient {
    async fn list_dependabot_alerts(
        &self,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::AdvisoriesApi::list_alerts(self, repo, state).await
    }

    async fn list_codeql_alerts(
        &self,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::AdvisoriesApi::list_codeql(self, repo, state).await
    }

    async fn list_secret_scanning_alerts(
        &self,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::AdvisoriesApi::list_secret_scanning(self, repo, state).await
    }

    async fn get_dependabot_alert(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::AdvisoriesApi::get_alert(self, repo, number).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::AttestationOps for ProviderClient {
    async fn list_attestations(
        &self,
        repo: &str,
        subject_digest: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::AttestationsApi::list(self, repo, subject_digest).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::BrowseOps for ProviderClient {
    async fn list_contents(
        &self,
        repo: &str,
        path: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::BrowseApi::list_contents(self, repo, path).await
    }

    async fn file_contents(
        &self,
        repo: &str,
        path: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::BrowseApi::file_contents(self, repo, path, r#ref).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::RawApiOps for ProviderClient {
    async fn raw_get(
        &self,
        endpoint: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::RawApi::get(self, endpoint).await
    }

    async fn raw_post(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::RawApi::post(self, endpoint, body).await
    }

    async fn raw_delete(
        &self,
        endpoint: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        crate::github::api::RawApi::delete(self, endpoint).await
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
        match crate::github::api::PackagesApi::list_for_org(self, owner, package_type, limit).await
        {
            Ok(packages) => Ok(packages),
            Err(gitfleet_core::errors::GitfleetError::NotFound(_)) => {
                crate::github::api::PackagesApi::list_for_user(self, owner, package_type, limit)
                    .await
            }
            Err(error) => Err(error),
        }
    }

    async fn get_package(
        &self,
        owner: &str,
        package_type: &str,
        package_name: &str,
    ) -> Result<serde_json::Value, gitfleet_core::errors::GitfleetError> {
        match crate::github::api::PackagesApi::get_json(self, owner, package_type, package_name)
            .await
        {
            Ok(package) => Ok(package),
            Err(gitfleet_core::errors::GitfleetError::NotFound(_)) => {
                crate::github::api::PackagesApi::get_json_for_user(
                    self,
                    owner,
                    package_type,
                    package_name,
                )
                .await
            }
            Err(error) => Err(error),
        }
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::DevEnvOps for ProviderClient {
    async fn list_codespaces(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::CodespaceSummary>, gitfleet_core::errors::GitfleetError>
    {
        crate::github::api::CodespacesApi::list_for_repo(self, repo).await
    }

    async fn create_codespace(
        &self,
        repo: &str,
        branch: Option<&str>,
    ) -> Result<gitfleet_core::types::CodespaceSummary, gitfleet_core::errors::GitfleetError> {
        crate::github::api::CodespacesApi::create(self, repo, branch, None, None).await
    }

    async fn delete_codespace(
        &self,
        _repo: &str,
        codespace_name: &str,
    ) -> Result<(), gitfleet_core::errors::GitfleetError> {
        crate::github::api::CodespacesApi::delete(self, codespace_name).await
    }
}

#[async_trait::async_trait]
impl gitfleet_core::provider::SecurityOps for ProviderClient {}

#[async_trait::async_trait]
impl gitfleet_core::provider::TemplateOps for ProviderClient {
    async fn list_issue_templates(
        &self,
        repo: &str,
    ) -> Result<Vec<gitfleet_core::types::IssueTemplate>, gitfleet_core::errors::GitfleetError>
    {
        crate::github::api::TemplatesApi::list(self, repo).await
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

fn parse_next_link(link_header: &str) -> Result<Option<String>, GitfleetError> {
    for part in link_header.split(',') {
        let part = part.trim();

        if part.contains("rel=\"next\"") {
            let start = part.find('<').ok_or_else(|| {
                GitfleetError::new("Malformed GitHub pagination Link header: missing '<'.")
            })?;

            let end = part[start + 1..].find('>').map(|offset| start + 1 + offset);
            let end = end.ok_or_else(|| {
                GitfleetError::new("Malformed GitHub pagination Link header: missing '>'.")
            })?;

            let next = &part[start + 1..end];
            if next.is_empty() {
                return Err(GitfleetError::new(
                    "Malformed GitHub pagination Link header: empty next URL.",
                ));
            }

            return Ok(Some(next.to_string()));
        }
    }

    Ok(None)
}

fn validate_next_url(base: &str, next: &str) -> Result<String, GitfleetError> {
    let base_url = url::Url::parse(base)
        .map_err(|e| GitfleetError::new(format!("Invalid API base URL: {e}")))?;
    let next_url = base_url
        .join(next)
        .map_err(|e| GitfleetError::new(format!("Invalid pagination URL: {e}")))?;

    if next_url.scheme() != base_url.scheme()
        || next_url.host_str() != base_url.host_str()
        || next_url.port_or_known_default() != base_url.port_or_known_default()
        || !next_url.username().is_empty()
        || next_url.password().is_some()
    {
        return Err(GitfleetError::new(
            "Provider pagination URL crossed the configured API origin.",
        ));
    }

    let base_path = base_url.path().trim_end_matches('/');
    let next_path = next_url.path();
    let within_api_path = base_path.is_empty()
        || next_path == base_path
        || next_path.starts_with(&format!("{base_path}/"));

    if !within_api_path {
        return Err(GitfleetError::new(
            "Provider pagination URL crossed the configured API path.",
        ));
    }

    Ok(next_url.to_string())
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

#[cfg(test)]
mod tests {
    use gitfleet_core::constants::{
        GITHUB_API_ACCEPT, GITHUB_API_BASE_URL, GITHUB_API_VERSION, STATUS_FORBIDDEN,
        STATUS_NOT_FOUND, STATUS_RATE_LIMITED, STATUS_UNAUTHORIZED, STATUS_UNPROCESSABLE,
    };

    use super::*;

    #[test]
    fn test_parse_next_link_with_next() {
        let header = r#"<https://api.github.com/repos?page=2>; rel="next", <https://api.github.com/repos?page=5>; rel="last""#;
        let result = parse_next_link(header);

        assert_eq!(
            result.unwrap(),
            Some("https://api.github.com/repos?page=2".to_string())
        );
    }

    #[test]
    fn test_parse_next_link_without_next() {
        let header = r#"<https://api.github.com/repos?page=1>; rel="first""#;
        let result = parse_next_link(header);

        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_parse_next_link_empty() {
        let result = parse_next_link("");

        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_parse_next_link_only_next() {
        let header = r#"<https://api.github.com/repos?page=3>; rel="next""#;
        let result = parse_next_link(header);

        assert_eq!(
            result.unwrap(),
            Some("https://api.github.com/repos?page=3".to_string())
        );
    }

    #[test]
    fn test_parse_next_link_prev_and_next() {
        let header = r#"<https://api.github.com/repos?page=1>; rel="prev", <https://api.github.com/repos?page=3>; rel="next""#;
        let result = parse_next_link(header);

        assert_eq!(
            result.unwrap(),
            Some("https://api.github.com/repos?page=3".to_string())
        );
    }

    #[test]
    fn test_parse_next_link_malformed_next() {
        let result = parse_next_link(r#"rel="next""#);

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_next_url_accepts_same_origin() {
        let result = validate_next_url(
            "https://api.github.com/repos",
            "https://api.github.com/repos?page=2",
        )
        .unwrap();

        assert_eq!(result, "https://api.github.com/repos?page=2");
    }

    #[test]
    fn test_validate_next_url_rejects_different_origin() {
        let result = validate_next_url(
            "https://api.github.com/repos",
            "https://attacker.example/repos?page=2",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_next_url_rejects_different_api_path() {
        let result = validate_next_url(
            "https://git.example.com/api/v3/repos",
            "https://git.example.com/admin?page=2",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_next_url_rejects_userinfo() {
        let result = validate_next_url(
            "https://api.github.com/repos",
            "https://user:password@api.github.com/repos?page=2",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_with_default_per_page_preserves_existing_value() {
        assert_eq!(
            with_default_per_page("/repos?per_page=25"),
            "/repos?per_page=25"
        );
    }

    #[test]
    fn test_with_default_per_page_adds_value() {
        assert_eq!(
            with_default_per_page("/repos?sort=updated"),
            "/repos?sort=updated&per_page=100"
        );
    }

    #[test]
    fn test_build_headers_with_token() {
        let client = ProviderClient::new();

        let headers = client.build_headers(Some("testtoken"), None, None).unwrap();

        assert_eq!(headers.get("authorization").unwrap(), "Bearer testtoken");

        assert_eq!(
            headers.get("X-GitHub-Api-Version").unwrap(),
            GITHUB_API_VERSION
        );

        assert_eq!(
            headers.get(reqwest::header::ACCEPT).unwrap(),
            GITHUB_API_ACCEPT
        );

        assert_eq!(
            headers.get(reqwest::header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_build_headers_without_token() {
        let client = ProviderClient::new();

        let headers = client.build_headers(None, None, None).unwrap();

        assert!(headers.get("authorization").is_none());
    }

    #[test]
    fn test_effective_optional_token_prefers_explicit_and_configured_tokens() {
        let client = ProviderClient::with_context("github.com", Some("configured".to_string()));

        assert_eq!(
            client.effective_optional_token(None).as_deref(),
            Some("configured")
        );
        assert_eq!(
            client.effective_optional_token(Some("explicit")).as_deref(),
            Some("explicit")
        );
    }

    #[test]
    fn test_build_headers_custom_accept() {
        let client = ProviderClient::new();

        let headers = client.build_headers(None, Some("text/html"), None).unwrap();

        assert_eq!(headers.get(reqwest::header::ACCEPT).unwrap(), "text/html");
    }

    #[test]
    fn test_build_headers_custom_content_type() {
        let client = ProviderClient::new();

        let headers = client
            .build_headers(None, None, Some("text/plain"))
            .unwrap();

        assert_eq!(
            headers.get(reqwest::header::CONTENT_TYPE).unwrap(),
            "text/plain"
        );
    }

    #[test]
    fn test_api_base_url_default() {
        let client = ProviderClient::new();

        let url = client.api_base_url(None);

        assert_eq!(url, GITHUB_API_BASE_URL);
    }

    #[test]
    fn test_api_base_url_override() {
        let client = ProviderClient::with_base_url("http://localhost:8080");

        let url = client.api_base_url(None);

        assert_eq!(url, "http://localhost:8080");
    }

    #[test]
    fn test_api_base_url_profile_host() {
        let client = ProviderClient::with_host("github.example.com");

        let url = client.api_base_url(None);

        assert_eq!(url, "https://github.example.com/api/v3");
    }

    #[test]
    fn test_api_base_url_profile_github_com_host() {
        let client = ProviderClient::with_host("github.com");

        let url = client.api_base_url(None);

        assert_eq!(url, GITHUB_API_BASE_URL);
    }

    #[test]
    fn test_api_base_url_custom_host() {
        let client = ProviderClient::new();

        let url = client.api_base_url(Some("ghe.example.com"));

        assert_eq!(url, "https://ghe.example.com/api/v3");
    }

    #[test]
    fn test_api_base_url_github_com_host() {
        let client = ProviderClient::new();

        let url = client.api_base_url(Some("github.com"));

        assert_eq!(url, GITHUB_API_BASE_URL);
    }

    #[test]
    fn test_graphql_base_url_uses_enterprise_api_root() {
        let client = ProviderClient::with_host("github.example.com");

        assert_eq!(
            client.graphql_base_url(None),
            "https://github.example.com/api"
        );
    }

    #[test]
    fn test_graphql_base_url_preserves_test_override() {
        let client = ProviderClient::with_base_url("http://localhost:8080");

        assert_eq!(client.graphql_base_url(None), "http://localhost:8080");
    }

    #[test]
    fn test_is_ok_true() {
        let client = ProviderClient::new();

        assert!(client.is_ok(200));

        assert!(client.is_ok(201));
        assert!(client.is_ok(204));

        assert!(client.is_ok(299));
    }

    #[test]
    fn test_is_ok_false() {
        let client = ProviderClient::new();

        assert!(!client.is_ok(100));

        assert!(!client.is_ok(301));
        assert!(!client.is_ok(404));

        assert!(!client.is_ok(500));
    }

    #[test]
    fn test_is_not_found() {
        let client = ProviderClient::new();

        assert!(client.is_not_found(404));

        assert!(!client.is_not_found(200));
        assert!(!client.is_not_found(403));

        assert!(!client.is_not_found(500));
    }

    #[test]
    fn test_handle_error_unauthorized_no_token() {
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
    fn test_handle_error_unauthorized_with_token() {
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

        assert!(err.to_string().contains("Unauthorized"));
    }

    #[test]
    fn test_handle_error_not_found() {
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

        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_handle_error_unprocessable() {
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

        assert!(err.to_string().contains("unprocessable"));
    }

    #[test]
    fn test_handle_error_rate_limited() {
        let err = handle_error(
            STATUS_RATE_LIMITED,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(429)
                    .header("x-ratelimit-remaining", "0")
                    .header("x-ratelimit-limit", "5000")
                    .header("x-ratelimit-reset", "0")
                    .body(String::new())
                    .unwrap(),
            ),
            true,
        );

        assert!(err.to_string().contains("Rate limit"));
    }

    #[test]
    fn test_handle_error_forbidden_rate_limited() {
        let err = handle_error(
            STATUS_FORBIDDEN,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(403)
                    .header("x-ratelimit-remaining", "0")
                    .header("x-ratelimit-limit", "5000")
                    .header("x-ratelimit-reset", "0")
                    .body(String::new())
                    .unwrap(),
            ),
            true,
        );

        assert!(err.to_string().contains("Rate limit"));
    }

    #[test]
    fn test_handle_error_forbidden_not_rate_limited() {
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
    fn test_handle_error_unknown_status() {
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
    fn test_default_provider_client() {
        let client = ProviderClient::default();

        let url = client.api_base_url(None);

        assert_eq!(url, GITHUB_API_BASE_URL);
    }

    #[test]
    fn test_build_headers_default_accept() {
        let client = ProviderClient::new();

        let headers = client.build_headers(None, None, None).unwrap();

        assert_eq!(
            headers.get(reqwest::header::ACCEPT).unwrap(),
            GITHUB_API_ACCEPT
        );
    }

    #[test]
    fn test_build_headers_github_api_version() {
        let client = ProviderClient::new();

        let headers = client.build_headers(None, None, None).unwrap();

        assert_eq!(
            headers.get("X-GitHub-Api-Version").unwrap(),
            GITHUB_API_VERSION
        );
    }

    #[test]
    fn test_handle_error_rate_limited_without_token() {
        let err = handle_error(
            STATUS_RATE_LIMITED,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(429)
                    .header("x-ratelimit-remaining", "0")
                    .header("x-ratelimit-limit", "5000")
                    .header("x-ratelimit-reset", "0")
                    .body(String::new())
                    .unwrap(),
            ),
            false,
        );

        assert!(err.to_string().contains("Rate limit"));
    }

    #[test]
    fn test_handle_error_forbidden_with_remaining() {
        let err = handle_error(
            STATUS_FORBIDDEN,
            &reqwest::Response::from(
                http::Response::builder()
                    .status(403)
                    .header("x-ratelimit-remaining", "5")
                    .header("x-ratelimit-limit", "5000")
                    .header("x-ratelimit-reset", "0")
                    .body(String::new())
                    .unwrap(),
            ),
            true,
        );

        assert!(err.to_string().contains("Unexpected"));
    }
}
