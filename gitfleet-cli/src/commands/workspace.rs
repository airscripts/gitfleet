use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

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
    }
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
