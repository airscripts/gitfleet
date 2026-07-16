use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum LabelCmdCommand {
    #[command(about = "List labels.")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Create a label.")]
    Create {
        name: String,
        #[arg(long, default_value = "ededed")]
        color: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Delete a label.")]
    Delete {
        name: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: LabelCmdCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        LabelCmdCommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::labels::list(p, app.renderer(), &repo_str).await
        }

        LabelCmdCommand::Create {
            name,
            color,
            description,
            repo,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let label = gitfleet_core::types::Label {
                name: name.clone(),
                color,
                new_name: None,
                description: description.unwrap_or_default(),
            };

            let ops = p.label_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Labels,
                ))
            })?;

            let data = ops.create_label(&label, &repo_str).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer().render_success_box("Label created", &name);
            }

            Ok(())
        }

        LabelCmdCommand::Delete { name, repo, yes } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("{repo_str} label {name}"),
                    }));
                } else {
                    app.renderer().render_box(
                        &format!("Would delete label '{name}' from {repo_str}"),
                        "warning",
                    );
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete label '{name}'?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            let ops = p.label_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Labels,
                ))
            })?;

            ops.delete_label(&name, &repo_str).await?;

            app.renderer().render_success_box("Label deleted", &name);

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_label_list() {
        let app = test_helpers::make_app();

        run(
            LabelCmdCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_label_list_json() {
        let app = test_helpers::make_app_json();

        run(
            LabelCmdCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_label_create() {
        let app = test_helpers::make_app();

        run(
            LabelCmdCommand::Create {
                name: "bug".into(),
                color: "ff0000".into(),
                description: None,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_label_create_with_description() {
        let app = test_helpers::make_app();

        run(
            LabelCmdCommand::Create {
                name: "enhancement".into(),
                color: "ededed".into(),
                description: Some("New feature".into()),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_label_create_json() {
        let app = test_helpers::make_app_json();

        run(
            LabelCmdCommand::Create {
                name: "bug".into(),
                color: "ff0000".into(),
                description: None,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_label_create_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            LabelCmdCommand::Create {
                name: "bug".into(),
                color: "ff0000".into(),
                description: None,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_label_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            LabelCmdCommand::Delete {
                name: "bug".into(),
                repo: Some("org/repo".into()),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_label_delete_dry_run_json() {
        let app = test_helpers::make_app_dry_run_json();

        run(
            LabelCmdCommand::Delete {
                name: "bug".into(),
                repo: Some("org/repo".into()),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_label_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            LabelCmdCommand::Delete {
                name: "bug".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_label_delete_silent_mode() {
        let app = test_helpers::make_app();

        run(
            LabelCmdCommand::Delete {
                name: "bug".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_label_delete_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            LabelCmdCommand::Delete {
                name: "bug".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
