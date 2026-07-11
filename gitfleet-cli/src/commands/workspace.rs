use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, PartialFailureError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum WorkspaceCommand {
    #[command(about = "Define or update a named workspace.")]
    Define {
        #[arg(long)]
        name: String,
        #[arg(long)]
        repos: Vec<String>,
    },

    #[command(about = "List all workspaces.")]
    List,

    #[command(about = "Remove a workspace.")]
    Remove { name: String },

    #[command(about = "Archive all compatible repositories in a workspace.")]
    Archive { name: String },
}

pub async fn run(cmd: WorkspaceCommand, app: &App) -> Result<(), GitfleetError> {
    match cmd {
        WorkspaceCommand::Define { name, repos } => {
            let ws = gitfleet_core::workspace::define(&name, &repos)?;

            if app.renderer().is_json() {
                app.renderer()
                    .write_result(&serde_json::to_value(&ws).unwrap_or_default());
            } else {
                app.renderer().render_success_box(
                    "Workspace defined",
                    &format!("'{}' with {} repositories", ws.name, ws.repositories.len()),
                );
            }

            Ok(())
        }

        WorkspaceCommand::List => {
            let workspaces = gitfleet_core::workspace::list()?;

            if app.renderer().is_json() {
                app.renderer()
                    .write_result(&serde_json::to_value(&workspaces).unwrap_or_default());
            } else {
                if workspaces.is_empty() {
                    app.renderer().write_value("No workspaces defined.");
                } else {
                    let rows: Vec<serde_json::Value> = workspaces
                        .iter()
                        .map(|ws| {
                            serde_json::json!({
                                "NAME": ws.name,
                                "REPOS": ws.repositories.len(),
                            })
                        })
                        .collect();

                    app.renderer().render_table_titled(
                        &rows,
                        Some("No workspaces defined."),
                        Some("Workspaces"),
                        None,
                    );
                }
            }

            Ok(())
        }

        WorkspaceCommand::Remove { name } => {
            gitfleet_core::workspace::remove(&name)?;

            app.renderer()
                .write_value(&format!("Workspace '{name}' removed."));

            Ok(())
        }

        WorkspaceCommand::Archive { name } => archive(&name, app).await,
    }
}

async fn archive(name: &str, app: &App) -> Result<(), GitfleetError> {
    let workspace = gitfleet_core::workspace::get(name)?;
    let provider = app.provider()?;

    let repo_ops = provider.repo_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            app.provider_id(),
            ProviderCapability::Repositories,
        ))
    })?;

    let mut results = Vec::with_capacity(workspace.repositories.len());
    let mut has_partial_failure = false;

    for repository in workspace.repositories {
        let target = format!("{}/{}", repository.namespace, repository.name);

        if repository.provider != app.provider_id() || repository.host != app.provider_host() {
            has_partial_failure = true;

            results.push(serde_json::json!({
                "repository": target,
                "provider": repository.provider.to_string(),
                "host": repository.host,
                "status": "skipped",
                "reason": "Repository does not match the active provider profile.",
            }));

            continue;
        }

        results.push(serde_json::json!({
            "repository": target,
            "provider": repository.provider.to_string(),
            "host": repository.host,
            "status": if app.dry_run() { "would_archive" } else { "pending" },
        }));
    }

    let has_pending_targets = results.iter().any(|result| result["status"] == "pending");

    if !app.dry_run() && has_pending_targets {
        gitfleet_core::prompt::confirm_destructive(
            &format!("Archive compatible repositories in workspace '{name}'?"),
            app.renderer().mode(),
            app.renderer().yes(),
        )?;
    }

    if !app.dry_run() && has_pending_targets {
        for result in &mut results {
            if result["status"] != "pending" {
                continue;
            }

            let target = result["repository"].as_str().unwrap_or_default();

            match repo_ops.archive_repo(target).await {
                Ok(()) => result["status"] = serde_json::json!("archived"),
                Err(error) => {
                    has_partial_failure = true;
                    result["status"] = serde_json::json!("failed");
                    result["reason"] = serde_json::json!(error.to_string());
                }
            }
        }
    }

    let archived = results
        .iter()
        .filter(|result| result["status"] == "archived")
        .count();

    let would_archive = results
        .iter()
        .filter(|result| result["status"] == "would_archive")
        .count();

    let skipped = results
        .iter()
        .filter(|result| result["status"] == "skipped")
        .count();

    let failed = results
        .iter()
        .filter(|result| result["status"] == "failed")
        .count();

    let report = serde_json::json!({
        "operation": "archive",
        "workspace": name,
        "provider": app.provider_id().to_string(),
        "host": app.provider_host(),
        "dry_run": app.dry_run(),
        "results": results,
        "summary": {
            "total": archived + would_archive + skipped + failed,
            "archived": archived,
            "would_archive": would_archive,
            "skipped": skipped,
            "failed": failed,
        },
    });

    if app.renderer().is_json() {
        app.renderer().write_result(&report);
    } else {
        let rows = report["results"].as_array().cloned().unwrap_or_default();

        app.renderer().render_table_titled(
            &rows,
            Some("Workspace has no repositories."),
            Some(&format!("Workspace '{name}' archive")),
            None,
        );
    }

    if has_partial_failure {
        return Err(GitfleetError::from(PartialFailureError::new(
            "Workspace archive completed with skipped or failed repositories.",
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    fn setup_test_env() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();

        let gitfleet_dir = dir.path().join(".config").join("gitfleet");
        std::fs::create_dir_all(&gitfleet_dir).unwrap();

        std::env::set_var("HOME", dir.path().to_string_lossy().to_string());
        dir
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_define() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        run(
            WorkspaceCommand::Define {
                name: "test-ws".into(),
                repos: vec!["org/repo1".into(), "org/repo2".into()],
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_define_json() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_json();

        run(
            WorkspaceCommand::Define {
                name: "json-ws".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_define_empty_repos() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        run(
            WorkspaceCommand::Define {
                name: "empty-ws".into(),
                repos: vec![],
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_define_invalid_repo() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        let result = run(
            WorkspaceCommand::Define {
                name: "bad-ws".into(),
                repos: vec!["invalid".into()],
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_list_empty() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        run(WorkspaceCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_list_json_empty() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_json();

        run(WorkspaceCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_archive_dry_run() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_dry_run();

        run(
            WorkspaceCommand::Define {
                name: "archive-ws".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        run(
            WorkspaceCommand::Archive {
                name: "archive-ws".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_archive_skips_mismatched_provider() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_dry_run();

        run(
            WorkspaceCommand::Define {
                name: "mixed-ws".into(),
                repos: vec!["gitlab@gitlab.com:org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        let result = run(
            WorkspaceCommand::Archive {
                name: "mixed-ws".into(),
            },
            &app,
        )
        .await;

        assert!(matches!(result, Err(GitfleetError::PartialFailure(_))));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_archive_json_requires_yes() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_json();

        run(
            WorkspaceCommand::Define {
                name: "confirm-ws".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        let result = run(
            WorkspaceCommand::Archive {
                name: "confirm-ws".into(),
            },
            &app,
        )
        .await;

        assert!(result.unwrap_err().to_string().contains("--yes"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_list_with_workspaces() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        run(
            WorkspaceCommand::Define {
                name: "ws-a".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        run(WorkspaceCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_list_json_with_workspaces() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_json();

        run(
            WorkspaceCommand::Define {
                name: "ws-b".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        run(WorkspaceCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_remove() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        run(
            WorkspaceCommand::Define {
                name: "rm-ws".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        run(
            WorkspaceCommand::Remove {
                name: "rm-ws".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_remove_nonexistent() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app();

        let result = run(
            WorkspaceCommand::Remove {
                name: "no-such-ws".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_workspace_remove_json() {
        let _dir = setup_test_env();

        let app = test_helpers::make_app_json();

        run(
            WorkspaceCommand::Define {
                name: "rm-json-ws".into(),
                repos: vec!["org/repo1".into()],
            },
            &app,
        )
        .await
        .unwrap();

        run(
            WorkspaceCommand::Remove {
                name: "rm-json-ws".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }
}
