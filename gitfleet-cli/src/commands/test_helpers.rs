use gitfleet_core::errors::GitfleetError;
use gitfleet_core::output::Renderer;
use gitfleet_core::output_state::OutputMode;
use gitfleet_core::provider::ProviderCapability;
use gitfleet_core::provider::*;
use gitfleet_core::types::*;
use gitfleet_providers::ProviderRegistry;

use crate::app::App;

pub struct MockProvider;

impl GitProvider for MockProvider {
    fn id(&self) -> ProviderId {
        ProviderId::GitHub
    }

    fn default_host(&self) -> &'static str {
        "github.com"
    }

    fn capabilities(&self) -> &[ProviderCapability] {
        &[]
    }

    fn repo_ops(&self) -> Option<&dyn RepoOps> {
        Some(self)
    }

    fn change_ops(&self) -> Option<&dyn ChangeOps> {
        Some(self)
    }

    fn review_ops(&self) -> Option<&dyn ReviewOps> {
        Some(self)
    }

    fn issue_ops(&self) -> Option<&dyn IssueOps> {
        Some(self)
    }

    fn label_ops(&self) -> Option<&dyn LabelOps> {
        Some(self)
    }

    fn notification_ops(&self) -> Option<&dyn NotificationOps> {
        Some(self)
    }

    fn pipeline_ops(&self) -> Option<&dyn PipelineOps> {
        Some(self)
    }

    fn release_ops(&self) -> Option<&dyn ReleaseOps> {
        Some(self)
    }

    fn wiki_ops(&self) -> Option<&dyn WikiOps> {
        Some(self)
    }

    fn webhook_ops(&self) -> Option<&dyn WebhookOps> {
        Some(self)
    }

    fn search_ops(&self) -> Option<&dyn SearchOps> {
        Some(self)
    }

    fn environment_ops(&self) -> Option<&dyn EnvironmentOps> {
        Some(self)
    }

    fn deploy_ops(&self) -> Option<&dyn DeployOps> {
        Some(self)
    }

    fn runner_ops(&self) -> Option<&dyn RunnerOps> {
        Some(self)
    }

    fn secret_ops(&self) -> Option<&dyn SecretOps> {
        Some(self)
    }

    fn variable_ops(&self) -> Option<&dyn VariableOps> {
        Some(self)
    }

    fn license_ops(&self) -> Option<&dyn LicenseOps> {
        Some(self)
    }

    fn discussion_ops(&self) -> Option<&dyn DiscussionOps> {
        Some(self)
    }

    fn code_ops(&self) -> Option<&dyn CodeOps> {
        Some(self)
    }

    fn template_ops(&self) -> Option<&dyn TemplateOps> {
        Some(self)
    }

    fn browse_ops(&self) -> Option<&dyn BrowseOps> {
        Some(self)
    }

    fn site_ops(&self) -> Option<&dyn SiteOps> {
        Some(self)
    }

    fn raw_api_ops(&self) -> Option<&dyn RawApiOps> {
        Some(self)
    }

    fn analytics_ops(&self) -> Option<&dyn AnalyticsOps> {
        Some(self)
    }

    fn advisory_ops(&self) -> Option<&dyn AdvisoryOps> {
        Some(self)
    }

    fn attestation_ops(&self) -> Option<&dyn AttestationOps> {
        Some(self)
    }

    fn dependency_ops(&self) -> Option<&dyn DependencyOps> {
        Some(self)
    }

    fn governance_ops(&self) -> Option<&dyn GovernanceOps> {
        Some(self)
    }

    fn policy_ops(&self) -> Option<&dyn PolicyOps> {
        Some(self)
    }

    fn access_ops(&self) -> Option<&dyn AccessOps> {
        Some(self)
    }

    fn identity_ops(&self) -> Option<&dyn IdentityOps> {
        Some(self)
    }

    fn snippet_ops(&self) -> Option<&dyn SnippetOps> {
        Some(self)
    }

    fn registry_ops(&self) -> Option<&dyn RegistryOps> {
        Some(self)
    }

    fn dev_env_ops(&self) -> Option<&dyn DevEnvOps> {
        Some(self)
    }

    fn planning_ops(&self) -> Option<&dyn PlanningOps> {
        Some(self)
    }
}

pub fn make_app() -> App {
    let registry = ProviderRegistry::with_provider(ProviderId::GitHub, Box::new(MockProvider));

    let renderer = Renderer::new(OutputMode::Silent);
    App::new(registry, renderer, ProviderId::GitHub, false)
}

pub fn make_app_yes() -> App {
    let registry = ProviderRegistry::with_provider(ProviderId::GitHub, Box::new(MockProvider));

    let renderer = Renderer::new(OutputMode::Silent).with_yes(true);
    App::new(registry, renderer, ProviderId::GitHub, false)
}

pub fn make_app_dry_run() -> App {
    let registry = ProviderRegistry::with_provider(ProviderId::GitHub, Box::new(MockProvider));

    let renderer = Renderer::new(OutputMode::Silent);
    App::new(registry, renderer, ProviderId::GitHub, true)
}

pub fn make_app_json() -> App {
    let registry = ProviderRegistry::with_provider(ProviderId::GitHub, Box::new(MockProvider));

    let renderer = Renderer::new(OutputMode::Json);
    App::new(registry, renderer, ProviderId::GitHub, false)
}

pub fn make_app_json_yes() -> App {
    let registry = ProviderRegistry::with_provider(ProviderId::GitHub, Box::new(MockProvider));

    let renderer = Renderer::new(OutputMode::Json).with_yes(true);
    App::new(registry, renderer, ProviderId::GitHub, false)
}

pub fn make_app_dry_run_json() -> App {
    let registry = ProviderRegistry::with_provider(ProviderId::GitHub, Box::new(MockProvider));

    let renderer = Renderer::new(OutputMode::Json);
    App::new(registry, renderer, ProviderId::GitHub, true)
}

pub fn make_app_human() -> App {
    let registry = ProviderRegistry::with_provider(ProviderId::GitHub, Box::new(MockProvider));

    let renderer = Renderer::new(OutputMode::Human);
    App::new(registry, renderer, ProviderId::GitHub, false)
}

pub struct NoCapProvider;

impl GitProvider for NoCapProvider {
    fn id(&self) -> ProviderId {
        ProviderId::GitHub
    }

    fn default_host(&self) -> &'static str {
        "github.com"
    }

    fn capabilities(&self) -> &[ProviderCapability] {
        &[]
    }
}

pub fn make_app_no_caps() -> App {
    let registry = ProviderRegistry::with_provider(ProviderId::GitHub, Box::new(NoCapProvider));

    let renderer = Renderer::new(OutputMode::Silent);
    App::new(registry, renderer, ProviderId::GitHub, false)
}

#[async_trait::async_trait]
impl RepoOps for MockProvider {
    async fn list_org_repos(&self, _org: &str) -> Result<Vec<RepoSummary>, GitfleetError> {
        Ok(vec![RepoSummary {
            id: 1,
            name: "repo1".into(),
            fork: false,
            full_name: "org/repo1".into(),
            private: false,
            archived: false,
            default_branch: "main".into(),
            pushed_at: None,
        }])
    }

    async fn list_user_repos(&self) -> Result<Vec<RepoSummary>, GitfleetError> {
        Ok(vec![RepoSummary {
            id: 2,
            name: "user-repo".into(),
            fork: false,
            full_name: "user/user-repo".into(),
            private: true,
            archived: false,
            default_branch: "main".into(),
            pushed_at: None,
        }])
    }

    async fn list_user_named_repos(
        &self,
        _username: &str,
    ) -> Result<Vec<RepoSummary>, GitfleetError> {
        Ok(vec![RepoSummary {
            id: 3,
            name: "named-repo".into(),
            fork: false,
            full_name: "other/named-repo".into(),
            private: false,
            archived: false,
            default_branch: "main".into(),
            pushed_at: None,
        }])
    }

    async fn get_repo(&self, _repo: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({
            "full_name": "org/repo1",
            "description": "A test repo",
            "private": false,
            "default_branch": "main",
            "html_url": "https://github.com/org/repo1"
        }))
    }

    async fn create_repo(
        &self,
        _name: &str,
        _visibility: &str,
        _owner: Option<&str>,
        _owner_type: Option<&str>,
        _description: Option<&str>,
        _initialize: bool,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({
            "full_name": "org/new-repo",
            "html_url": "https://github.com/org/new-repo"
        }))
    }

    async fn update_repo(
        &self,
        _repo: &str,
        _options: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({}))
    }

    async fn delete_repo(&self, _repo: &str) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn star_repo(&self, _repo: &str) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn unstar_repo(&self, _repo: &str) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn list_forks(&self, _repo: &str) -> Result<Vec<RepoSummary>, GitfleetError> {
        Ok(vec![RepoSummary {
            id: 4,
            name: "fork".into(),
            fork: true,
            full_name: "user/fork".into(),
            private: false,
            archived: false,
            default_branch: "main".into(),
            pushed_at: None,
        }])
    }

    async fn fork_repo(
        &self,
        _repo: &str,
        _destination_owner: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({}))
    }

    async fn archive_repo(&self, _repo: &str) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn unarchive_repo(&self, _repo: &str) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl ChangeOps for MockProvider {
    async fn list_changes(
        &self,
        _repo: &str,
        _state: &str,
        _limit: u32,
        _base: Option<&str>,
        _head: Option<&str>,
    ) -> Result<Vec<PullRequest>, GitfleetError> {
        Ok(vec![PullRequest {
            title: "Fix bug".into(),
            state: "open".into(),
            number: 42,
            merged: false,
            draft: Some(false),
            html_url: None,
            created_at: None,
            updated_at: None,
            body: None,
            mergeable_state: None,
            merged_at: None,
            mergeable: None,
            user: Some(PullRequestUser {
                login: "dev".into(),
            }),
            maintainer_can_modify: false,
            merge_commit_sha: None,
            labels: None,
            requested_reviewers: None,
            head: PullRequestHead {
                r#ref: "feature".into(),
                sha: None,
                repo: None,
            },
            base: PullRequestBase {
                r#ref: "main".into(),
                repo: None,
            },
        }])
    }

    async fn get_change(&self, _repo: &str, _number: u64) -> Result<PullRequest, GitfleetError> {
        Ok(PullRequest {
            title: "Fix bug".into(),
            state: "open".into(),
            number: 42,
            merged: false,
            draft: Some(false),
            html_url: None,
            created_at: None,
            updated_at: None,
            body: None,
            mergeable_state: None,
            merged_at: None,
            mergeable: None,
            user: Some(PullRequestUser {
                login: "dev".into(),
            }),
            maintainer_can_modify: false,
            merge_commit_sha: None,
            labels: None,
            requested_reviewers: None,
            head: PullRequestHead {
                r#ref: "feature".into(),
                sha: None,
                repo: None,
            },
            base: PullRequestBase {
                r#ref: "main".into(),
                repo: None,
            },
        })
    }

    async fn create_change(
        &self,
        _repo: &str,
        _title: &str,
        _head: &str,
        _base: &str,
        _body: Option<&str>,
        _draft: bool,
    ) -> Result<PullRequest, GitfleetError> {
        Ok(PullRequest {
            title: "New PR".into(),
            state: "open".into(),
            number: 99,
            merged: false,
            draft: Some(false),
            html_url: None,
            created_at: None,
            updated_at: None,
            body: None,
            mergeable_state: None,
            merged_at: None,
            mergeable: None,
            user: Some(PullRequestUser {
                login: "dev".into(),
            }),
            maintainer_can_modify: false,
            merge_commit_sha: None,
            labels: None,
            requested_reviewers: None,
            head: PullRequestHead {
                r#ref: "feature".into(),
                sha: None,
                repo: None,
            },
            base: PullRequestBase {
                r#ref: "main".into(),
                repo: None,
            },
        })
    }

    async fn update_change(
        &self,
        _repo: &str,
        _number: u64,
        _options: serde_json::Value,
    ) -> Result<PullRequest, GitfleetError> {
        Ok(PullRequest {
            title: "Updated".into(),
            state: "open".into(),
            number: 42,
            merged: false,
            draft: Some(false),
            html_url: None,
            created_at: None,
            updated_at: None,
            body: None,
            mergeable_state: None,
            merged_at: None,
            mergeable: None,
            user: Some(PullRequestUser {
                login: "dev".into(),
            }),
            maintainer_can_modify: false,
            merge_commit_sha: None,
            labels: None,
            requested_reviewers: None,
            head: PullRequestHead {
                r#ref: "feature".into(),
                sha: None,
                repo: None,
            },
            base: PullRequestBase {
                r#ref: "main".into(),
                repo: None,
            },
        })
    }

    async fn merge_change(
        &self,
        _repo: &str,
        _number: u64,
        _method: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"sha": "abc123"}))
    }

    async fn comment_on_change(
        &self,
        _repo: &str,
        _number: u64,
        _body: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({}))
    }

    async fn lock_change(&self, _repo: &str, _number: u64) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn unlock_change(&self, _repo: &str, _number: u64) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn list_change_comments(
        &self,
        _repo: &str,
        _number: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!([]))
    }
}

#[async_trait::async_trait]
impl ReviewOps for MockProvider {
    async fn list_reactions_for_issue(
        &self,
        _repo: &str,
        _issue_number: u64,
    ) -> Result<Vec<ReactionSummary>, GitfleetError> {
        Ok(vec![ReactionSummary {
            id: 1,
            content: "+1".into(),
            user: Some("dev".into()),
            created_at: "2025-01-01".into(),
        }])
    }

    async fn create_reaction_for_issue(
        &self,
        _repo: &str,
        _issue_number: u64,
        _content: &str,
    ) -> Result<ReactionSummary, GitfleetError> {
        Ok(ReactionSummary {
            id: 2,
            content: "+1".into(),
            user: Some("dev".into()),
            created_at: "2025-01-01".into(),
        })
    }

    async fn delete_reaction_for_issue(
        &self,
        _repo: &str,
        _issue_number: u64,
        _reaction_id: u64,
    ) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl IssueOps for MockProvider {
    async fn get_issue(
        &self,
        _repo: &str,
        _number: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(
            serde_json::json!({"number": 1, "title": "Bug", "state": "open", "user": {"login": "dev"}}),
        )
    }

    async fn create_issue(
        &self,
        _repo: &str,
        _title: &str,
        _body: Option<&str>,
        _labels: &[String],
        _assignees: &[String],
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"number": 1, "title": "Bug"}))
    }

    async fn list_issues(
        &self,
        _repo: &str,
        _state: &str,
        _limit: u32,
        _labels: &[String],
        _assignees: &[String],
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"items": [{"number": 1, "title": "Bug", "state": "open"}]}))
    }

    async fn update_issue(
        &self,
        _repo: &str,
        _number: u64,
        _options: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({}))
    }

    async fn comment_on_issue(
        &self,
        _repo: &str,
        _number: u64,
        _body: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({}))
    }

    async fn list_issue_comments(
        &self,
        _repo: &str,
        _number: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!([]))
    }
}

#[async_trait::async_trait]
impl LabelOps for MockProvider {
    async fn list_labels(&self, _repo: &str) -> Result<Vec<Label>, GitfleetError> {
        Ok(vec![Label {
            name: "bug".into(),
            color: "ff0000".into(),
            new_name: None,
            description: "Bug".into(),
        }])
    }

    async fn create_label(
        &self,
        _label: &Label,
        _repo: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({}))
    }

    async fn delete_label(&self, _name: &str, _repo: &str) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl NotificationOps for MockProvider {
    async fn list_notifications(
        &self,
        _all: bool,
        _participating: bool,
        _repo: Option<&str>,
    ) -> Result<Vec<Notification>, GitfleetError> {
        Ok(vec![Notification {
            id: "1".into(),
            repository: "org/repo".into(),
            subject_title: "PR #1".into(),
            subject_type: "PullRequest".into(),
            reason: "subscribed".into(),
            unread: true,
            updated_at: "2025-01-01".into(),
        }])
    }

    async fn mark_notifications_read(&self) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl PipelineOps for MockProvider {
    async fn list_workflows(
        &self,
        _repo: &str,
        _limit: u32,
        _page: Option<u32>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(
            serde_json::json!({"workflows": [{"id": 1, "name": "CI", "state": "active", "path": ".github/workflows/ci.yml"}]}),
        )
    }

    async fn get_workflow(
        &self,
        _repo: &str,
        _workflow_id: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(
            serde_json::json!({"id": 1, "name": "CI", "state": "active", "path": ".github/workflows/ci.yml"}),
        )
    }

    async fn dispatch_pipeline(
        &self,
        _repo: &str,
        _definition_id: Option<&str>,
        _ref: &str,
        _inputs: Option<serde_json::Value>,
    ) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn list_runs(
        &self,
        _repo: &str,
        _filters: &str,
        _limit: u32,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(
            serde_json::json!({"workflow_runs": [{"id": 1, "name": "build", "status": "completed", "conclusion": "success", "head_branch": "main"}]}),
        )
    }

    async fn get_run(&self, _repo: &str, _run_id: u64) -> Result<serde_json::Value, GitfleetError> {
        Ok(
            serde_json::json!({"id": 1, "name": "build", "status": "completed", "conclusion": "success", "head_branch": "main"}),
        )
    }

    async fn cancel_run(&self, _repo: &str, _run_id: u64) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn rerun(&self, _repo: &str, _run_id: u64) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn delete_run(&self, _repo: &str, _run_id: u64) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl ReleaseOps for MockProvider {
    async fn list_releases(
        &self,
        _repo: &str,
        _limit: u32,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(
            serde_json::json!([{"tag_name": "v1.0", "name": "Release 1.0", "draft": false, "prerelease": false, "published_at": "2025-01-01"}]),
        )
    }

    async fn fetch_release_by_tag(
        &self,
        _repo: &str,
        _tag: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(
            serde_json::json!({"tag_name": "v1.0", "name": "Release 1.0", "draft": false, "prerelease": false, "html_url": "https://github.com/org/repo/releases/tag/v1.0"}),
        )
    }

    async fn create_release(
        &self,
        _repo: &str,
        _body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(
            serde_json::json!({"tag_name": "v1.0", "html_url": "https://github.com/org/repo/releases/tag/v1.0"}),
        )
    }

    async fn update_release(
        &self,
        _repo: &str,
        _release: &str,
        _body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({}))
    }

    async fn delete_release(&self, _repo: &str, _release: &str) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl WikiOps for MockProvider {
    async fn list_wiki_pages(&self, _repo: &str) -> Result<Vec<WikiPage>, GitfleetError> {
        Ok(vec![WikiPage {
            path: "Home".into(),
            title: "Home".into(),
            format: "md".into(),
            filename: "Home.md".into(),
        }])
    }

    async fn get_wiki_page(
        &self,
        _repo: &str,
        _page: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        Ok(WikiPageContent {
            page: WikiPage {
                path: "Home".into(),
                title: "Home".into(),
                format: "md".into(),
                filename: "Home.md".into(),
            },
            content: "# Home".into(),
        })
    }

    async fn create_wiki_page(
        &self,
        _repo: &str,
        _title: &str,
        _content: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        Ok(WikiPageContent {
            page: WikiPage {
                path: "New".into(),
                title: "New".into(),
                format: "md".into(),
                filename: "New.md".into(),
            },
            content: "Content".into(),
        })
    }

    async fn update_wiki_page(
        &self,
        _repo: &str,
        _page: &str,
        _content: &str,
    ) -> Result<WikiPageContent, GitfleetError> {
        Ok(WikiPageContent {
            page: WikiPage {
                path: "Home".into(),
                title: "Home".into(),
                format: "md".into(),
                filename: "Home.md".into(),
            },
            content: "Updated".into(),
        })
    }

    async fn delete_wiki_page(&self, _repo: &str, _page: &str) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl WebhookOps for MockProvider {
    async fn list_webhooks(&self, _repo: &str) -> Result<Vec<WebhookSummary>, GitfleetError> {
        Ok(vec![WebhookSummary {
            id: 1,
            name: "web".into(),
            url: "https://example.com/hook".into(),
            events: vec!["push".into()],
            active: true,
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
        }])
    }

    async fn create_webhook(
        &self,
        _repo: &str,
        _input: serde_json::Value,
    ) -> Result<WebhookSummary, GitfleetError> {
        Ok(WebhookSummary {
            id: 2,
            name: "new-hook".into(),
            url: "https://example.com/new".into(),
            events: vec!["push".into()],
            active: true,
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
        })
    }

    async fn remove_webhook(&self, _repo: &str, _hook_id: u64) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn test_webhook(&self, _repo: &str, _hook_id: u64) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SearchOps for MockProvider {
    async fn search_issues(
        &self,
        _query: &str,
        _sort: Option<&str>,
        _order: Option<&str>,
        _limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError> {
        Ok(SearchResult {
            items: vec![
                serde_json::json!({"number": 1, "title": "Bug", "state": "open", "html_url": "https://github.com/org/repo/issues/1"}),
            ],
            total_count: 1,
            incomplete_results: false,
        })
    }

    async fn search_repos(
        &self,
        _query: &str,
        _sort: Option<&str>,
        _order: Option<&str>,
        _limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError> {
        Ok(SearchResult {
            items: vec![
                serde_json::json!({"full_name": "org/repo", "stargazers_count": 100, "language": "Rust", "private": false}),
            ],
            total_count: 1,
            incomplete_results: false,
        })
    }

    async fn search_code(
        &self,
        _query: &str,
        _limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError> {
        Ok(SearchResult {
            items: vec![serde_json::json!({"file": "main.rs", "repo": "org/repo"})],
            total_count: 1,
            incomplete_results: false,
        })
    }
}

#[async_trait::async_trait]
impl EnvironmentOps for MockProvider {
    async fn list_environments(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<EnvironmentListResponse, GitfleetError> {
        Ok(EnvironmentListResponse {
            total_count: 1,
            environments: vec![Environment {
                id: 1,
                name: "production".into(),
                url: Some("https://example.com".into()),
                html_url: "https://github.com/org/repo/settings/environments/1".into(),
                created_at: "2025-01-01".into(),
                updated_at: "2025-01-01".into(),
                wait_timer: None,
                protection_rules: None,
            }],
        })
    }

    async fn create_environment(
        &self,
        _owner: &str,
        _repo: &str,
        _name: &str,
        _wait_timer: Option<u32>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"name": "staging"}))
    }

    async fn delete_environment(
        &self,
        _owner: &str,
        _repo: &str,
        _name: &str,
    ) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl DeployOps for MockProvider {
    async fn list_deployments(
        &self,
        _repo: &str,
        _environment: Option<&str>,
        _limit: u32,
    ) -> Result<Vec<DeploymentSummary>, GitfleetError> {
        Ok(vec![DeploymentSummary {
            id: 1,
            r#ref: "main".into(),
            environment: "production".into(),
            task: "deploy".into(),
            description: None,
            creator: None,
            created_at: "2025-01-01".into(),
            production: true,
        }])
    }

    async fn create_deployment(
        &self,
        _repo: &str,
        _input: serde_json::Value,
    ) -> Result<DeploymentSummary, GitfleetError> {
        Ok(DeploymentSummary {
            id: 2,
            r#ref: "main".into(),
            environment: "staging".into(),
            task: "deploy".into(),
            description: None,
            creator: None,
            created_at: "2025-01-01".into(),
            production: false,
        })
    }
}

#[async_trait::async_trait]
impl RunnerOps for MockProvider {
    async fn list_runners(&self, _repo: &str) -> Result<Vec<RunnerSummary>, GitfleetError> {
        Ok(vec![RunnerSummary {
            id: 1,
            name: "runner-1".into(),
            os: "linux".into(),
            status: "online".into(),
            busy: false,
            labels: vec!["self-hosted".into()],
        }])
    }

    async fn remove_runner(&self, _repo: &str, _runner_id: u64) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SecretOps for MockProvider {
    async fn list_repo_secrets(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<SecretListResponse<RepoSecret>, GitfleetError> {
        Ok(SecretListResponse {
            total_count: 1,
            secrets: vec![RepoSecret {
                name: "TOKEN".into(),
                created_at: "2025-01-01".into(),
                updated_at: "2025-01-01".into(),
            }],
        })
    }

    async fn get_repo_public_key(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<PublicKeyResponse, GitfleetError> {
        Ok(PublicKeyResponse {
            key_id: "key123".into(),
            key: "base64key".into(),
        })
    }

    async fn set_repo_secret(
        &self,
        _owner: &str,
        _repo: &str,
        _name: &str,
        _encrypted_value: &str,
        _key_id: &str,
    ) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn delete_repo_secret(
        &self,
        _owner: &str,
        _repo: &str,
        _name: &str,
    ) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl VariableOps for MockProvider {
    async fn list_repo_variables(
        &self,
        _owner: &str,
        _repo: &str,
    ) -> Result<VariableListResponse<RepoVariable>, GitfleetError> {
        Ok(VariableListResponse {
            total_count: 1,
            variables: vec![RepoVariable {
                name: "ENV".into(),
                created_at: "2025-01-01".into(),
                updated_at: "2025-01-01".into(),
                value: Some("prod".into()),
            }],
        })
    }

    async fn set_repo_variable(
        &self,
        _owner: &str,
        _repo: &str,
        _name: &str,
        _value: &str,
    ) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn delete_repo_variable(
        &self,
        _owner: &str,
        _repo: &str,
        _name: &str,
    ) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl LicenseOps for MockProvider {
    async fn list_licenses(&self) -> Result<Vec<LicenseSummary>, GitfleetError> {
        Ok(vec![LicenseSummary {
            key: "mit".into(),
            name: "MIT License".into(),
            spdx_id: "MIT".into(),
            url: "https://api.github.com/licenses/mit".into(),
        }])
    }

    async fn get_license(&self, _key: &str) -> Result<LicenseDetail, GitfleetError> {
        Ok(LicenseDetail {
            key: "mit".into(),
            name: "MIT License".into(),
            spdx_id: "MIT".into(),
            url: "https://api.github.com/licenses/mit".into(),
            description: "A permissive license".into(),
            implementation: "Create LICENSE".into(),
            permissions: vec!["commercial-use".into()],
            conditions: vec!["include-copyright".into()],
            limitations: vec!["liability".into()],
            body: "MIT License text...".into(),
        })
    }

    async fn repo_license(&self, _repo: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait::async_trait]
impl DiscussionOps for MockProvider {
    async fn list_discussions(
        &self,
        _owner: &str,
        _name: &str,
        _category_id: Option<&str>,
        _limit: u32,
    ) -> Result<Vec<Discussion>, GitfleetError> {
        Ok(vec![Discussion {
            id: "1".into(),
            url: "https://github.com/org/repo/discussions/1".into(),
            body: "discussion body".into(),
            title: "How to?".into(),
            number: 1,
            author: "dev".into(),
            closed: false,
            category: "Q&A".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
            comments_count: 3,
        }])
    }

    async fn get_discussion(
        &self,
        _owner: &str,
        _name: &str,
        _discussion_number: u64,
    ) -> Result<Discussion, GitfleetError> {
        Ok(Discussion {
            id: "1".into(),
            url: "https://github.com/org/repo/discussions/1".into(),
            body: "discussion body".into(),
            title: "How to?".into(),
            number: 1,
            author: "dev".into(),
            closed: false,
            category: "Q&A".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
            comments_count: 3,
        })
    }

    async fn create_discussion(
        &self,
        _owner: &str,
        _name: &str,
        _title: &str,
        _body: &str,
        _category_id: Option<&str>,
    ) -> Result<Discussion, GitfleetError> {
        Ok(Discussion {
            id: "2".into(),
            url: "https://github.com/org/repo/discussions/2".into(),
            body: "new body".into(),
            title: "New discussion".into(),
            number: 2,
            author: "dev".into(),
            closed: false,
            category: "General".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
            comments_count: 0,
        })
    }
}

#[async_trait::async_trait]
impl CodeOps for MockProvider {
    async fn get_file_contents(
        &self,
        _repo: &str,
        _path: &str,
        _ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"content": "file contents", "encoding": "base64"}))
    }

    async fn search_code(
        &self,
        _query: &str,
        _repo: Option<&str>,
        _language: Option<&str>,
        _limit: u32,
    ) -> Result<Vec<CodeSearchResult>, GitfleetError> {
        Ok(vec![CodeSearchResult {
            file: "src/main.rs".into(),
            repo: "org/repo".into(),
            url: "https://github.com/org/repo/blob/main/src/main.rs".into(),
        }])
    }
}

#[async_trait::async_trait]
impl TemplateOps for MockProvider {
    async fn list_issue_templates(&self, _repo: &str) -> Result<Vec<IssueTemplate>, GitfleetError> {
        Ok(vec![IssueTemplate {
            name: "bug_report".into(),
            filename: ".github/ISSUE_TEMPLATE/bug_report.md".into(),
            path: ".github/ISSUE_TEMPLATE/bug_report.md".into(),
            body: Some("Describe the bug".into()),
            about: Some("Report a bug".into()),
            title: Some("Bug Report".into()),
            labels: Some(vec!["bug".into()]),
            assignees: None,
        }])
    }
}

#[async_trait::async_trait]
impl BrowseOps for MockProvider {
    async fn list_contents(
        &self,
        _repo: &str,
        _path: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!([{"name": "README.md", "type": "file"}]))
    }

    async fn file_contents(
        &self,
        _repo: &str,
        _path: &str,
        _ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"content": "base64content"}))
    }
}

#[async_trait::async_trait]
impl SiteOps for MockProvider {
    async fn get_pages(&self, _repo: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"url": "https://org.github.io/repo", "status": "built"}))
    }

    async fn create_pages(
        &self,
        _repo: &str,
        _source: &str,
        _build_type: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"url": "https://org.github.io/repo", "status": "queued"}))
    }

    async fn remove_pages(&self, _repo: &str) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl RawApiOps for MockProvider {
    async fn raw_get(&self, _endpoint: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"data": "value"}))
    }

    async fn raw_post(
        &self,
        _endpoint: &str,
        _body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"created": true}))
    }

    async fn raw_put(
        &self,
        _endpoint: &str,
        _body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"updated": true}))
    }

    async fn raw_patch(
        &self,
        _endpoint: &str,
        _body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"updated": true}))
    }

    async fn raw_delete(&self, _endpoint: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"deleted": true}))
    }
}

#[async_trait::async_trait]
impl AnalyticsOps for MockProvider {
    async fn get_traffic_views(&self, _repo: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"views": {"count": 100}}))
    }

    async fn get_traffic_clones(&self, _repo: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"clones": {"count": 50}}))
    }
}

#[async_trait::async_trait]
impl AdvisoryOps for MockProvider {
    async fn list_dependabot_alerts(
        &self,
        _repo: &str,
        _state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!([{"number": 1, "severity": "high"}]))
    }

    async fn list_codeql_alerts(
        &self,
        _repo: &str,
        _state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!([{"number": 2, "severity": "medium"}]))
    }

    async fn list_secret_scanning_alerts(
        &self,
        _repo: &str,
        _state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!([{"number": 3, "severity": "critical"}]))
    }

    async fn get_dependabot_alert(
        &self,
        _repo: &str,
        _number: u64,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"number": 1, "severity": "high"}))
    }
}

#[async_trait::async_trait]
impl AttestationOps for MockProvider {
    async fn list_attestations(
        &self,
        _repo: &str,
        _subject_digest: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"attestations": [{"repository_id": 1}]}))
    }
}

#[async_trait::async_trait]
impl DependencyOps for MockProvider {
    async fn sbom(&self, _repo: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"packages": []}))
    }

    async fn review_dependencies(
        &self,
        _repo: &str,
        _base: &str,
        _head: &str,
    ) -> Result<Vec<DependencyReviewChange>, GitfleetError> {
        Ok(vec![])
    }
}

#[async_trait::async_trait]
impl GovernanceOps for MockProvider {
    async fn list_rulesets(&self, _repo: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!([{"id": 1}]))
    }

    async fn create_ruleset(
        &self,
        _repo: &str,
        _input: &RulesetInput,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"id": 2}))
    }

    async fn delete_ruleset(&self, _repo: &str, _ruleset_id: u64) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl PolicyOps for MockProvider {
    async fn get_branch_protection(
        &self,
        _repo: &str,
        _branch: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"required_status_checks": {}}))
    }

    async fn protect_branch(
        &self,
        _repo: &str,
        _branch: &str,
        _input: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({}))
    }

    async fn unprotect_branch(&self, _repo: &str, _branch: &str) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn list_tag_protection(&self, _repo: &str) -> Result<Vec<TagProtection>, GitfleetError> {
        Ok(vec![])
    }

    async fn create_tag_protection(
        &self,
        _repo: &str,
        _pattern: &str,
    ) -> Result<TagProtection, GitfleetError> {
        Ok(TagProtection {
            identifier: "1".into(),
            pattern: "v*".into(),
            created_at: "2025-01-01".into(),
        })
    }

    async fn delete_tag_protection(
        &self,
        _repo: &str,
        _identifier: &str,
    ) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl AccessOps for MockProvider {
    async fn invite_org_member(
        &self,
        _org: &str,
        _username: &str,
        _role: &str,
    ) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn invite_collaborator(
        &self,
        _owner: &str,
        _repo: &str,
        _username: &str,
        _permission: &str,
    ) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn list_org_members(&self, _org: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!([{"login": "dev1"}]))
    }

    async fn remove_org_member(&self, _org: &str, _username: &str) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn list_teams(&self, _org: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!([{"name": "team1"}]))
    }

    async fn create_team(
        &self,
        _org: &str,
        _name: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"name": "team1"}))
    }

    async fn list_team_members(
        &self,
        _org: &str,
        _team_slug: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!([{"login": "dev1"}]))
    }
}

#[async_trait::async_trait]
impl IdentityOps for MockProvider {
    async fn list_ssh_keys(&self) -> Result<Vec<SshKeySummary>, GitfleetError> {
        Ok(vec![SshKeySummary {
            id: 1,
            title: "key1".into(),
            key: "ssh-rsa ...".into(),
            created_at: "2025-01-01".into(),
        }])
    }

    async fn add_ssh_key(&self, _title: &str, _key: &str) -> Result<SshKeySummary, GitfleetError> {
        Ok(SshKeySummary {
            id: 2,
            title: "new-key".into(),
            key: "ssh-rsa ...".into(),
            created_at: "2025-01-01".into(),
        })
    }

    async fn delete_ssh_key(&self, _key_id: u64) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn list_gpg_keys(&self) -> Result<Vec<GpgKeySummary>, GitfleetError> {
        Ok(vec![GpgKeySummary {
            id: 1,
            name: "gpg1".into(),
            key_id: "ABC123".into(),
            created_at: "2025-01-01".into(),
        }])
    }

    async fn add_gpg_key(&self, _armored_key: &str) -> Result<GpgKeySummary, GitfleetError> {
        Ok(GpgKeySummary {
            id: 2,
            name: "new-gpg".into(),
            key_id: "DEF456".into(),
            created_at: "2025-01-01".into(),
        })
    }

    async fn delete_gpg_key(&self, _key_id: u64) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SnippetOps for MockProvider {
    async fn list_snippets(&self, _owner: &str) -> Result<Vec<GistSummary>, GitfleetError> {
        Ok(vec![GistSummary {
            id: "1".into(),
            description: Some("snippet1".into()),
            public: true,
            html_url: "https://gist.github.com/1".into(),
            git_pull_url: "https://gist.github.com/1.git".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
            owner: None,
            files: vec![],
        }])
    }

    async fn get_snippet(&self, _gist_id: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"id": "1"}))
    }

    async fn create_snippet(
        &self,
        _description: &str,
        _public: bool,
        _files: serde_json::Value,
    ) -> Result<GistSummary, GitfleetError> {
        Ok(GistSummary {
            id: "2".into(),
            description: Some("new".into()),
            public: true,
            html_url: "https://gist.github.com/2".into(),
            git_pull_url: "https://gist.github.com/2.git".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
            owner: None,
            files: vec![],
        })
    }

    async fn delete_snippet(&self, _gist_id: &str) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl RegistryOps for MockProvider {
    async fn list_packages(
        &self,
        _owner: &str,
        _package_type: Option<&str>,
        _limit: u32,
    ) -> Result<Vec<PackageSummary>, GitfleetError> {
        Ok(vec![PackageSummary {
            id: 1,
            name: "pkg1".into(),
            package_type: "npm".into(),
            visibility: "public".into(),
            url: "https://github.com/org/repo/packages/1".into(),
            html_url: "https://github.com/org/repo/packages/1".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
            owner: "org".into(),
            repository: "repo".into(),
        }])
    }

    async fn get_package(
        &self,
        _owner: &str,
        _package_type: &str,
        _package_name: &str,
    ) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"name": "pkg1"}))
    }
}

#[async_trait::async_trait]
impl DevEnvOps for MockProvider {
    async fn list_codespaces(&self, _repo: &str) -> Result<Vec<CodespaceSummary>, GitfleetError> {
        Ok(vec![CodespaceSummary {
            id: 1,
            name: "codespace-1".into(),
            state: "Available".into(),
            owner: "dev".into(),
            repo: "org/repo".into(),
            branch: "main".into(),
            created_at: "2025-01-01".into(),
            idle_timeout_minutes: 30,
            machine: "standardLinux32gb".into(),
        }])
    }

    async fn create_codespace(
        &self,
        _repo: &str,
        _branch: Option<&str>,
    ) -> Result<CodespaceSummary, GitfleetError> {
        Ok(CodespaceSummary {
            id: 2,
            name: "new-codespace".into(),
            state: "Creating".into(),
            owner: "dev".into(),
            repo: "org/repo".into(),
            branch: "main".into(),
            created_at: "2025-01-01".into(),
            idle_timeout_minutes: 30,
            machine: "standardLinux32gb".into(),
        })
    }

    async fn delete_codespace(
        &self,
        _repo: &str,
        _codespace_name: &str,
    ) -> Result<(), GitfleetError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl PlanningOps for MockProvider {
    async fn list_milestones(
        &self,
        _repo: &str,
        _state: Option<&str>,
        _limit: u32,
    ) -> Result<Vec<Milestone>, GitfleetError> {
        Ok(vec![Milestone {
            id: 1,
            url: "https://github.com/org/repo/milestone/1".into(),
            title: "v1.0".into(),
            number: 1,
            html_url: "https://github.com/org/repo/milestone/1".into(),
            open_issues: 3,
            state: MilestoneState::Open,
            due_on: None,
            closed_issues: 2,
        }])
    }

    async fn create_milestone(
        &self,
        _repo: &str,
        _title: &str,
        _description: Option<&str>,
    ) -> Result<Milestone, GitfleetError> {
        Ok(Milestone {
            id: 2,
            url: "https://github.com/org/repo/milestone/2".into(),
            title: "v2.0".into(),
            number: 2,
            html_url: "https://github.com/org/repo/milestone/2".into(),
            open_issues: 0,
            state: MilestoneState::Open,
            due_on: None,
            closed_issues: 0,
        })
    }

    async fn get_milestone(&self, _repo: &str, _number: u64) -> Result<Milestone, GitfleetError> {
        Ok(Milestone {
            id: 1,
            url: "https://github.com/org/repo/milestone/1".into(),
            title: "v1.0".into(),
            number: 1,
            html_url: "https://github.com/org/repo/milestone/1".into(),
            open_issues: 3,
            state: MilestoneState::Open,
            due_on: None,
            closed_issues: 2,
        })
    }

    async fn update_milestone(
        &self,
        _repo: &str,
        _number: u64,
        _input: serde_json::Value,
    ) -> Result<Milestone, GitfleetError> {
        Ok(Milestone {
            id: 1,
            url: "https://github.com/org/repo/milestone/1".into(),
            title: "Updated".into(),
            number: 1,
            html_url: "https://github.com/org/repo/milestone/1".into(),
            open_issues: 3,
            state: MilestoneState::Open,
            due_on: None,
            closed_issues: 2,
        })
    }

    async fn delete_milestone(&self, _repo: &str, _number: u64) -> Result<(), GitfleetError> {
        Ok(())
    }

    async fn list_projects(
        &self,
        _owner: &str,
        _limit: u32,
    ) -> Result<Vec<ProjectSummary>, GitfleetError> {
        Ok(vec![ProjectSummary {
            id: "1".into(),
            number: 1,
            title: "Project 1".into(),
            description: "Description".into(),
            closed: false,
            url: "https://github.com/orgs/org/projects/1".into(),
            updated_at: Some("2025-01-01".into()),
        }])
    }

    async fn get_project(&self, _project_id: &str) -> Result<serde_json::Value, GitfleetError> {
        Ok(serde_json::json!({"id": 1}))
    }

    async fn create_project(
        &self,
        _owner: &str,
        _title: &str,
        _body: Option<&str>,
    ) -> Result<ProjectSummary, GitfleetError> {
        Ok(ProjectSummary {
            id: "2".into(),
            number: 2,
            title: "New Project".into(),
            description: "Description".into(),
            closed: false,
            url: "https://github.com/orgs/org/projects/2".into(),
            updated_at: Some("2025-01-01".into()),
        })
    }

    async fn delete_project(&self, _project_id: &str) -> Result<(), GitfleetError> {
        Ok(())
    }
}
