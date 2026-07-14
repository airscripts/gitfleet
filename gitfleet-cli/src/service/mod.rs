pub mod auth;
pub mod browse;
pub mod changes;
pub mod deployments;
pub mod discussions;
pub mod environments;
pub mod inbox;
pub mod issues;
pub mod labels;
pub mod licenses;
pub mod pipelines;
pub mod releases;
pub mod repos;
pub mod runners;
pub mod search;
pub mod secrets;
pub mod variables;
pub mod webhooks;
pub mod wiki;
pub mod workspace;

#[cfg(test)]
mod tests {
    use gitfleet_core::errors::GitfleetError;
    use gitfleet_core::output::Renderer;
    use gitfleet_core::output_state::OutputMode;
    use gitfleet_core::provider::*;
    use gitfleet_core::types::*;

    use super::*;

    struct MockProvider;

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

        async fn get_change(
            &self,
            _repo: &str,
            _number: u64,
        ) -> Result<PullRequest, GitfleetError> {
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
            r#ref: &str,
            _inputs: Option<serde_json::Value>,
        ) -> Result<(), GitfleetError> {
            let _ = r#ref;

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

        async fn get_run(
            &self,
            _repo: &str,
            _run_id: u64,
        ) -> Result<serde_json::Value, GitfleetError> {
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

    fn mock_provider() -> MockProvider {
        MockProvider
    }

    #[tokio::test]
    async fn test_service_module_compiles() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);

        assert_eq!(p.id(), ProviderId::GitHub);

        assert_eq!(r.mode(), OutputMode::Silent);
    }

    #[tokio::test]
    async fn test_repos_list_user_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        repos::list(&p, &r, None, None).await.unwrap();
    }

    #[tokio::test]
    async fn test_repos_list_org_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        repos::list(&p, &r, Some("org"), None).await.unwrap();
    }

    #[tokio::test]
    async fn test_repos_list_named_user_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        repos::list(&p, &r, None, Some("user")).await.unwrap();
    }

    #[tokio::test]
    async fn test_repos_list_user_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        repos::list(&p, &r, None, None).await.unwrap();
    }

    #[tokio::test]
    async fn test_repos_view_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        repos::view(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_repos_view_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        repos::view(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_repos_create_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        repos::create(
            &p,
            &r,
            repos::CreateOptions {
                name: "new-repo",
                owner: None,
                owner_type: None,
                visibility: "public",
                description: None,
                initialize: false,
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repos_create_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        repos::create(
            &p,
            &r,
            repos::CreateOptions {
                name: "new-repo",
                owner: None,
                owner_type: None,
                visibility: "public",
                description: None,
                initialize: false,
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repos_delete_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        repos::delete(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_repos_delete_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        repos::delete(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_repos_star_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        repos::star(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_repos_star_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        repos::star(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_repos_unstar_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        repos::unstar(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_repos_unstar_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        repos::unstar(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_changes_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        changes::list(&p, &r, "org/repo", "open", 10, None, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_changes_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        changes::list(&p, &r, "org/repo", "open", 10, None, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_changes_view_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        changes::view(&p, &r, "org/repo", 42).await.unwrap();
    }

    #[tokio::test]
    async fn test_changes_view_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        changes::view(&p, &r, "org/repo", 42).await.unwrap();
    }

    #[tokio::test]
    async fn test_changes_create_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        changes::create(
            &p, &r, "org/repo", "Fix bug", "feature", "main", None, false,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_changes_create_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        changes::create(
            &p, &r, "org/repo", "Fix bug", "feature", "main", None, false,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_changes_merge_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        changes::merge(&p, &r, "org/repo", 42, "merge")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_changes_merge_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        changes::merge(&p, &r, "org/repo", 42, "merge")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_issues_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        issues::list(&p, &r, "org/repo", "open", 10).await.unwrap();
    }

    #[tokio::test]
    async fn test_issues_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        issues::list(&p, &r, "org/repo", "open", 10).await.unwrap();
    }

    #[tokio::test]
    async fn test_issues_view_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        issues::view(&p, &r, "org/repo", 1).await.unwrap();
    }

    #[tokio::test]
    async fn test_issues_view_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        issues::view(&p, &r, "org/repo", 1).await.unwrap();
    }

    #[tokio::test]
    async fn test_issues_create_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        issues::create(&p, &r, "org/repo", "Bug report", None, &[], &[])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_issues_create_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        issues::create(&p, &r, "org/repo", "Bug report", None, &[], &[])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_labels_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        labels::list(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_labels_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        labels::list(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_inbox_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        inbox::list(&p, &r, false, false, None).await.unwrap();
    }

    #[tokio::test]
    async fn test_inbox_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        inbox::list(&p, &r, true, false, None).await.unwrap();
    }

    #[tokio::test]
    async fn test_pipelines_list_workflows_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        pipelines::list_workflows(&p, &r, "org/repo", 10, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_pipelines_list_workflows_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        pipelines::list_workflows(&p, &r, "org/repo", 10, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_pipelines_view_workflow_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        pipelines::view_workflow(&p, &r, "org/repo", "1")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_pipelines_view_workflow_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        pipelines::view_workflow(&p, &r, "org/repo", "1")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_pipelines_list_runs_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        pipelines::list_runs(&p, &r, "org/repo", "status=completed", 10)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_pipelines_list_runs_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        pipelines::list_runs(&p, &r, "org/repo", "status=completed", 10)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_pipelines_view_run_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        pipelines::view_run(&p, &r, "org/repo", 1).await.unwrap();
    }

    #[tokio::test]
    async fn test_pipelines_view_run_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        pipelines::view_run(&p, &r, "org/repo", 1).await.unwrap();
    }

    #[tokio::test]
    async fn test_pipelines_trigger_run() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        pipelines::trigger_run(&p, &r, "org/repo", Some("ci.yml"), "main", None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_pipelines_cancel_run() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        pipelines::cancel_run(&p, &r, "org/repo", 1).await.unwrap();
    }

    #[tokio::test]
    async fn test_releases_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        releases::list(&p, &r, "org/repo", 10).await.unwrap();
    }

    #[tokio::test]
    async fn test_releases_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        releases::list(&p, &r, "org/repo", 10).await.unwrap();
    }

    #[tokio::test]
    async fn test_releases_view_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        releases::view(&p, &r, "org/repo", "v1.0").await.unwrap();
    }

    #[tokio::test]
    async fn test_releases_view_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        releases::view(&p, &r, "org/repo", "v1.0").await.unwrap();
    }

    #[tokio::test]
    async fn test_releases_create_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        releases::create(&p, &r, "org/repo", serde_json::json!({"tag_name": "v1.0"}))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_releases_create_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        releases::create(&p, &r, "org/repo", serde_json::json!({"tag_name": "v1.0"}))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_releases_delete_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        releases::delete(&p, &r, "org/repo", "1").await.unwrap();
    }

    #[tokio::test]
    async fn test_wiki_list_pages_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        wiki::list_pages(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_wiki_list_pages_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        wiki::list_pages(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_webhooks_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        webhooks::list(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_webhooks_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        webhooks::list(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_webhooks_create_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        webhooks::create(&p, &r, "org/repo", serde_json::json!({}))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_webhooks_create_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        webhooks::create(&p, &r, "org/repo", serde_json::json!({}))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_webhooks_delete() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        webhooks::delete(&p, &r, "org/repo", 1).await.unwrap();
    }

    #[tokio::test]
    async fn test_webhooks_test() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        webhooks::test(&p, &r, "org/repo", 1).await.unwrap();
    }

    #[tokio::test]
    async fn test_search_issues_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        search::search_issues(&p, &r, "bug", None, None, 10)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_search_issues_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        search::search_issues(&p, &r, "bug", None, None, 10)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_search_repos_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        search::search_repos(&p, &r, "rust", None, None, 10)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_search_repos_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        search::search_repos(&p, &r, "rust", None, None, 10)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_search_code_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        search::search_code(&p, &r, "fn main", 10).await.unwrap();
    }

    #[tokio::test]
    async fn test_search_code_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        search::search_code(&p, &r, "fn main", 10).await.unwrap();
    }

    #[tokio::test]
    async fn test_environments_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        environments::list(&p, &r, "org", "repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_environments_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        environments::list(&p, &r, "org", "repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_environments_create_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        environments::create(&p, &r, "org", "repo", "staging", None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_environments_create_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        environments::create(&p, &r, "org", "repo", "staging", None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_deployments_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        deployments::list(&p, &r, "org/repo", None, 10)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_deployments_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        deployments::list(&p, &r, "org/repo", None, 10)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_deployments_create_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        deployments::create(&p, &r, "org/repo", serde_json::json!({}))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_deployments_create_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        deployments::create(&p, &r, "org/repo", serde_json::json!({}))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_runners_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        runners::list(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_runners_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        runners::list(&p, &r, "org/repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_runners_remove() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        runners::remove(&p, &r, "org/repo", 1).await.unwrap();
    }

    #[tokio::test]
    async fn test_secrets_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        secrets::list(&p, &r, "org", "repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_secrets_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        secrets::list(&p, &r, "org", "repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_secrets_set() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        secrets::set(&p, &r, "org", "repo", "MY_SECRET", "encrypted", "key123")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_secrets_delete() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        secrets::delete(&p, &r, "org", "repo", "MY_SECRET")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_secrets_get_public_key_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        secrets::get_public_key(&p, &r, "org", "repo")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_secrets_get_public_key_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        secrets::get_public_key(&p, &r, "org", "repo")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_variables_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        variables::list(&p, &r, "org", "repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_variables_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        variables::list(&p, &r, "org", "repo").await.unwrap();
    }

    #[tokio::test]
    async fn test_variables_set() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        variables::set(&p, &r, "org", "repo", "MY_VAR", "value")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_variables_delete() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        variables::delete(&p, &r, "org", "repo", "MY_VAR")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_licenses_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        licenses::list(&p, &r).await.unwrap();
    }

    #[tokio::test]
    async fn test_licenses_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        licenses::list(&p, &r).await.unwrap();
    }

    #[tokio::test]
    async fn test_licenses_view_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        licenses::view(&p, &r, "mit").await.unwrap();
    }

    #[tokio::test]
    async fn test_licenses_view_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        licenses::view(&p, &r, "mit").await.unwrap();
    }

    #[tokio::test]
    async fn test_discussions_list_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        discussions::list(&p, &r, "org", "repo", None, 10)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_discussions_list_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        discussions::list(&p, &r, "org", "repo", None, 10)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_browse_open_json() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        browse::open(&p, &r, "github.com", "org/repo", None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_browse_open_human() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Human);
        browse::open(&p, &r, "github.com", "org/repo", None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_browse_open_with_path() {
        let p = mock_provider();

        let r = Renderer::new(OutputMode::Silent);
        browse::open(&p, &r, "github.com", "org/repo", Some("src/main.rs"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_auth_get_authenticated_user_json() {
        let status = AuthStatus {
            user: AuthUser {
                login: "octocat".into(),
                html_url: "https://github.com/octocat".into(),
                avatar_url: "https://github.com/octocat.png".into(),
                name: Some("Octocat".into()),
            },
            scopes: vec!["repo".into()],
        };

        let r = Renderer::new(OutputMode::Silent);
        auth::get_authenticated_user(&status, &r).await.unwrap();
    }

    #[tokio::test]
    async fn test_auth_get_authenticated_user_human() {
        let status = AuthStatus {
            user: AuthUser {
                login: "octocat".into(),
                html_url: "https://github.com/octocat".into(),
                avatar_url: "https://github.com/octocat.png".into(),
                name: None,
            },
            scopes: vec!["repo".into(), "read:org".into()],
        };

        let r = Renderer::new(OutputMode::Human);
        auth::get_authenticated_user(&status, &r).await.unwrap();
    }

    #[tokio::test]
    async fn test_repo_util_resolve_repo_some() {
        let repo = crate::repo_util::resolve_repo(&Some("org/repo".to_string())).unwrap();

        assert_eq!(repo, "org/repo");
    }

    #[tokio::test]
    async fn test_repo_util_split_repo() {
        let (owner, name) = crate::repo_util::split_repo("org/repo").unwrap();

        assert_eq!(owner, "org");

        assert_eq!(name, "repo");
    }

    struct NoCapProvider;

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

    #[tokio::test]
    async fn test_repos_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = repos::list(&p, &r, None, None).await;

        assert!(result.is_err());

        assert!(result.unwrap_err().to_string().contains("does not support"));
    }

    #[tokio::test]
    async fn test_changes_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = changes::list(&p, &r, "org/repo", "open", 10, None, None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_issues_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = issues::list(&p, &r, "org/repo", "open", 10).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_labels_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = labels::list(&p, &r, "org/repo").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_inbox_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = inbox::list(&p, &r, false, false, None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pipelines_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = pipelines::list_workflows(&p, &r, "org/repo", 10, None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_releases_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = releases::list(&p, &r, "org/repo", 10).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_wiki_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = wiki::list_pages(&p, &r, "org/repo").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_webhooks_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = webhooks::list(&p, &r, "org/repo").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_issues_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = search::search_issues(&p, &r, "bug", None, None, 10).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_environments_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = environments::list(&p, &r, "org", "repo").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_deployments_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = deployments::list(&p, &r, "org/repo", None, 10).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runners_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = runners::list(&p, &r, "org/repo").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_secrets_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = secrets::list(&p, &r, "org", "repo").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_variables_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = variables::list(&p, &r, "org", "repo").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_licenses_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = licenses::list(&p, &r).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_discussions_list_no_capability() {
        let p = NoCapProvider;
        let r = Renderer::new(OutputMode::Silent);

        let result = discussions::list(&p, &r, "org", "repo", None, 10).await;

        assert!(result.is_err());
    }
}
