use std::fmt;

use serde::{Deserialize, Serialize};

use crate::errors::GitfleetError;
use crate::types::{
    CodeSearchResult, CodespaceSummary, DependencyReviewChange, DeploymentSummary, Discussion,
    EnvironmentListResponse, GistSummary, GpgKeySummary, IssueTemplate, Label, LicenseDetail,
    LicenseSummary, Notification, PackageSummary, PublicKeyResponse, PullRequest, RepoSecret,
    RepoSummary, RepoVariable, RulesetInput, RunnerSummary, SearchResult, SecretListResponse,
    SshKeySummary, VariableListResponse, WebhookSummary, WikiPage, WikiPageContent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderId {
    GitHub,
    GitLab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenSource {
    Environment,
    Profile,
    None,
}

#[derive(Clone)]
pub struct ProviderContext {
    pub profile_name: String,
    pub provider: ProviderId,
    pub host: String,
    pub token: Option<String>,
    pub token_source: TokenSource,
    pub capabilities: Vec<ProviderCapability>,
}

impl ProviderContext {
    pub fn with_capabilities(mut self, capabilities: &[ProviderCapability]) -> Self {
        self.capabilities = capabilities.to_vec();
        self
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderId::GitHub => write!(f, "github"),
            ProviderId::GitLab => write!(f, "gitlab"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderCapability {
    Repositories,
    Changes,
    Reviews,
    Issues,
    Pipelines,
    Releases,
    Milestones,
    Projects,
    Wiki,
    Site,
    Discussions,
    Security,
    Registry,
    DevelopmentEnvironments,
    Deployments,
    Environments,
    Runners,
    Webhooks,
    Access,
    Identity,
    Analytics,
    Snippets,
    Governance,
    MergeAutomation,
    RepositoryPolicies,
    Notifications,
    Search,
    Code,
    Labels,
    Templates,
    Dependencies,
    Advisories,
    Attestations,
    Secrets,
    Variables,
    Licenses,
    Browsing,
    RawApi,
}

impl fmt::Display for ProviderCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ProviderCapability::Repositories => "repositories",
            ProviderCapability::Changes => "changes",
            ProviderCapability::Reviews => "reviews",
            ProviderCapability::Issues => "issues",
            ProviderCapability::Pipelines => "pipelines",
            ProviderCapability::Releases => "releases",
            ProviderCapability::Milestones => "milestones",
            ProviderCapability::Projects => "projects",
            ProviderCapability::Wiki => "wiki",
            ProviderCapability::Site => "site",
            ProviderCapability::Discussions => "discussions",
            ProviderCapability::Security => "security",
            ProviderCapability::Registry => "registry",
            ProviderCapability::DevelopmentEnvironments => "developmentEnvironments",
            ProviderCapability::Deployments => "deployments",
            ProviderCapability::Environments => "environments",
            ProviderCapability::Runners => "runners",
            ProviderCapability::Webhooks => "webhooks",
            ProviderCapability::Access => "access",
            ProviderCapability::Identity => "identity",
            ProviderCapability::Analytics => "analytics",
            ProviderCapability::Snippets => "snippets",
            ProviderCapability::Governance => "governance",
            ProviderCapability::MergeAutomation => "mergeAutomation",
            ProviderCapability::RepositoryPolicies => "repositoryPolicies",
            ProviderCapability::Notifications => "notifications",
            ProviderCapability::Search => "search",
            ProviderCapability::Code => "code",
            ProviderCapability::Labels => "labels",
            ProviderCapability::Templates => "templates",
            ProviderCapability::Dependencies => "dependencies",
            ProviderCapability::Advisories => "advisories",
            ProviderCapability::Attestations => "attestations",
            ProviderCapability::Secrets => "secrets",
            ProviderCapability::Variables => "variables",
            ProviderCapability::Licenses => "licenses",
            ProviderCapability::Browsing => "browsing",
            ProviderCapability::RawApi => "rawApi",
        };

        write!(f, "{s}")
    }
}

#[async_trait::async_trait]
pub trait RepoOps: Send + Sync {
    async fn list_org_repos(&self, org: &str) -> Result<Vec<RepoSummary>, GitfleetError>;
    async fn list_user_repos(&self) -> Result<Vec<RepoSummary>, GitfleetError>;
    async fn list_user_named_repos(
        &self,
        username: &str,
    ) -> Result<Vec<RepoSummary>, GitfleetError>;
    async fn get_repo(&self, repo: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn create_repo(
        &self,
        name: &str,
        visibility: &str,
        owner: Option<&str>,
        owner_type: Option<&str>,
        description: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn update_repo(
        &self,
        repo: &str,
        options: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn delete_repo(&self, repo: &str) -> Result<(), GitfleetError>;
    async fn star_repo(&self, repo: &str) -> Result<(), GitfleetError>;
    async fn unstar_repo(&self, repo: &str) -> Result<(), GitfleetError>;
    async fn fork_repo(&self, repo: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn archive_repo(&self, repo: &str) -> Result<(), GitfleetError>;
    async fn unarchive_repo(&self, repo: &str) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait ChangeOps: Send + Sync {
    async fn list_changes(
        &self,
        repo: &str,
        state: &str,
        limit: u32,
        base: Option<&str>,
        head: Option<&str>,
    ) -> Result<Vec<PullRequest>, GitfleetError>;
    async fn get_change(&self, repo: &str, number: u64) -> Result<PullRequest, GitfleetError>;
    async fn create_change(
        &self,
        repo: &str,
        title: &str,
        head: &str,
        base: &str,
        body: Option<&str>,
        draft: bool,
    ) -> Result<PullRequest, GitfleetError>;
    async fn update_change(
        &self,
        repo: &str,
        number: u64,
        options: serde_json::Value,
    ) -> Result<PullRequest, GitfleetError>;
    async fn merge_change(
        &self,
        repo: &str,
        number: u64,
        method: &str,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn comment_on_change(
        &self,
        repo: &str,
        number: u64,
        body: &str,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn list_change_comments(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn lock_change(&self, repo: &str, number: u64) -> Result<(), GitfleetError>;
    async fn unlock_change(&self, repo: &str, number: u64) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait IssueOps: Send + Sync {
    async fn get_issue(&self, repo: &str, number: u64) -> Result<serde_json::Value, GitfleetError>;
    async fn create_issue(
        &self,
        repo: &str,
        title: &str,
        body: Option<&str>,
        labels: &[String],
        assignees: &[String],
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn list_issues(
        &self,
        repo: &str,
        state: &str,
        limit: u32,
        labels: &[String],
        assignees: &[String],
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn update_issue(
        &self,
        repo: &str,
        number: u64,
        options: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn comment_on_issue(
        &self,
        repo: &str,
        number: u64,
        body: &str,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn list_issue_comments(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, GitfleetError>;
}

#[async_trait::async_trait]
pub trait ReviewOps: Send + Sync {
    async fn list_reactions_for_issue(
        &self,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<crate::types::ReactionSummary>, GitfleetError>;
    async fn create_reaction_for_issue(
        &self,
        repo: &str,
        issue_number: u64,
        content: &str,
    ) -> Result<crate::types::ReactionSummary, GitfleetError>;
    async fn delete_reaction_for_issue(
        &self,
        repo: &str,
        issue_number: u64,
        reaction_id: u64,
    ) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait PipelineOps: Send + Sync {
    async fn list_workflows(
        &self,
        repo: &str,
        limit: u32,
        page: Option<u32>,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn get_workflow(
        &self,
        repo: &str,
        workflow_id: &str,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn dispatch_pipeline(
        &self,
        repo: &str,
        definition_id: Option<&str>,
        r#ref: &str,
        inputs: Option<serde_json::Value>,
    ) -> Result<(), GitfleetError>;
    async fn list_runs(
        &self,
        repo: &str,
        filters: &str,
        limit: u32,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn get_run(&self, repo: &str, run_id: u64) -> Result<serde_json::Value, GitfleetError>;
    async fn cancel_run(&self, repo: &str, run_id: u64) -> Result<(), GitfleetError>;
    async fn rerun(&self, repo: &str, run_id: u64) -> Result<(), GitfleetError>;
    async fn delete_run(&self, repo: &str, run_id: u64) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait ReleaseOps: Send + Sync {
    async fn list_releases(
        &self,
        repo: &str,
        limit: u32,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn fetch_release_by_tag(
        &self,
        repo: &str,
        tag: &str,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn create_release(
        &self,
        repo: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn update_release(
        &self,
        repo: &str,
        release: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn delete_release(&self, repo: &str, release: &str) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait PlanningOps: Send + Sync {
    async fn list_milestones(
        &self,
        repo: &str,
        state: Option<&str>,
        limit: u32,
    ) -> Result<Vec<crate::types::Milestone>, GitfleetError>;
    async fn create_milestone(
        &self,
        repo: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<crate::types::Milestone, GitfleetError>;
    async fn get_milestone(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<crate::types::Milestone, GitfleetError>;
    async fn update_milestone(
        &self,
        repo: &str,
        number: u64,
        input: serde_json::Value,
    ) -> Result<crate::types::Milestone, GitfleetError>;
    async fn delete_milestone(&self, repo: &str, number: u64) -> Result<(), GitfleetError>;
    async fn list_projects(
        &self,
        owner: &str,
        limit: u32,
    ) -> Result<Vec<crate::types::ProjectSummary>, GitfleetError>;
    async fn get_project(&self, project_id: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn create_project(
        &self,
        owner: &str,
        title: &str,
        body: Option<&str>,
    ) -> Result<crate::types::ProjectSummary, GitfleetError>;
    async fn delete_project(&self, project_id: &str) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait WikiOps: Send + Sync {
    async fn list_wiki_pages(&self, repo: &str) -> Result<Vec<WikiPage>, GitfleetError>;
    async fn get_wiki_page(&self, repo: &str, page: &str)
        -> Result<WikiPageContent, GitfleetError>;
    async fn create_wiki_page(
        &self,
        repo: &str,
        title: &str,
        content: &str,
    ) -> Result<WikiPageContent, GitfleetError>;
    async fn update_wiki_page(
        &self,
        repo: &str,
        page: &str,
        content: &str,
    ) -> Result<WikiPageContent, GitfleetError>;
    async fn delete_wiki_page(&self, repo: &str, page: &str) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait SiteOps: Send + Sync {
    async fn get_pages(&self, repo: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn create_pages(
        &self,
        repo: &str,
        source: &str,
        build_type: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn remove_pages(&self, repo: &str) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait DiscussionOps: Send + Sync {
    async fn list_discussions(
        &self,
        owner: &str,
        name: &str,
        category_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Discussion>, GitfleetError>;
    async fn get_discussion(
        &self,
        owner: &str,
        name: &str,
        discussion_number: u64,
    ) -> Result<Discussion, GitfleetError>;
    async fn create_discussion(
        &self,
        owner: &str,
        name: &str,
        title: &str,
        body: &str,
        category_id: Option<&str>,
    ) -> Result<Discussion, GitfleetError>;
}

pub trait SecurityOps: Send + Sync {}

#[async_trait::async_trait]
pub trait RegistryOps: Send + Sync {
    async fn list_packages(
        &self,
        owner: &str,
        package_type: Option<&str>,
        limit: u32,
    ) -> Result<Vec<PackageSummary>, GitfleetError>;
    async fn get_package(
        &self,
        owner: &str,
        package_type: &str,
        package_name: &str,
    ) -> Result<serde_json::Value, GitfleetError>;
}

#[async_trait::async_trait]
pub trait DevEnvOps: Send + Sync {
    async fn list_codespaces(&self, repo: &str) -> Result<Vec<CodespaceSummary>, GitfleetError>;
    async fn create_codespace(
        &self,
        repo: &str,
        branch: Option<&str>,
    ) -> Result<CodespaceSummary, GitfleetError>;
    async fn delete_codespace(&self, repo: &str, codespace_name: &str)
        -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait DeployOps: Send + Sync {
    async fn list_deployments(
        &self,
        repo: &str,
        environment: Option<&str>,
        limit: u32,
    ) -> Result<Vec<DeploymentSummary>, GitfleetError>;
    async fn create_deployment(
        &self,
        repo: &str,
        input: serde_json::Value,
    ) -> Result<DeploymentSummary, GitfleetError>;
}

#[async_trait::async_trait]
pub trait EnvironmentOps: Send + Sync {
    async fn list_environments(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<EnvironmentListResponse, GitfleetError>;
    async fn create_environment(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        wait_timer: Option<u32>,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn delete_environment(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait RunnerOps: Send + Sync {
    async fn list_runners(&self, repo: &str) -> Result<Vec<RunnerSummary>, GitfleetError>;
    async fn remove_runner(&self, repo: &str, runner_id: u64) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait WebhookOps: Send + Sync {
    async fn list_webhooks(&self, repo: &str) -> Result<Vec<WebhookSummary>, GitfleetError>;
    async fn create_webhook(
        &self,
        repo: &str,
        input: serde_json::Value,
    ) -> Result<WebhookSummary, GitfleetError>;
    async fn remove_webhook(&self, repo: &str, hook_id: u64) -> Result<(), GitfleetError>;
    async fn test_webhook(&self, repo: &str, hook_id: u64) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait AccessOps: Send + Sync {
    async fn invite_org_member(
        &self,
        org: &str,
        username: &str,
        role: &str,
    ) -> Result<(), GitfleetError>;
    async fn invite_collaborator(
        &self,
        owner: &str,
        repo: &str,
        username: &str,
        permission: &str,
    ) -> Result<(), GitfleetError>;
    async fn list_org_members(&self, org: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn remove_org_member(&self, org: &str, username: &str) -> Result<(), GitfleetError>;
    async fn list_teams(&self, org: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn create_team(&self, org: &str, name: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn list_team_members(
        &self,
        org: &str,
        team_slug: &str,
    ) -> Result<serde_json::Value, GitfleetError>;
}

#[async_trait::async_trait]
pub trait IdentityOps: Send + Sync {
    async fn list_ssh_keys(&self) -> Result<Vec<SshKeySummary>, GitfleetError>;
    async fn add_ssh_key(&self, title: &str, key: &str) -> Result<SshKeySummary, GitfleetError>;
    async fn delete_ssh_key(&self, key_id: u64) -> Result<(), GitfleetError>;
    async fn list_gpg_keys(&self) -> Result<Vec<GpgKeySummary>, GitfleetError>;
    async fn add_gpg_key(&self, armored_key: &str) -> Result<GpgKeySummary, GitfleetError>;
    async fn delete_gpg_key(&self, key_id: u64) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait AnalyticsOps: Send + Sync {
    async fn get_traffic_views(&self, repo: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn get_traffic_clones(&self, repo: &str) -> Result<serde_json::Value, GitfleetError>;
}

#[async_trait::async_trait]
pub trait SnippetOps: Send + Sync {
    async fn list_snippets(&self, owner: &str) -> Result<Vec<GistSummary>, GitfleetError>;
    async fn get_snippet(&self, gist_id: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn create_snippet(
        &self,
        description: &str,
        public: bool,
        files: serde_json::Value,
    ) -> Result<GistSummary, GitfleetError>;
    async fn delete_snippet(&self, gist_id: &str) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait GovernanceOps: Send + Sync {
    async fn list_rulesets(&self, repo: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn create_ruleset(
        &self,
        repo: &str,
        input: &RulesetInput,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn delete_ruleset(&self, repo: &str, ruleset_id: u64) -> Result<(), GitfleetError>;
}

pub trait MergeAutomationOps: Send + Sync {}
#[async_trait::async_trait]
pub trait PolicyOps: Send + Sync {
    async fn get_branch_protection(
        &self,
        repo: &str,
        branch: &str,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn protect_branch(
        &self,
        repo: &str,
        branch: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn unprotect_branch(&self, repo: &str, branch: &str) -> Result<(), GitfleetError>;
    async fn list_tag_protection(
        &self,
        repo: &str,
    ) -> Result<Vec<crate::types::TagProtection>, GitfleetError>;
    async fn create_tag_protection(
        &self,
        repo: &str,
        pattern: &str,
    ) -> Result<crate::types::TagProtection, GitfleetError>;
    async fn delete_tag_protection(
        &self,
        repo: &str,
        identifier: &str,
    ) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait NotificationOps: Send + Sync {
    async fn list_notifications(
        &self,
        all: bool,
        participating: bool,
        repo: Option<&str>,
    ) -> Result<Vec<Notification>, GitfleetError>;
    async fn mark_notifications_read(&self) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait LabelOps: Send + Sync {
    async fn list_labels(&self, repo: &str) -> Result<Vec<Label>, GitfleetError>;
    async fn create_label(
        &self,
        label: &Label,
        repo: &str,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn delete_label(&self, name: &str, repo: &str) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait SearchOps: Send + Sync {
    async fn search_issues(
        &self,
        query: &str,
        sort: Option<&str>,
        order: Option<&str>,
        limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError>;
    async fn search_repos(
        &self,
        query: &str,
        sort: Option<&str>,
        order: Option<&str>,
        limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError>;
    async fn search_code(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<SearchResult<serde_json::Value>, GitfleetError>;
}

#[async_trait::async_trait]
pub trait CodeOps: Send + Sync {
    async fn get_file_contents(
        &self,
        repo: &str,
        path: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn search_code(
        &self,
        query: &str,
        repo: Option<&str>,
        language: Option<&str>,
        limit: u32,
    ) -> Result<Vec<CodeSearchResult>, GitfleetError>;
}

#[async_trait::async_trait]
pub trait TemplateOps: Send + Sync {
    async fn list_issue_templates(&self, repo: &str) -> Result<Vec<IssueTemplate>, GitfleetError>;
}

#[async_trait::async_trait]
pub trait DependencyOps: Send + Sync {
    async fn sbom(&self, repo: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn review_dependencies(
        &self,
        repo: &str,
        base: &str,
        head: &str,
    ) -> Result<Vec<DependencyReviewChange>, GitfleetError>;
}

#[async_trait::async_trait]
pub trait AdvisoryOps: Send + Sync {
    async fn list_dependabot_alerts(
        &self,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn list_codeql_alerts(
        &self,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn list_secret_scanning_alerts(
        &self,
        repo: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn get_dependabot_alert(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<serde_json::Value, GitfleetError>;
}

#[async_trait::async_trait]
pub trait AttestationOps: Send + Sync {
    async fn list_attestations(
        &self,
        repo: &str,
        subject_digest: &str,
    ) -> Result<serde_json::Value, GitfleetError>;
}

#[async_trait::async_trait]
pub trait SecretOps: Send + Sync {
    async fn list_repo_secrets(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<SecretListResponse<RepoSecret>, GitfleetError>;
    async fn get_repo_public_key(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<PublicKeyResponse, GitfleetError>;
    async fn set_repo_secret(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        encrypted_value: &str,
        key_id: &str,
    ) -> Result<(), GitfleetError>;
    async fn delete_repo_secret(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait VariableOps: Send + Sync {
    async fn list_repo_variables(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<VariableListResponse<RepoVariable>, GitfleetError>;
    async fn set_repo_variable(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        value: &str,
    ) -> Result<(), GitfleetError>;
    async fn delete_repo_variable(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
    ) -> Result<(), GitfleetError>;
}

#[async_trait::async_trait]
pub trait LicenseOps: Send + Sync {
    async fn list_licenses(&self) -> Result<Vec<LicenseSummary>, GitfleetError>;
    async fn get_license(&self, key: &str) -> Result<LicenseDetail, GitfleetError>;
    async fn repo_license(&self, repo: &str) -> Result<serde_json::Value, GitfleetError>;
}

#[async_trait::async_trait]
pub trait BrowseOps: Send + Sync {
    async fn list_contents(
        &self,
        repo: &str,
        path: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn file_contents(
        &self,
        repo: &str,
        path: &str,
        r#ref: Option<&str>,
    ) -> Result<serde_json::Value, GitfleetError>;
}

#[async_trait::async_trait]
pub trait RawApiOps: Send + Sync {
    async fn raw_get(&self, endpoint: &str) -> Result<serde_json::Value, GitfleetError>;
    async fn raw_post(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, GitfleetError>;
    async fn raw_delete(&self, endpoint: &str) -> Result<serde_json::Value, GitfleetError>;
}

pub trait GitProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn default_host(&self) -> &'static str;
    fn capabilities(&self) -> &[ProviderCapability];

    fn repo_ops(&self) -> Option<&dyn RepoOps> {
        None
    }

    fn change_ops(&self) -> Option<&dyn ChangeOps> {
        None
    }

    fn review_ops(&self) -> Option<&dyn ReviewOps> {
        None
    }

    fn issue_ops(&self) -> Option<&dyn IssueOps> {
        None
    }

    fn pipeline_ops(&self) -> Option<&dyn PipelineOps> {
        None
    }

    fn release_ops(&self) -> Option<&dyn ReleaseOps> {
        None
    }

    fn planning_ops(&self) -> Option<&dyn PlanningOps> {
        None
    }

    fn wiki_ops(&self) -> Option<&dyn WikiOps> {
        None
    }

    fn site_ops(&self) -> Option<&dyn SiteOps> {
        None
    }

    fn discussion_ops(&self) -> Option<&dyn DiscussionOps> {
        None
    }

    fn security_ops(&self) -> Option<&dyn SecurityOps> {
        None
    }

    fn registry_ops(&self) -> Option<&dyn RegistryOps> {
        None
    }

    fn dev_env_ops(&self) -> Option<&dyn DevEnvOps> {
        None
    }

    fn deploy_ops(&self) -> Option<&dyn DeployOps> {
        None
    }

    fn environment_ops(&self) -> Option<&dyn EnvironmentOps> {
        None
    }

    fn runner_ops(&self) -> Option<&dyn RunnerOps> {
        None
    }

    fn webhook_ops(&self) -> Option<&dyn WebhookOps> {
        None
    }

    fn access_ops(&self) -> Option<&dyn AccessOps> {
        None
    }

    fn identity_ops(&self) -> Option<&dyn IdentityOps> {
        None
    }

    fn analytics_ops(&self) -> Option<&dyn AnalyticsOps> {
        None
    }

    fn snippet_ops(&self) -> Option<&dyn SnippetOps> {
        None
    }

    fn governance_ops(&self) -> Option<&dyn GovernanceOps> {
        None
    }

    fn merge_automation_ops(&self) -> Option<&dyn MergeAutomationOps> {
        None
    }

    fn policy_ops(&self) -> Option<&dyn PolicyOps> {
        None
    }

    fn notification_ops(&self) -> Option<&dyn NotificationOps> {
        None
    }

    fn search_ops(&self) -> Option<&dyn SearchOps> {
        None
    }

    fn code_ops(&self) -> Option<&dyn CodeOps> {
        None
    }

    fn label_ops(&self) -> Option<&dyn LabelOps> {
        None
    }

    fn template_ops(&self) -> Option<&dyn TemplateOps> {
        None
    }

    fn dependency_ops(&self) -> Option<&dyn DependencyOps> {
        None
    }

    fn advisory_ops(&self) -> Option<&dyn AdvisoryOps> {
        None
    }

    fn attestation_ops(&self) -> Option<&dyn AttestationOps> {
        None
    }

    fn secret_ops(&self) -> Option<&dyn SecretOps> {
        None
    }

    fn variable_ops(&self) -> Option<&dyn VariableOps> {
        None
    }

    fn license_ops(&self) -> Option<&dyn LicenseOps> {
        None
    }

    fn browse_ops(&self) -> Option<&dyn BrowseOps> {
        None
    }

    fn raw_api_ops(&self) -> Option<&dyn RawApiOps> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{AccountRef, RepositoryRef};

    #[test]
    fn test_provider_id_display_github() {
        assert_eq!(format!("{}", ProviderId::GitHub), "github");
    }

    #[test]
    fn test_provider_id_display_gitlab() {
        assert_eq!(format!("{}", ProviderId::GitLab), "gitlab");
    }

    #[test]
    fn test_provider_id_equality() {
        assert_eq!(ProviderId::GitHub, ProviderId::GitHub);

        assert_ne!(ProviderId::GitHub, ProviderId::GitLab);
    }

    #[test]
    fn test_provider_capability_display() {
        assert_eq!(
            format!("{}", ProviderCapability::Repositories),
            "repositories"
        );

        assert_eq!(format!("{}", ProviderCapability::Changes), "changes");

        assert_eq!(format!("{}", ProviderCapability::Reviews), "reviews");
        assert_eq!(format!("{}", ProviderCapability::Issues), "issues");

        assert_eq!(format!("{}", ProviderCapability::Pipelines), "pipelines");
        assert_eq!(format!("{}", ProviderCapability::Releases), "releases");

        assert_eq!(format!("{}", ProviderCapability::Milestones), "milestones");
        assert_eq!(format!("{}", ProviderCapability::Projects), "projects");
        assert_eq!(format!("{}", ProviderCapability::Wiki), "wiki");

        assert_eq!(format!("{}", ProviderCapability::Site), "site");
        assert_eq!(
            format!("{}", ProviderCapability::Discussions),
            "discussions"
        );

        assert_eq!(format!("{}", ProviderCapability::Security), "security");

        assert_eq!(format!("{}", ProviderCapability::Registry), "registry");
        assert_eq!(
            format!("{}", ProviderCapability::DevelopmentEnvironments),
            "developmentEnvironments"
        );

        assert_eq!(
            format!("{}", ProviderCapability::Deployments),
            "deployments"
        );

        assert_eq!(
            format!("{}", ProviderCapability::Environments),
            "environments"
        );

        assert_eq!(format!("{}", ProviderCapability::Runners), "runners");

        assert_eq!(format!("{}", ProviderCapability::Webhooks), "webhooks");
        assert_eq!(format!("{}", ProviderCapability::Access), "access");

        assert_eq!(format!("{}", ProviderCapability::Identity), "identity");
        assert_eq!(format!("{}", ProviderCapability::Analytics), "analytics");

        assert_eq!(format!("{}", ProviderCapability::Snippets), "snippets");
        assert_eq!(format!("{}", ProviderCapability::Governance), "governance");

        assert_eq!(
            format!("{}", ProviderCapability::MergeAutomation),
            "mergeAutomation"
        );

        assert_eq!(
            format!("{}", ProviderCapability::RepositoryPolicies),
            "repositoryPolicies"
        );

        assert_eq!(
            format!("{}", ProviderCapability::Notifications),
            "notifications"
        );

        assert_eq!(format!("{}", ProviderCapability::Search), "search");

        assert_eq!(format!("{}", ProviderCapability::Code), "code");
        assert_eq!(format!("{}", ProviderCapability::Labels), "labels");

        assert_eq!(format!("{}", ProviderCapability::Templates), "templates");
        assert_eq!(
            format!("{}", ProviderCapability::Dependencies),
            "dependencies"
        );

        assert_eq!(format!("{}", ProviderCapability::Advisories), "advisories");

        assert_eq!(
            format!("{}", ProviderCapability::Attestations),
            "attestations"
        );

        assert_eq!(format!("{}", ProviderCapability::Secrets), "secrets");

        assert_eq!(format!("{}", ProviderCapability::Variables), "variables");
        assert_eq!(format!("{}", ProviderCapability::Licenses), "licenses");

        assert_eq!(format!("{}", ProviderCapability::Browsing), "browsing");
        assert_eq!(format!("{}", ProviderCapability::RawApi), "rawApi");
    }

    #[test]
    fn test_repository_ref_full_name() {
        let r = RepositoryRef {
            provider: ProviderId::GitHub,
            host: "github.com".to_string(),
            namespace: "org".to_string(),
            name: "repo".to_string(),
        };

        assert_eq!(r.full_name(), "org/repo");
    }

    #[test]
    fn test_repository_ref_qualified() {
        let r = RepositoryRef {
            provider: ProviderId::GitHub,
            host: "github.com".to_string(),
            namespace: "org".to_string(),
            name: "repo".to_string(),
        };

        assert_eq!(r.qualified(), "github@github.com:org/repo");
    }

    #[test]
    fn test_repository_ref_gitlab() {
        let r = RepositoryRef {
            provider: ProviderId::GitLab,
            host: "gitlab.com".to_string(),
            namespace: "group".to_string(),
            name: "project".to_string(),
        };

        assert_eq!(r.qualified(), "gitlab@gitlab.com:group/project");
    }

    #[test]
    fn test_account_ref_fields() {
        let a = AccountRef {
            provider: ProviderId::GitHub,
            host: "github.com".to_string(),
            profile: "default".to_string(),
        };

        assert_eq!(a.provider, ProviderId::GitHub);

        assert_eq!(a.host, "github.com");
        assert_eq!(a.profile, "default");
    }

    #[test]
    fn test_provider_capability_equality() {
        assert_eq!(
            ProviderCapability::Repositories,
            ProviderCapability::Repositories
        );

        assert_ne!(
            ProviderCapability::Repositories,
            ProviderCapability::Changes
        );
    }

    #[test]
    fn test_default_git_provider_returns_none() {
        struct NullProvider;
        impl GitProvider for NullProvider {
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

        let p = NullProvider;

        assert!(p.repo_ops().is_none());

        assert!(p.change_ops().is_none());
        assert!(p.review_ops().is_none());

        assert!(p.issue_ops().is_none());
        assert!(p.pipeline_ops().is_none());

        assert!(p.release_ops().is_none());
        assert!(p.planning_ops().is_none());

        assert!(p.wiki_ops().is_none());
        assert!(p.site_ops().is_none());

        assert!(p.discussion_ops().is_none());
        assert!(p.security_ops().is_none());

        assert!(p.registry_ops().is_none());
        assert!(p.dev_env_ops().is_none());

        assert!(p.deploy_ops().is_none());
        assert!(p.environment_ops().is_none());

        assert!(p.runner_ops().is_none());
        assert!(p.webhook_ops().is_none());

        assert!(p.access_ops().is_none());
        assert!(p.identity_ops().is_none());

        assert!(p.analytics_ops().is_none());
        assert!(p.snippet_ops().is_none());

        assert!(p.governance_ops().is_none());
        assert!(p.merge_automation_ops().is_none());

        assert!(p.policy_ops().is_none());
        assert!(p.notification_ops().is_none());

        assert!(p.search_ops().is_none());
        assert!(p.code_ops().is_none());

        assert!(p.label_ops().is_none());
        assert!(p.template_ops().is_none());

        assert!(p.dependency_ops().is_none());
        assert!(p.advisory_ops().is_none());

        assert!(p.attestation_ops().is_none());
        assert!(p.secret_ops().is_none());

        assert!(p.variable_ops().is_none());
        assert!(p.license_ops().is_none());

        assert!(p.browse_ops().is_none());
        assert!(p.raw_api_ops().is_none());
    }

    #[test]
    fn test_provider_id_serialization() {
        let json = serde_json::to_string(&ProviderId::GitHub).unwrap();

        assert!(json.contains("GitHub"));

        let gitlab_json = serde_json::to_string(&ProviderId::GitLab).unwrap();

        assert!(gitlab_json.contains("GitLab"));
    }

    #[test]
    fn test_provider_id_deserialization() {
        let github: ProviderId = serde_json::from_str("\"GitHub\"").unwrap();

        assert_eq!(github, ProviderId::GitHub);

        let gitlab: ProviderId = serde_json::from_str("\"GitLab\"").unwrap();

        assert_eq!(gitlab, ProviderId::GitLab);
    }

    #[test]
    fn test_provider_context_tracks_capabilities() {
        let context = ProviderContext {
            profile_name: "work".to_string(),
            provider: ProviderId::GitLab,
            host: "git.example.com".to_string(),
            token: Some("token".to_string()),
            token_source: TokenSource::Profile,
            capabilities: Vec::new(),
        }
        .with_capabilities(&[ProviderCapability::Repositories]);

        assert_eq!(context.profile_name, "work");
        assert_eq!(context.provider, ProviderId::GitLab);
        assert_eq!(context.token_source, TokenSource::Profile);
        assert_eq!(context.capabilities, vec![ProviderCapability::Repositories]);
    }

    #[test]
    fn test_repository_ref_serialization() {
        let r = RepositoryRef {
            provider: ProviderId::GitHub,
            host: "github.com".to_string(),
            namespace: "org".to_string(),
            name: "repo".to_string(),
        };

        let json = serde_json::to_string(&r).unwrap();

        let deserialized: RepositoryRef = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.namespace, "org");

        assert_eq!(deserialized.name, "repo");
    }
}
