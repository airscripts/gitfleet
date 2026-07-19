use std::path::{Path, PathBuf};

use gitfleet_core::errors::{
    GitfleetError, PartialFailureError, UnprocessableError, UnsupportedCapabilityError,
};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};
use gitfleet_core::types::RepoSummary;

pub struct CreateOptions<'a> {
    pub name: &'a str,
    pub owner: Option<&'a str>,
    pub owner_type: Option<&'a str>,
    pub visibility: &'a str,
    pub description: Option<&'a str>,
    pub initialize: bool,
}

pub struct CloneOptions<'a> {
    pub repository: Option<&'a str>,
    pub all: bool,
    pub org: Option<&'a str>,
    pub user: Option<&'a str>,
    pub directory: Option<&'a str>,
    pub depth: Option<u32>,
    pub ssh: bool,
    pub concurrency: usize,
    pub include_forks: bool,
    pub include_archived: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CloneStatus {
    Cloned,
    WouldClone,
    Skipped,
    Excluded,
    Failed,
}

impl CloneStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Cloned => "cloned",
            Self::WouldClone => "would_clone",
            Self::Skipped => "skipped",
            Self::Excluded => "excluded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
struct CloneTarget {
    repo: RepoSummary,
    url: String,
    directory: PathBuf,
    status: CloneStatus,
    reason: Option<String>,
}

struct CloneReportContext<'a> {
    owner_kind: &'a str,
    owner: &'a str,
    host: &'a str,
    root: &'a Path,
    dry_run: bool,
    ssh: bool,
}

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    org: Option<&str>,
    username: Option<&str>,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    let repos = if let Some(org) = org {
        ops.list_org_repos(org).await?
    } else if let Some(username) = username {
        ops.list_user_named_repos(username).await?
    } else {
        ops.list_user_repos().await?
    };

    if renderer.is_json() {
        let json = serde_json::to_value(&repos)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize repos: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = repos
            .iter()
            .map(|r| {
                serde_json::json!({
                    "NAME": r.full_name,
                    "VISIBILITY": if r.private { "private" } else { "public" },
                    "DEFAULT BRANCH": r.default_branch,
                    "ARCHIVED": r.archived,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No repositories found."),
            Some("Repositories"),
            Some(&["NAME", "VISIBILITY", "DEFAULT BRANCH", "ARCHIVED"]),
        );
    }

    Ok(())
}

pub async fn view(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    let data = ops.get_repo(repo).await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        renderer.render_summary(
            "Repository",
            &[
                (
                    "Name",
                    data.get("full_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "Description",
                    data.get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A")
                        .to_string(),
                ),
                (
                    "Visibility",
                    if data
                        .get("private")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        "private".to_string()
                    } else {
                        "public".to_string()
                    },
                ),
                (
                    "Default Branch",
                    data.get("default_branch")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                (
                    "URL",
                    data.get("html_url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
            ],
        );
    }

    Ok(())
}

pub async fn create(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    options: CreateOptions<'_>,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    let data = ops
        .create_repo(
            options.name,
            options.visibility,
            options.owner,
            options.owner_type,
            options.description,
            options.initialize,
        )
        .await?;

    if renderer.is_json() {
        renderer.write_result(&data);
    } else {
        let full_name = data
            .get("full_name")
            .and_then(|v| v.as_str())
            .unwrap_or(options.name);

        let html_url = data.get("html_url").and_then(|v| v.as_str()).unwrap_or("");
        renderer.render_success_box("Repository created", &format!("{full_name}\n{html_url}"));
    }

    Ok(())
}

pub async fn clone(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    host: &str,
    options: CloneOptions<'_>,
) -> Result<(), GitfleetError> {
    validate_clone_options(&options)?;

    if !options.all {
        let repository = options.repository.unwrap_or_default();
        let url = clone_url(host, repository, options.ssh);
        let depth_display = match options.depth {
            Some(d) => format!("depth: {d}"),
            None => "full depth".to_string(),
        };

        gitfleet_core::git::clone_repository(&url, options.depth, options.directory, None)?;

        renderer.render_success_box("Cloned", &format!("{repository} ({depth_display})"));

        return Ok(());
    }

    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    let (owner_kind, owner, repos) = if let Some(org) = options.org {
        ("org", org, ops.list_org_repos(org).await?)
    } else {
        let user = options.user.unwrap_or_default();

        ("user", user, ops.list_user_named_repos(user).await?)
    };

    let root = options
        .directory
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    if !options.dry_run {
        std::fs::create_dir_all(&root).map_err(|error| {
            GitfleetError::from(UnprocessableError::new(format!(
                "Failed to create clone directory '{}': {error}",
                root.display()
            )))
        })?;
    }

    let mut targets = build_clone_targets(
        repos,
        host,
        &root,
        options.ssh,
        options.include_forks,
        options.include_archived,
        options.dry_run,
    );

    let pending_indices: Vec<usize> = targets
        .iter()
        .enumerate()
        .filter_map(|(index, target)| {
            (target.status == CloneStatus::WouldClone && !options.dry_run).then_some(index)
        })
        .collect();

    if !pending_indices.is_empty() {
        let depth = options.depth;
        let clone_jobs: Vec<(String, PathBuf)> = pending_indices
            .iter()
            .map(|index| {
                let target = &targets[*index];

                (target.url.clone(), target.directory.clone())
            })
            .collect();

        let clone_results = gitfleet_core::bulk::run_bulk(
            &clone_jobs,
            move |(url, directory), _| {
                let directory = directory.to_string_lossy().to_string();

                async move {
                    gitfleet_core::git::clone_repository(&url, depth, Some(&directory), None)
                        .map_err(|error| error.to_string())
                }
            },
            options.concurrency,
        )
        .await;

        for clone_result in clone_results {
            let Some(result_index) = pending_indices.get(clone_result.index).copied() else {
                continue;
            };

            let target = &mut targets[result_index];

            if clone_result.success {
                target.status = CloneStatus::Cloned;
            } else {
                target.status = CloneStatus::Failed;
                target.reason = clone_result.error;
            }
        }
    }

    render_clone_report(
        renderer,
        CloneReportContext {
            owner_kind,
            owner,
            host,
            root: &root,
            dry_run: options.dry_run,
            ssh: options.ssh,
        },
        &targets,
    );

    if targets
        .iter()
        .any(|target| target.status == CloneStatus::Failed)
    {
        return Err(GitfleetError::from(PartialFailureError::new(
            "Repository clone completed with failures.",
        )));
    }

    Ok(())
}

fn validate_clone_options(options: &CloneOptions<'_>) -> Result<(), GitfleetError> {
    if options.all && options.repository.is_some() {
        return Err(GitfleetError::from(UnprocessableError::new(
            "Use either a repository argument or --all, not both.",
        )));
    }

    if !options.all && options.repository.is_none() {
        return Err(GitfleetError::from(UnprocessableError::new(
            "Repository is required unless --all is used.",
        )));
    }

    if !options.all
        && (options.org.is_some()
            || options.user.is_some()
            || options.include_forks
            || options.include_archived
            || options.concurrency != 4)
    {
        return Err(GitfleetError::from(UnprocessableError::new(
            "--org, --user, --include-forks, --include-archived, and --concurrency require --all.",
        )));
    }

    if options.all && options.org.is_none() && options.user.is_none() {
        return Err(GitfleetError::from(UnprocessableError::new(
            "--all requires either --org ORG or --user USER.",
        )));
    }

    if options.org.is_some() && options.user.is_some() {
        return Err(GitfleetError::from(UnprocessableError::new(
            "Use either --org or --user, not both.",
        )));
    }

    Ok(())
}

fn build_clone_targets(
    repos: Vec<RepoSummary>,
    host: &str,
    root: &Path,
    ssh: bool,
    include_forks: bool,
    include_archived: bool,
    dry_run: bool,
) -> Vec<CloneTarget> {
    repos
        .into_iter()
        .map(|repo| {
            let directory = root.join(&repo.name);

            let url = clone_url(host, &repo.full_name, ssh);

            let mut status = if dry_run {
                CloneStatus::WouldClone
            } else {
                CloneStatus::Cloned
            };
            let mut reason = None;

            if repo.fork && !include_forks {
                status = CloneStatus::Excluded;
                reason = Some("Repository is a fork.".to_string());
            } else if repo.archived && !include_archived {
                status = CloneStatus::Excluded;
                reason = Some("Repository is archived.".to_string());
            } else if directory.exists() {
                status = CloneStatus::Skipped;
                reason = Some("Target directory already exists.".to_string());
            } else if !dry_run {
                status = CloneStatus::WouldClone;
            }

            CloneTarget {
                repo,
                url,
                directory,
                status,
                reason,
            }
        })
        .collect()
}

fn clone_url(host: &str, repository: &str, ssh: bool) -> String {
    if ssh {
        format!("git@{host}:{repository}.git")
    } else {
        format!("https://{host}/{repository}")
    }
}

fn clone_report_rows(targets: &[CloneTarget]) -> Vec<serde_json::Value> {
    targets
        .iter()
        .map(|target| {
            serde_json::json!({
                "repository": target.repo.full_name,
                "directory": target.directory.display().to_string(),
                "status": target.status.as_str(),
                "reason": target.reason,
            })
        })
        .collect()
}

fn clone_report_human_rows(targets: &[CloneTarget]) -> Vec<serde_json::Value> {
    targets
        .iter()
        .map(|target| {
            serde_json::json!({
                "REPOSITORY": target.repo.full_name,
                "DIRECTORY": target.directory.display().to_string(),
                "STATUS": target.status.as_str(),
                "REASON": target.reason,
            })
        })
        .collect()
}

fn render_clone_report(
    renderer: &Renderer,
    context: CloneReportContext<'_>,
    targets: &[CloneTarget],
) {
    let counts = clone_counts(targets);
    let rows = clone_report_rows(targets);

    let report = serde_json::json!({
        "operation": "clone",
        "owner_kind": context.owner_kind,
        "owner": context.owner,
        "host": context.host,
        "destination": context.root.display().to_string(),
        "protocol": if context.ssh { "ssh" } else { "https" },
        "dry_run": context.dry_run,
        "summary": counts,
        "results": rows,
    });

    if renderer.is_json() {
        renderer.write_result(&report);
    } else {
        renderer.write_blank_line();
        renderer.render_summary(
            "Repository Clone",
            &[
                ("Owner", format!("{}:{}", context.owner_kind, context.owner)),
                ("Destination", context.root.display().to_string()),
                (
                    "Protocol",
                    if context.ssh { "ssh" } else { "https" }.to_string(),
                ),
                ("Cloned", counts["cloned"].to_string()),
                ("Would clone", counts["would_clone"].to_string()),
                ("Skipped", counts["skipped"].to_string()),
                ("Excluded", counts["excluded"].to_string()),
                ("Failed", counts["failed"].to_string()),
            ],
        );

        let human_rows = clone_report_human_rows(targets);

        renderer.render_table_titled(
            &human_rows,
            Some("No repositories matched."),
            Some("Repositories"),
            Some(&["REPOSITORY", "DIRECTORY", "STATUS", "REASON"]),
        );
    }
}

fn clone_counts(targets: &[CloneTarget]) -> serde_json::Value {
    let cloned = targets
        .iter()
        .filter(|target| target.status == CloneStatus::Cloned)
        .count();

    let would_clone = targets
        .iter()
        .filter(|target| target.status == CloneStatus::WouldClone)
        .count();

    let skipped = targets
        .iter()
        .filter(|target| target.status == CloneStatus::Skipped)
        .count();

    let excluded = targets
        .iter()
        .filter(|target| target.status == CloneStatus::Excluded)
        .count();

    let failed = targets
        .iter()
        .filter(|target| target.status == CloneStatus::Failed)
        .count();

    serde_json::json!({
        "total": targets.len(),
        "cloned": cloned,
        "would_clone": would_clone,
        "skipped": skipped,
        "excluded": excluded,
        "failed": failed,
    })
}

pub async fn delete(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    ops.delete_repo(repo).await?;

    renderer.render_success_box("Repository deleted", repo);

    Ok(())
}

pub async fn star(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    ops.star_repo(repo).await?;

    renderer.render_success_box("Starred", repo);

    Ok(())
}

pub async fn unstar(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Repositories,
        ))
    })?;

    ops.unstar_repo(repo).await?;

    renderer.render_success_box("Unstarred", repo);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(name: &str, fork: bool, archived: bool) -> RepoSummary {
        RepoSummary {
            id: 1,
            name: name.to_string(),
            fork,
            full_name: format!("owner/{name}"),
            private: false,
            archived,
            default_branch: "main".to_string(),
            pushed_at: None,
        }
    }

    fn clone_options<'a>(
        repository: Option<&'a str>,
        all: bool,
        org: Option<&'a str>,
        user: Option<&'a str>,
    ) -> CloneOptions<'a> {
        CloneOptions {
            repository,
            all,
            org,
            user,
            directory: None,
            depth: None,
            ssh: false,
            concurrency: 4,
            include_forks: false,
            include_archived: false,
            dry_run: true,
        }
    }

    #[test]
    fn test_clone_url_https() {
        assert_eq!(
            clone_url("github.com", "owner/repo", false),
            "https://github.com/owner/repo"
        );
    }

    #[test]
    fn test_clone_url_ssh() {
        assert_eq!(
            clone_url("github.com", "owner/repo", true),
            "git@github.com:owner/repo.git"
        );
    }

    #[test]
    fn test_validate_clone_options_all_requires_owner() {
        let options = clone_options(None, true, None, None);

        let result = validate_clone_options(&options);

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_clone_options_rejects_repository_with_all() {
        let options = clone_options(Some("owner/repo"), true, Some("owner"), None);

        let result = validate_clone_options(&options);

        assert!(result.is_err());
    }

    #[test]
    fn test_build_clone_targets_excludes_forks_and_archived_by_default() {
        let root = tempfile::tempdir().unwrap();
        let targets = build_clone_targets(
            vec![
                repo("active", false, false),
                repo("forked", true, false),
                repo("archived", false, true),
            ],
            "github.com",
            root.path(),
            false,
            false,
            false,
            true,
        );

        assert_eq!(targets[0].status, CloneStatus::WouldClone);
        assert_eq!(targets[1].status, CloneStatus::Excluded);
        assert_eq!(targets[2].status, CloneStatus::Excluded);
    }

    #[test]
    fn test_build_clone_targets_include_flags_allow_forks_and_archived() {
        let root = tempfile::tempdir().unwrap();
        let targets = build_clone_targets(
            vec![repo("forked", true, false), repo("archived", false, true)],
            "github.com",
            root.path(),
            false,
            true,
            true,
            true,
        );

        assert_eq!(targets[0].status, CloneStatus::WouldClone);
        assert_eq!(targets[1].status, CloneStatus::WouldClone);
    }

    #[test]
    fn test_build_clone_targets_skips_existing_directory() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("active")).unwrap();

        let targets = build_clone_targets(
            vec![repo("active", false, false)],
            "github.com",
            root.path(),
            false,
            false,
            false,
            true,
        );

        assert_eq!(targets[0].status, CloneStatus::Skipped);
    }

    #[test]
    fn test_clone_counts() {
        let root = tempfile::tempdir().unwrap();
        let targets = build_clone_targets(
            vec![repo("active", false, false), repo("forked", true, false)],
            "github.com",
            root.path(),
            false,
            false,
            false,
            true,
        );

        let counts = clone_counts(&targets);

        assert_eq!(counts["total"], 2);
        assert_eq!(counts["would_clone"], 1);
        assert_eq!(counts["excluded"], 1);
    }

    #[test]
    fn test_clone_report_human_rows_use_uppercase_headers() {
        let root = tempfile::tempdir().unwrap();
        let targets = build_clone_targets(
            vec![repo("active", false, false)],
            "github.com",
            root.path(),
            false,
            false,
            false,
            true,
        );

        let json_rows = clone_report_rows(&targets);
        let human_rows = clone_report_human_rows(&targets);

        assert!(json_rows[0].get("repository").is_some());
        assert!(json_rows[0].get("REPOSITORY").is_none());
        assert!(human_rows[0].get("REPOSITORY").is_some());
        assert!(human_rows[0].get("repository").is_none());
    }
}
