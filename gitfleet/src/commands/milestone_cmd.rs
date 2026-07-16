use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnprocessableError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum MilestoneCmdCommand {
    #[command(about = "List milestones.")]
    List {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, default_value = "open")]
        state: String,
    },

    #[command(about = "Create a milestone.")]
    Create {
        title: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },

    #[command(about = "View a milestone.")]
    View {
        number: u64,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Update a milestone.")]
    Update {
        number: u64,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        due_date: Option<String>,
        #[arg(long, value_parser = ["open", "closed"])]
        state: Option<String>,
    },

    #[command(about = "Delete a milestone.")]
    Delete {
        number: u64,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: MilestoneCmdCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    let ops = p.planning_ops().ok_or_else(|| {
        GitfleetError::UnsupportedCapability(UnsupportedCapabilityError::new(
            app.provider_id(),
            ProviderCapability::Milestones,
        ))
    })?;

    match cmd {
        MilestoneCmdCommand::List { repo, state } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let limit = 30u32;
            let data = ops
                .list_milestones(&repo_str, Some(state.as_str()), limit)
                .await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&data).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize milestones: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = data
                    .iter()
                    .map(|m| {
                        serde_json::json!({
                            "NUMBER": m.number,
                            "TITLE": m.title,
                            "STATE": format!("{:?}", m.state),
                            "OPEN": m.open_issues,
                            "CLOSED": m.closed_issues,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No milestones found."),
                    Some("Milestones"),
                    Some(&["NUMBER", "TITLE", "STATE", "OPEN", "CLOSED"]),
                );
            }

            Ok(())
        }

        MilestoneCmdCommand::Create {
            title,
            repo,
            description,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let result = ops
                .create_milestone(&repo_str, &title, description.as_deref())
                .await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize milestone: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("Milestone created", &title);
            }

            Ok(())
        }

        MilestoneCmdCommand::View { number, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let result = ops.get_milestone(&repo_str, number).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize milestone: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("Milestone", &result.title);
            }

            Ok(())
        }

        MilestoneCmdCommand::Update {
            number,
            repo,
            title,
            description,
            due_date,
            state,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if title.is_none() && description.is_none() && due_date.is_none() && state.is_none() {
                return Err(GitfleetError::from(UnprocessableError::new(
                    "At least one milestone update option is required.",
                )));
            }

            let mut input = serde_json::json!({});

            if let Some(title) = title {
                input["title"] = serde_json::Value::String(title);
            }

            if let Some(description) = description {
                input["description"] = serde_json::Value::String(description);
            }

            if let Some(due_date) = due_date {
                input["due_on"] = serde_json::Value::String(due_date);
            }

            if let Some(state) = state {
                input["state"] = serde_json::Value::String(state);
            }

            let result = ops.update_milestone(&repo_str, number, input).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize milestone: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("Milestone updated", &result.title);
            }

            Ok(())
        }

        MilestoneCmdCommand::Delete { number, repo, yes } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("milestone {number}"),
                    }));
                } else {
                    app.renderer()
                        .render_box(&format!("Would delete milestone {number}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete milestone {number}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            ops.delete_milestone(&repo_str, number).await?;

            app.renderer()
                .render_success_box("Milestone deleted", &number.to_string());

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use gitfleet_core::output::Renderer;
    use gitfleet_core::output_state::OutputMode;
    use gitfleet_core::provider::ProviderId;
    use gitfleet_providers::ProviderRegistry;

    use super::super::test_helpers;
    use super::*;
    use crate::app::App;

    fn make_app_dry_run_json() -> App {
        let registry = ProviderRegistry::with_provider(
            ProviderId::GitHub,
            Box::new(test_helpers::MockProvider),
        );

        let renderer = Renderer::new(OutputMode::Json);
        App::new(registry, renderer, ProviderId::GitHub, true)
    }

    #[tokio::test]
    async fn test_milestone_list() {
        let app = test_helpers::make_app();

        run(
            MilestoneCmdCommand::List {
                repo: Some("org/repo".into()),
                state: "open".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_milestone_list_json() {
        let app = test_helpers::make_app_json();

        run(
            MilestoneCmdCommand::List {
                repo: Some("org/repo".into()),
                state: "open".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_milestone_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            MilestoneCmdCommand::List {
                repo: Some("org/repo".into()),
                state: "open".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_milestone_create() {
        let app = test_helpers::make_app();

        run(
            MilestoneCmdCommand::Create {
                title: "v1.0".into(),
                repo: Some("org/repo".into()),
                description: Some("First release".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_milestone_create_json() {
        let app = test_helpers::make_app_json();

        run(
            MilestoneCmdCommand::Create {
                title: "v1.0".into(),
                repo: Some("org/repo".into()),
                description: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_milestone_view() {
        let app = test_helpers::make_app();

        run(
            MilestoneCmdCommand::View {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_milestone_view_json() {
        let app = test_helpers::make_app_json();

        run(
            MilestoneCmdCommand::View {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_milestone_update() {
        let app = test_helpers::make_app();

        run(
            MilestoneCmdCommand::Update {
                number: 1,
                repo: Some("org/repo".into()),
                title: Some("Updated milestone".into()),
                description: None,
                due_date: None,
                state: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_milestone_update_json() {
        let app = test_helpers::make_app_json();

        run(
            MilestoneCmdCommand::Update {
                number: 1,
                repo: Some("org/repo".into()),
                title: None,
                description: Some("Updated description".into()),
                due_date: Some("2026-12-31".into()),
                state: Some("closed".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_milestone_update_requires_a_change() {
        let app = test_helpers::make_app();

        let result = run(
            MilestoneCmdCommand::Update {
                number: 1,
                repo: Some("org/repo".into()),
                title: None,
                description: None,
                due_date: None,
                state: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_milestone_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            MilestoneCmdCommand::Delete {
                number: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_milestone_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            MilestoneCmdCommand::Delete {
                number: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_milestone_delete_dry_run_json() {
        let app = make_app_dry_run_json();

        run(
            MilestoneCmdCommand::Delete {
                number: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_milestone_delete_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            MilestoneCmdCommand::Delete {
                number: 1,
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
