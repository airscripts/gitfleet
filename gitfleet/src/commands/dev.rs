use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum DevCommand {
    #[command(about = "List codespaces.")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Create a codespace.")]
    Create {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        branch: Option<String>,
    },

    #[command(about = "Delete a codespace.")]
    Delete {
        id: String,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: DevCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    let ops = p.dev_env_ops().ok_or_else(|| {
        GitfleetError::UnsupportedCapability(UnsupportedCapabilityError::new(
            app.provider_id(),
            ProviderCapability::DevelopmentEnvironments,
        ))
    })?;

    match cmd {
        DevCommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let data = ops.list_codespaces(&repo_str).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&data).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize codespaces: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = data
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "ID": c.id,
                            "NAME": c.name,
                            "STATE": c.state,
                            "REPO": c.repo,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No codespaces found."),
                    Some("Codespaces"),
                    Some(&["ID", "NAME", "STATE", "REPO"]),
                );
            }

            Ok(())
        }

        DevCommand::Create { repo, branch } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let result = ops.create_codespace(&repo_str, branch.as_deref()).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize codespace: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("Codespace created", &result.name);
            }

            Ok(())
        }

        DevCommand::Delete { id, yes } => {
            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("codespace {id}"),
                    }));
                } else {
                    app.renderer()
                        .render_box(&format!("Would delete codespace {id}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete codespace {id}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            let repo_str = {
                let remote = gitfleet_core::git::get_remote_url(None)?;

                let parsed = gitfleet_core::repository::repository_ref_from_remote(&remote)?;

                parsed.full_name()
            };

            ops.delete_codespace(&repo_str, &id).await?;

            app.renderer().render_success_box("Codespace deleted", &id);

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_dev_list() {
        let app = test_helpers::make_app();

        run(
            DevCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dev_list_json() {
        let app = test_helpers::make_app_json();

        run(
            DevCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dev_create() {
        let app = test_helpers::make_app();

        run(
            DevCommand::Create {
                repo: Some("org/repo".into()),
                branch: Some("main".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dev_create_json() {
        let app = test_helpers::make_app_json();

        run(
            DevCommand::Create {
                repo: Some("org/repo".into()),
                branch: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dev_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            DevCommand::Delete {
                id: "codespace-1".into(),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dev_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            DevCommand::Delete {
                id: "codespace-1".into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dev_delete_non_interactive_no_yes() {
        let app = test_helpers::make_app_json();

        let result = run(
            DevCommand::Delete {
                id: "codespace-1".into(),
                yes: false,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dev_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            DevCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
