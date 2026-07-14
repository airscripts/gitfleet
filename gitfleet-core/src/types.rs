use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_name: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSummary {
    pub id: u64,
    pub name: String,
    pub fork: bool,
    pub full_name: String,
    pub private: bool,
    pub archived: bool,
    pub default_branch: String,
    pub pushed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInspectResult {
    pub score: u32,
    pub present: Vec<String>,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretScanFinding {
    pub file: String,
    pub rule: String,
    pub r#match: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    pub confidence: SecretScanConfidence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SecretScanConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretScanningAlert {
    pub url: String,
    pub state: String,
    pub number: u64,
    pub created_at: String,
    pub secret_type: String,
    pub repository: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    pub secret_type_display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependabotAlert {
    pub state: String,
    pub number: u64,
    pub severity: String,
    pub advisory: String,
    pub ecosystem: String,
    pub repository: String,
    pub package_name: String,
    pub manifest_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismissed_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ComplianceCheckStatus {
    Pass,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub id: String,
    pub label: String,
    pub message: String,
    pub status: ComplianceCheckStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub repo: String,
    pub score: u32,
    pub remediation: Vec<String>,
    pub checks: Vec<ComplianceCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkRepoResult<T = serde_json::Value> {
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkRepoMetadata<T = serde_json::Value> {
    pub failed: usize,
    pub completed: usize,
    pub results: Vec<BulkRepoResult<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTargetOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repos: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extra: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialsFile {
    pub active_profile: String,
    pub profiles: std::collections::HashMap<String, Profile>,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub aliases: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRcFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub login: String,
    pub html_url: String,
    pub avatar_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub user: AuthUser,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub number: u64,
    pub html_url: String,
    pub open_issues: u64,
    pub state: MilestoneState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    pub closed_issues: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MilestoneState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneProgress {
    pub title: String,
    pub total: u64,
    pub percent: u64,
    pub open_issues: u64,
    pub closed_issues: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<IssueUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_lock_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<IssueUser>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<LabelEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueUser {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub title: String,
    pub state: String,
    pub number: u64,
    #[serde(default)]
    pub merged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mergeable_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mergeable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<PullRequestUser>,
    #[serde(default)]
    pub maintainer_can_modify: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_commit_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<LabelEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_reviewers: Option<Vec<PullRequestUser>>,
    pub head: PullRequestHead,
    pub base: PullRequestBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestUser {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestHead {
    pub r#ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<PullRequestRepo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestBase {
    pub r#ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<PullRequestRepo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestRepo {
    pub full_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMergeSettings {
    pub default_branch: String,
    pub allow_rebase_merge: bool,
    pub allow_squash_merge: bool,
    pub allow_merge_commit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSummary {
    pub id: u64,
    pub r#ref: String,
    pub environment: String,
    pub task: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    pub created_at: String,
    pub production: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatusSummary {
    pub id: u64,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtection {
    pub pattern: String,
    pub required_checks: Vec<String>,
    pub required_reviews: u32,
    pub dismiss_stale: bool,
    pub enforce_admins: bool,
    pub allow_force_pushes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagProtection {
    pub identifier: String,
    pub pattern: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSummary {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: u64,
    pub guid: String,
    pub delivered_at: String,
    pub status_code: u32,
    pub duration: u64,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewThread {
    pub id: u64,
    pub path: String,
    pub line: u32,
    pub resolved: bool,
    pub comments: Vec<ReviewComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub id: u64,
    pub body: String,
    pub path: String,
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_hunk: Option<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to_id: Option<u64>,
    pub side: String,
    pub user: ReviewCommentUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCommentUser {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSuggestion {
    pub id: u64,
    pub path: String,
    pub line: u32,
    pub original_text: String,
    pub suggested_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewApplyResult {
    pub branch: String,
    pub applied: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionSummary {
    pub id: u64,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentSummary {
    pub id: u64,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEntry {
    pub name: String,
    pub version: String,
    pub ecosystem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyReviewChange {
    pub change_type: String,
    pub package: String,
    pub ecosystem: String,
    pub version: String,
    pub severity: String,
    pub vulnerabilities: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisorySummary {
    pub ghsa_id: String,
    pub summary: String,
    pub severity: String,
    pub ecosystem: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cve_id: Option<String>,
    pub published_at: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisoryCreateInput {
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cve_id: Option<String>,
    pub summary: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerable_version_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patched_version_range: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQLAlertSummary {
    pub number: u64,
    pub rule: String,
    pub severity: String,
    pub state: String,
    pub tool: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSearchResult {
    pub file: String,
    pub repo: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameEntry {
    pub sha: String,
    pub author: String,
    pub date: String,
    pub message: String,
    pub pr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTemplate {
    pub name: String,
    pub filename: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSummary {
    pub id: u64,
    pub name: String,
    pub package_type: String,
    pub visibility: String,
    pub url: String,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub owner: String,
    pub repository: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersion {
    pub id: u64,
    pub name: String,
    pub version: String,
    pub url: String,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerSummary {
    pub id: u64,
    pub name: String,
    pub os: String,
    pub status: String,
    pub busy: bool,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerLabel {
    pub id: u64,
    pub name: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodespaceSummary {
    pub id: u64,
    pub name: String,
    pub state: String,
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub created_at: String,
    pub idle_timeout_minutes: u32,
    pub machine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationSummary {
    pub bundle_type: String,
    pub predicate_type: String,
    pub digest: String,
    pub repository_id: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeySummary {
    pub id: u64,
    pub title: String,
    pub key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgKeySummary {
    pub id: u64,
    pub name: String,
    pub key_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasEntry {
    pub name: String,
    pub expansion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseSummary {
    pub key: String,
    pub name: String,
    pub spdx_id: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseDetail {
    pub key: String,
    pub name: String,
    pub spdx_id: String,
    pub url: String,
    pub description: String,
    pub implementation: String,
    pub permissions: Vec<String>,
    pub conditions: Vec<String>,
    pub limitations: Vec<String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistFile {
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub raw_url: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistSummary {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub public: bool,
    pub html_url: String,
    pub git_pull_url: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub files: Vec<GistFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionsCacheEntry {
    pub id: u64,
    pub key: String,
    pub r#ref: String,
    pub version: String,
    pub created_at: String,
    pub size_in_bytes: u64,
    pub last_accessed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSummary {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowValidationIssue {
    pub file: String,
    pub rule: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    pub message: String,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowValidateResult {
    pub file: String,
    pub valid: bool,
    pub issues: Vec<WorkflowValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDryRunJob {
    pub id: String,
    pub needs: Vec<String>,
    pub matrix: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runs_on: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDryRunResult {
    pub file: String,
    pub triggers: Vec<String>,
    pub jobs: Vec<WorkflowDryRunJob>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    pub unresolved_expressions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDebugJob {
    pub id: u64,
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_run_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDebugArtifact {
    pub id: u64,
    pub name: String,
    pub size_in_bytes: u64,
    pub archive_download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDebugResult {
    pub repo: String,
    pub run_id: u64,
    pub status: String,
    pub output_dir: String,
    pub jobs: Vec<RunDebugJob>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<String>,
    pub artifacts: Vec<RunDebugArtifact>,
    pub annotations: Vec<RunDebugAnnotation>,
    pub files: RunDebugFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDebugAnnotation {
    pub path: String,
    pub message: String,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDebugFiles {
    pub artifacts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs_zip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBoard {
    pub owner: String,
    pub title: String,
    pub number: u64,
    pub columns: Vec<ProjectBoardColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBoardColumn {
    pub name: String,
    pub items: Vec<ProjectBoardItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBoardItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub r#type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: String,
    pub number: u64,
    pub title: String,
    pub description: String,
    pub closed: bool,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectItem {
    pub id: String,
    pub r#type: String,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectField {
    pub id: String,
    pub name: String,
    pub data_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ProjectFieldOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFieldOption {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesetInput {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforcement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub repository: String,
    pub subject_title: String,
    pub subject_type: String,
    pub reason: String,
    pub unread: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityResult {
    pub assigned_issues: Vec<Notification>,
    pub review_requests: Vec<Notification>,
    pub recent_mentions: Vec<Notification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub incomplete_results: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSearchItem {
    pub id: u64,
    pub title: String,
    pub state: String,
    pub number: u64,
    pub html_url: String,
    pub is_pull_request: bool,
    pub repository_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<IssueUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<LabelEntry>>,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub comments: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<IssueUser>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSearchItem {
    pub id: u64,
    pub name: String,
    pub score: f64,
    pub html_url: String,
    pub private: bool,
    pub full_name: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub forks_count: u64,
    pub archived: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub stargazers_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSearchItem {
    pub name: String,
    pub path: String,
    pub score: f64,
    pub html_url: String,
    pub repository: CodeSearchRepo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSearchRepo {
    pub full_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSearchItem {
    pub sha: String,
    pub score: f64,
    pub html_url: String,
    pub date: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<CommitSearchAuthor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSearchAuthor {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discussion {
    pub id: String,
    pub url: String,
    pub body: String,
    pub title: String,
    pub number: u64,
    pub author: String,
    pub closed: bool,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
    pub comments_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionCategory {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionComment {
    pub id: String,
    pub body: String,
    pub author: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: u64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_timer: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protection_rules: Option<Vec<EnvironmentProtectionRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentProtectionRule {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_timer: Option<u32>,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewers: Option<Vec<EnvironmentReviewer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_policy: Option<EnvironmentBranchPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentReviewer {
    pub r#type: String,
    pub reviewer: EnvironmentReviewerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentReviewerInfo {
    pub id: u64,
    pub login: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentBranchPolicy {
    pub protected_branches: bool,
    pub custom_branch_policies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentListResponse {
    pub total_count: u64,
    pub environments: Vec<Environment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesSite {
    pub url: String,
    pub status: String,
    pub html_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PagesSource>,
    pub https_enforced: bool,
    pub build_type: PagesBuildType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PagesBuildType {
    Legacy,
    Workflow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesSource {
    pub branch: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesBuild {
    pub url: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub path: String,
    pub title: String,
    pub format: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPageContent {
    #[serde(flatten)]
    pub page: WikiPage,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSecret {
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgSecret {
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub visibility: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_repositories_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSecret {
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SecretVisibility {
    All,
    Private,
    Selected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSecretInput {
    pub encrypted_value: String,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyResponse {
    pub key_id: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretListResponse<T> {
    pub total_count: u64,
    pub secrets: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoVariable {
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgVariable {
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub visibility: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableListResponse<T> {
    pub total_count: u64,
    pub variables: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistComment {
    pub id: u64,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) -> T {
        let json = serde_json::to_string(value).unwrap();

        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn test_label_round_trip() {
        let label = Label {
            name: "bug".into(),
            color: "ff0000".into(),
            new_name: None,
            description: "Bug report".into(),
        };

        let result = round_trip(&label);

        assert_eq!(result.name, "bug");

        assert_eq!(result.color, "ff0000");
        assert!(result.new_name.is_none());

        assert_eq!(result.description, "Bug report");
    }

    #[test]
    fn test_label_with_new_name() {
        let label = Label {
            name: "old".into(),
            color: "000000".into(),
            new_name: Some("new".into()),
            description: "renamed".into(),
        };

        let result = round_trip(&label);

        assert_eq!(result.new_name.as_deref(), Some("new"));
    }

    #[test]
    fn test_repo_summary_round_trip() {
        let repo = RepoSummary {
            id: 42,
            name: "my-repo".into(),
            fork: false,
            full_name: "org/my-repo".into(),
            private: true,
            archived: false,
            default_branch: "main".into(),
            pushed_at: Some("2025-01-01T00:00:00Z".into()),
        };

        let result = round_trip(&repo);

        assert_eq!(result.id, 42);

        assert_eq!(result.name, "my-repo");
        assert!(!result.fork);

        assert!(result.private);
        assert!(!result.archived);
    }

    #[test]
    fn test_repo_summary_optional_fields() {
        let json = r#"{"id":1,"name":"r","fork":false,"full_name":"o/r","private":false,"archived":false,"default_branch":"main","pushed_at":null}"#;
        let repo: RepoSummary = serde_json::from_str(json).unwrap();

        assert!(repo.pushed_at.is_none());
    }

    #[test]
    fn test_pull_request_round_trip() {
        let pr = PullRequest {
            title: "Fix bug".into(),
            state: "open".into(),
            number: 123,
            merged: false,
            draft: Some(true),
            html_url: Some("https://github.com/o/r/pull/123".into()),
            created_at: Some("2025-01-01".into()),
            updated_at: None,
            body: Some("description".into()),
            mergeable_state: Some("clean".into()),
            merged_at: None,
            mergeable: Some(true),
            user: Some(PullRequestUser {
                login: "user".into(),
            }),
            maintainer_can_modify: true,
            merge_commit_sha: None,
            labels: Some(vec![]),
            requested_reviewers: None,
            head: PullRequestHead {
                r#ref: "feature".into(),
                sha: Some("abc123".into()),
                repo: Some(PullRequestRepo {
                    full_name: "user/feature".into(),
                    html_url: None,
                }),
            },
            base: PullRequestBase {
                r#ref: "main".into(),
                repo: None,
            },
        };

        let result = round_trip(&pr);

        assert_eq!(result.number, 123);

        assert_eq!(result.head.r#ref, "feature");
        assert_eq!(result.base.r#ref, "main");

        assert!(result.draft.unwrap());
    }

    #[test]
    fn test_notification_round_trip() {
        let n = Notification {
            id: "1".into(),
            repository: "org/repo".into(),
            subject_title: "PR #1".into(),
            subject_type: "PullRequest".into(),
            reason: "subscribed".into(),
            unread: true,
            updated_at: "2025-01-01".into(),
        };

        let result = round_trip(&n);

        assert_eq!(result.id, "1");

        assert!(result.unread);
    }

    #[test]
    fn test_auth_status_round_trip() {
        let status = AuthStatus {
            user: AuthUser {
                login: "octocat".into(),
                html_url: "https://github.com/octocat".into(),
                avatar_url: "https://github.com/avatar".into(),
                name: Some("Octocat".into()),
            },
            scopes: vec!["repo".into(), "read:org".into()],
        };

        let result = round_trip(&status);

        assert_eq!(result.user.login, "octocat");

        assert_eq!(result.scopes.len(), 2);
    }

    #[test]
    fn test_auth_user_optional_name() {
        let user = AuthUser {
            login: "bot".into(),
            html_url: "https://github.com/bot".into(),
            avatar_url: "https://github.com/bot.png".into(),
            name: None,
        };

        let result = round_trip(&user);

        assert!(result.name.is_none());
    }

    #[test]
    fn test_webhook_summary_round_trip() {
        let wh = WebhookSummary {
            id: 99,
            name: "web".into(),
            url: "https://example.com/hook".into(),
            events: vec!["push".into(), "pull_request".into()],
            active: true,
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-02".into(),
        };

        let result = round_trip(&wh);

        assert_eq!(result.id, 99);

        assert_eq!(result.events.len(), 2);
    }

    #[test]
    fn test_workflow_summary_round_trip() {
        let ws = WorkflowSummary {
            id: 7,
            name: "CI".into(),
            path: ".github/workflows/ci.yml".into(),
            state: "active".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-02".into(),
            html_url: "https://github.com/o/r/actions".into(),
        };

        let result = round_trip(&ws);

        assert_eq!(result.name, "CI");
    }

    #[test]
    fn test_ruleset_input_round_trip() {
        let ri = RulesetInput {
            name: "main-rules".into(),
            target: Some("branch".into()),
            rules: None,
            enforcement: Some("active".into()),
            conditions: None,
        };

        let result = round_trip(&ri);

        assert_eq!(result.name, "main-rules");

        assert!(result.rules.is_none());
    }

    #[test]
    fn test_secret_list_response_round_trip() {
        let slr: SecretListResponse<RepoSecret> = SecretListResponse {
            total_count: 2,
            secrets: vec![
                RepoSecret {
                    name: "TOKEN".into(),
                    created_at: "2025-01-01".into(),
                    updated_at: "2025-01-01".into(),
                },
                RepoSecret {
                    name: "KEY".into(),
                    created_at: "2025-01-02".into(),
                    updated_at: "2025-01-02".into(),
                },
            ],
        };

        let result = round_trip(&slr);

        assert_eq!(result.total_count, 2);

        assert_eq!(result.secrets.len(), 2);
    }

    #[test]
    fn test_variable_list_response_round_trip() {
        let vlr: VariableListResponse<RepoVariable> = VariableListResponse {
            total_count: 1,
            variables: vec![RepoVariable {
                name: "ENV".into(),
                created_at: "2025-01-01".into(),
                updated_at: "2025-01-01".into(),
                value: Some("prod".into()),
            }],
        };

        let result = round_trip(&vlr);

        assert_eq!(result.total_count, 1);

        assert_eq!(result.variables[0].name, "ENV");
    }

    #[test]
    fn test_search_result_round_trip() {
        let sr: SearchResult<RepoSearchItem> = SearchResult {
            items: vec![],
            total_count: 0,
            incomplete_results: false,
        };

        let result = round_trip(&sr);

        assert!(result.items.is_empty());

        assert_eq!(result.total_count, 0);
    }

    #[test]
    fn test_issue_search_item_round_trip() {
        let item = IssueSearchItem {
            id: 1,
            title: "Bug".into(),
            state: "open".into(),
            number: 42,
            html_url: "https://github.com/o/r/issues/42".into(),
            is_pull_request: false,
            repository_url: "https://github.com/o/r".into(),
            user: None,
            labels: None,
            score: 1.5,
            body: None,
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-02".into(),
            comments: 3,
            assignees: None,
        };

        let result = round_trip(&item);

        assert_eq!(result.id, 1);

        assert!(!result.is_pull_request);
        assert_eq!(result.score, 1.5);
    }

    #[test]
    fn test_repo_search_item_round_trip() {
        let item = RepoSearchItem {
            id: 10,
            name: "repo".into(),
            score: 99.9,
            html_url: "https://github.com/o/r".into(),
            private: false,
            full_name: "o/r".into(),
            updated_at: "2025-01-01".into(),
            language: Some("Rust".into()),
            forks_count: 5,
            archived: false,
            description: Some("A repo".into()),
            stargazers_count: 100,
        };

        let result = round_trip(&item);

        assert_eq!(result.language.as_deref(), Some("Rust"));

        assert_eq!(result.stargazers_count, 100);
    }

    #[test]
    fn test_environment_round_trip() {
        let env = Environment {
            id: 1,
            name: "production".into(),
            url: Some("https://example.com".into()),
            html_url: "https://github.com/o/r/settings/environments/1".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
            wait_timer: Some(30),
            protection_rules: None,
        };

        let result = round_trip(&env);

        assert_eq!(result.name, "production");

        assert_eq!(result.wait_timer, Some(30));
    }

    #[test]
    fn test_environment_list_response_round_trip() {
        let elr = EnvironmentListResponse {
            total_count: 0,
            environments: vec![],
        };

        let result = round_trip(&elr);

        assert_eq!(result.total_count, 0);

        assert!(result.environments.is_empty());
    }

    #[test]
    fn test_deployment_summary_round_trip() {
        let d = DeploymentSummary {
            id: 1,
            r#ref: "main".into(),
            environment: "production".into(),
            task: "deploy".into(),
            description: Some("deploy main".into()),
            creator: Some("octocat".into()),
            created_at: "2025-01-01".into(),
            production: true,
        };

        let result = round_trip(&d);

        assert!(result.production);

        assert_eq!(result.r#ref, "main");
    }

    #[test]
    fn test_runner_summary_round_trip() {
        let r = RunnerSummary {
            id: 1,
            name: "runner-1".into(),
            os: "linux".into(),
            status: "online".into(),
            busy: false,
            labels: vec!["self-hosted".into()],
        };

        let result = round_trip(&r);

        assert!(!result.busy);

        assert_eq!(result.labels.len(), 1);
    }

    #[test]
    fn test_ssh_key_summary_round_trip() {
        let k = SshKeySummary {
            id: 1,
            title: "my-key".into(),
            key: "ssh-ed25519 AAAA".into(),
            created_at: "2025-01-01".into(),
        };

        let result = round_trip(&k);

        assert_eq!(result.title, "my-key");
    }

    #[test]
    fn test_gpg_key_summary_round_trip() {
        let k = GpgKeySummary {
            id: 1,
            name: "gpg-key".into(),
            key_id: "ABCD1234".into(),
            created_at: "2025-01-01".into(),
        };

        let result = round_trip(&k);

        assert_eq!(result.key_id, "ABCD1234");
    }

    #[test]
    fn test_license_summary_round_trip() {
        let ls = LicenseSummary {
            key: "mit".into(),
            name: "MIT License".into(),
            spdx_id: "MIT".into(),
            url: "https://api.github.com/licenses/mit".into(),
        };

        let result = round_trip(&ls);

        assert_eq!(result.key, "mit");
    }

    #[test]
    fn test_license_detail_round_trip() {
        let ld = LicenseDetail {
            key: "mit".into(),
            name: "MIT License".into(),
            spdx_id: "MIT".into(),
            url: "https://api.github.com/licenses/mit".into(),
            description: "A permissive license".into(),
            implementation: "Create a LICENSE file".into(),
            permissions: vec!["commercial-use".into(), "modification".into()],
            conditions: vec!["include-copyright".into()],
            limitations: vec!["liability".into()],
            body: "MIT License text...".into(),
        };

        let result = round_trip(&ld);

        assert_eq!(result.permissions.len(), 2);

        assert_eq!(result.conditions.len(), 1);
    }

    #[test]
    fn test_code_search_result_round_trip() {
        let csr = CodeSearchResult {
            file: "main.rs".into(),
            repo: "o/r".into(),
            url: "https://github.com/o/r/blob/main/main.rs".into(),
        };

        let result = round_trip(&csr);

        assert_eq!(result.file, "main.rs");
    }

    #[test]
    fn test_wiki_page_round_trip() {
        let wp = WikiPage {
            path: "Home".into(),
            title: "Home".into(),
            format: "md".into(),
            filename: "Home.md".into(),
        };

        let result = round_trip(&wp);

        assert_eq!(result.path, "Home");
    }

    #[test]
    fn test_wiki_page_content_round_trip() {
        let wpc = WikiPageContent {
            page: WikiPage {
                path: "Home".into(),
                title: "Home".into(),
                format: "md".into(),
                filename: "Home.md".into(),
            },
            content: "Welcome".into(),
        };

        let json = serde_json::to_string(&wpc).unwrap();

        let result: WikiPageContent = serde_json::from_str(&json).unwrap();

        assert_eq!(result.content, "Welcome");

        assert_eq!(result.page.path, "Home");
    }

    #[test]
    fn test_issue_template_round_trip() {
        let it = IssueTemplate {
            name: "Bug Report".into(),
            filename: "bug_report.md".into(),
            path: ".github/ISSUE_TEMPLATE/bug_report.md".into(),
            body: Some("### Describe the bug".into()),
            about: Some("File a bug".into()),
            title: Some("[Bug]".into()),
            labels: Some(vec!["bug".into()]),
            assignees: None,
        };

        let result = round_trip(&it);

        assert_eq!(result.name, "Bug Report");

        assert!(result.assignees.is_none());
    }

    #[test]
    fn test_milestone_round_trip() {
        let ms = Milestone {
            id: 1,
            url: "https://api.github.com/repos/o/r/milestones/1".into(),
            title: "v1.0".into(),
            number: 1,
            html_url: "https://github.com/o/r/milestone/1".into(),
            open_issues: 5,
            state: MilestoneState::Open,
            due_on: Some("2025-12-31".into()),
            closed_issues: 2,
        };

        let result = round_trip(&ms);

        assert_eq!(result.title, "v1.0");

        assert!(matches!(result.state, MilestoneState::Open));
    }

    #[test]
    fn test_milestone_progress_round_trip() {
        let mp = MilestoneProgress {
            title: "v1.0".into(),
            total: 10,
            percent: 20,
            open_issues: 8,
            closed_issues: 2,
        };

        let result = round_trip(&mp);

        assert_eq!(result.percent, 20);
    }

    #[test]
    fn test_audit_event_round_trip() {
        let ae = AuditEvent {
            id: "evt-1".into(),
            action: "repo.create".into(),
            repo: Some("org/repo".into()),
            actor: Some("octocat".into()),
            created_at: Some("2025-01-01".into()),
            raw: serde_json::Value::Null,
        };

        let result = round_trip(&ae);

        assert_eq!(result.action, "repo.create");

        assert!(result.raw.is_null());
    }

    #[test]
    fn test_branch_protection_round_trip() {
        let bp = BranchProtection {
            pattern: "main".into(),
            required_checks: vec!["CI".into()],
            required_reviews: 1,
            dismiss_stale: true,
            enforce_admins: false,
            allow_force_pushes: false,
        };

        let result = round_trip(&bp);

        assert_eq!(result.pattern, "main");

        assert!(result.dismiss_stale);
    }

    #[test]
    fn test_bulk_repo_result_round_trip() {
        let brr: BulkRepoResult = BulkRepoResult {
            repo: "org/repo".into(),
            metadata: None,
            error: Some("not found".into()),
            success: false,
        };

        let result = round_trip(&brr);

        assert!(!result.success);

        assert!(result.metadata.is_none());
    }

    #[test]
    fn test_bulk_repo_metadata_round_trip() {
        let brm: BulkRepoMetadata = BulkRepoMetadata {
            failed: 1,
            completed: 2,
            results: vec![],
        };

        let result = round_trip(&brm);

        assert_eq!(result.failed, 1);

        assert_eq!(result.completed, 2);
        assert!(result.results.is_empty());
    }

    #[test]
    fn test_repo_target_options_round_trip() {
        let rto = RepoTargetOptions {
            org: Some("myorg".into()),
            user: None,
            file: None,
            repos: None,
            limit: Some(10),
        };

        let result = round_trip(&rto);

        assert_eq!(result.org.as_deref(), Some("myorg"));

        assert!(result.user.is_none());
    }

    #[test]
    fn test_profile_round_trip() {
        let p = Profile {
            token: Some("ghp_abc123".into()),
            host: Some("github.com".into()),
            provider: Some("github".into()),
            extra: Default::default(),
        };

        let result = round_trip(&p);

        assert_eq!(result.token.as_deref(), Some("ghp_abc123"));
    }

    #[test]
    fn test_profile_serialization_skips_none() {
        let p = Profile {
            token: None,
            host: None,
            provider: None,
            extra: Default::default(),
        };

        let json = serde_json::to_string(&p).unwrap();

        assert!(!json.contains("token"));

        assert!(!json.contains("host"));
        assert!(!json.contains("provider"));
    }

    #[test]
    fn test_credentials_file_round_trip() {
        let mut profiles = std::collections::HashMap::new();
        profiles.insert(
            "default".into(),
            Profile {
                token: Some("ghp_abc".into()),
                host: None,
                provider: None,
                extra: Default::default(),
            },
        );

        let cf = CredentialsFile {
            active_profile: "default".into(),
            profiles,
            aliases: std::collections::HashMap::new(),
        };

        let result = round_trip(&cf);

        assert_eq!(result.active_profile, "default");

        assert!(result.profiles.contains_key("default"));
    }

    #[test]
    fn test_project_board_round_trip() {
        let pb = ProjectBoard {
            owner: "org".into(),
            title: "Board 1".into(),
            number: 1,
            columns: vec![],
        };

        let result = round_trip(&pb);

        assert_eq!(result.owner, "org");

        assert!(result.columns.is_empty());
    }

    #[test]
    fn test_project_summary_round_trip() {
        let ps = ProjectSummary {
            id: "PVT_1".into(),
            number: 1,
            title: "My Project".into(),
            description: "desc".into(),
            closed: false,
            url: "https://github.com/org/projects/1".into(),
            updated_at: None,
        };

        let result = round_trip(&ps);

        assert!(!result.closed);
    }

    #[test]
    fn test_project_item_round_trip() {
        let pi = ProjectItem {
            id: "PVTI_1".into(),
            r#type: "ISSUE".into(),
            title: "Item 1".into(),
            status: "Todo".into(),
            state: Some("open".into()),
            number: Some(1),
            url: None,
            repository: None,
        };

        let result = round_trip(&pi);

        assert_eq!(result.r#type, "ISSUE");
    }

    #[test]
    fn test_project_field_round_trip() {
        let pf = ProjectField {
            id: "PVT_1".into(),
            name: "Status".into(),
            data_type: "SINGLE_SELECT".into(),
            options: Some(vec![ProjectFieldOption {
                id: "opt1".into(),
                name: "Todo".into(),
            }]),
        };

        let result = round_trip(&pf);

        assert!(result.options.is_some());
    }

    #[test]
    fn test_package_summary_round_trip() {
        let ps = PackageSummary {
            id: 1,
            name: "my-pkg".into(),
            package_type: "npm".into(),
            visibility: "public".into(),
            url: "https://api.github.com/packages/1".into(),
            html_url: "https://github.com/o/r/pkgs/1".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-02".into(),
            owner: "o".into(),
            repository: "r".into(),
        };

        let result = round_trip(&ps);

        assert_eq!(result.package_type, "npm");
    }

    #[test]
    fn test_package_version_round_trip() {
        let pv = PackageVersion {
            id: 1,
            name: "my-pkg".into(),
            version: "1.0.0".into(),
            url: "https://api.github.com/packages/1/versions/1".into(),
            html_url: "https://github.com/o/r/pkgs/1/1.0.0".into(),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-01".into(),
        };

        let result = round_trip(&pv);

        assert_eq!(result.version, "1.0.0");
    }

    #[test]
    fn test_codespace_summary_round_trip() {
        let cs = CodespaceSummary {
            id: 1,
            name: "my-codespace".into(),
            state: "Available".into(),
            owner: "octocat".into(),
            repo: "o/r".into(),
            branch: "main".into(),
            created_at: "2025-01-01".into(),
            idle_timeout_minutes: 30,
            machine: "standardLinux".into(),
        };

        let result = round_trip(&cs);

        assert_eq!(result.idle_timeout_minutes, 30);
    }

    #[test]
    fn test_attestation_summary_round_trip() {
        let as_ = AttestationSummary {
            bundle_type: "sigstore".into(),
            predicate_type: "https://example.com/pred".into(),
            digest: "sha256:abc123".into(),
            repository_id: 42,
            created_at: "2025-01-01".into(),
        };

        let result = round_trip(&as_);

        assert_eq!(result.repository_id, 42);
    }

    #[test]
    fn test_alias_entry_round_trip() {
        let ae = AliasEntry {
            name: "co".into(),
            expansion: "checkout".into(),
        };

        let result = round_trip(&ae);

        assert_eq!(result.name, "co");

        assert_eq!(result.expansion, "checkout");
    }

    #[test]
    fn test_comment_summary_round_trip() {
        let cs = CommentSummary {
            id: 1,
            body: "Nice PR".into(),
            author: Some("octocat".into()),
            created_at: "2025-01-01".into(),
            updated_at: "2025-01-02".into(),
        };

        let result = round_trip(&cs);

        assert_eq!(result.body, "Nice PR");
    }

    #[test]
    fn test_reaction_summary_round_trip() {
        let rs = ReactionSummary {
            id: 1,
            content: "+1".into(),
            user: Some("octocat".into()),
            created_at: "2025-01-01".into(),
        };

        let result = round_trip(&rs);

        assert_eq!(result.content, "+1");
    }

    #[test]
    fn test_dependency_entry_round_trip() {
        let de = DependencyEntry {
            name: "serde".into(),
            version: "1.0".into(),
            ecosystem: "crates.io".into(),
        };

        let result = round_trip(&de);

        assert_eq!(result.name, "serde");
    }

    #[test]
    fn test_dependency_review_change_round_trip() {
        let drc = DependencyReviewChange {
            change_type: "added".into(),
            package: "serde".into(),
            ecosystem: "crates.io".into(),
            version: "1.0".into(),
            severity: "low".into(),
            vulnerabilities: 0,
        };

        let result = round_trip(&drc);

        assert_eq!(result.change_type, "added");
    }

    #[test]
    fn test_advisory_summary_round_trip() {
        let as_ = AdvisorySummary {
            ghsa_id: "GHSA-abc-123".into(),
            summary: "Vulnerability".into(),
            severity: "high".into(),
            ecosystem: "crates.io".into(),
            cve_id: Some("CVE-2025-0001".into()),
            published_at: "2025-01-01".into(),
            html_url: "https://github.com/advisories/GHSA-abc-123".into(),
        };

        let result = round_trip(&as_);

        assert_eq!(result.ghsa_id, "GHSA-abc-123");
    }

    #[test]
    fn test_advisory_create_input_round_trip() {
        let aci = AdvisoryCreateInput {
            severity: "high".into(),
            cve_id: None,
            summary: "desc".into(),
            description: "details".into(),
            vulnerable_version_range: Some(">=1.0".into()),
            patched_version_range: None,
        };

        let result = round_trip(&aci);

        assert_eq!(result.severity, "high");

        assert!(result.cve_id.is_none());
    }

    #[test]
    fn test_codeql_alert_summary_round_trip() {
        let cas = CodeQLAlertSummary {
            number: 1,
            rule: "js/sql-injection".into(),
            severity: "error".into(),
            state: "open".into(),
            tool: "CodeQL".into(),
            created_at: "2025-01-01".into(),
        };

        let result = round_trip(&cas);

        assert_eq!(result.rule, "js/sql-injection");
    }

    #[test]
    fn test_search_options_round_trip() {
        let so = SearchOptions {
            repo: Some("o/r".into()),
            sort: None,
            limit: Some(10),
            state: None,
            author: Some("octocat".into()),
            language: None,
            order: None,
        };

        let result = round_trip(&so);

        assert_eq!(result.repo.as_deref(), Some("o/r"));
    }

    #[test]
    fn test_secret_scan_finding_round_trip() {
        let ssf = SecretScanFinding {
            file: "config.yml".into(),
            rule: "AWS Secret Key".into(),
            r#match: "AKIA...".into(),
            line: Some(10),
            confidence: SecretScanConfidence::High,
        };

        let result = round_trip(&ssf);

        assert_eq!(result.file, "config.yml");

        assert!(matches!(result.confidence, SecretScanConfidence::High));
    }

    #[test]
    fn test_secret_scan_confidence_round_trip() {
        for conf in [
            SecretScanConfidence::High,
            SecretScanConfidence::Medium,
            SecretScanConfidence::Low,
        ] {
            let result = round_trip(&conf);

            assert_eq!(
                std::mem::discriminant(&result),
                std::mem::discriminant(&conf)
            );
        }
    }

    #[test]
    fn test_secret_scanning_alert_round_trip() {
        let ssa = SecretScanningAlert {
            url: "https://api.github.com/repos/o/r/secret-scanning/alerts/1".into(),
            state: "open".into(),
            number: 1,
            created_at: "2025-01-01".into(),
            secret_type: "aws_access_key_id".into(),
            repository: "o/r".into(),
            resolution: None,
            resolved_at: None,
            secret_type_display_name: "AWS Access Key ID".into(),
        };

        let result = round_trip(&ssa);

        assert!(result.resolution.is_none());
    }

    #[test]
    fn test_dependabot_alert_round_trip() {
        let da = DependabotAlert {
            state: "open".into(),
            number: 1,
            severity: "high".into(),
            advisory: "GHSA-abc".into(),
            ecosystem: "npm".into(),
            repository: "o/r".into(),
            package_name: "lodash".into(),
            manifest_path: "package.json".into(),
            dismissed_reason: None,
        };

        assert!(da.dismissed_reason.is_none());
    }

    #[test]
    fn test_compliance_check_round_trip() {
        let cc = ComplianceCheck {
            id: "check-1".into(),
            label: "Branch Protection".into(),
            message: "Missing".into(),
            status: ComplianceCheckStatus::Fail,
        };

        let result = round_trip(&cc);

        assert!(matches!(result.status, ComplianceCheckStatus::Fail));
    }

    #[test]
    fn test_compliance_result_round_trip() {
        let cr = ComplianceResult {
            repo: "o/r".into(),
            score: 75,
            remediation: vec!["Enable branch protection".into()],
            checks: vec![ComplianceCheck {
                id: "c1".into(),
                label: "Check".into(),
                message: "Pass".into(),
                status: ComplianceCheckStatus::Pass,
            }],
        };

        let result = round_trip(&cr);

        assert_eq!(result.score, 75);
    }

    #[test]
    fn test_compliance_check_status_round_trip() {
        for status in [
            ComplianceCheckStatus::Pass,
            ComplianceCheckStatus::Fail,
            ComplianceCheckStatus::Unknown,
        ] {
            let result = round_trip(&status);

            assert_eq!(
                std::mem::discriminant(&result),
                std::mem::discriminant(&status)
            );
        }
    }

    #[test]
    fn test_empty_collections_round_trip() {
        let sr: SearchResult<RepoSearchItem> = SearchResult {
            items: vec![],
            total_count: 0,
            incomplete_results: false,
        };

        let json = serde_json::to_string(&sr).unwrap();

        let result: SearchResult<RepoSearchItem> = serde_json::from_str(&json).unwrap();

        assert!(result.items.is_empty());
    }

    #[test]
    fn test_bulk_repo_result_typed_metadata() {
        let brr: BulkRepoResult<String> = BulkRepoResult {
            repo: "o/r".into(),
            metadata: Some("hello".into()),
            error: None,
            success: true,
        };

        let result = round_trip(&brr);

        assert_eq!(result.metadata.as_deref(), Some("hello"));
    }
}
