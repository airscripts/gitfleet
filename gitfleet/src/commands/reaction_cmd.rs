use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum ReactionCmdCommand {
    #[command(about = "List reactions.")]
    List {
        number: u64,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Create a reaction.")]
    Create {
        number: u64,
        content: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Delete a reaction.")]
    Delete {
        reaction_id: u64,
        number: u64,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: ReactionCmdCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    let ops = p.review_ops().ok_or_else(|| {
        GitfleetError::UnsupportedCapability(UnsupportedCapabilityError::new(
            app.provider_id(),
            ProviderCapability::Reviews,
        ))
    })?;

    match cmd {
        ReactionCmdCommand::List { number, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let data = ops.list_reactions_for_issue(&repo_str, number).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&data).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize reactions: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = data
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "ID": r.id,
                            "CONTENT": r.content,
                            "USER": r.user,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No reactions found."),
                    Some("Reactions"),
                    Some(&["ID", "CONTENT", "USER"]),
                );
            }

            Ok(())
        }

        ReactionCmdCommand::Create {
            number,
            content,
            repo,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let result = ops
                .create_reaction_for_issue(&repo_str, number, &content)
                .await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize reaction: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("Reaction created", &content);
            }

            Ok(())
        }

        ReactionCmdCommand::Delete {
            reaction_id,
            number,
            repo,
            yes,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("reaction {reaction_id}"),
                    }));
                } else {
                    app.renderer()
                        .render_box(&format!("Would delete reaction {reaction_id}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete reaction {reaction_id}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            ops.delete_reaction_for_issue(&repo_str, number, reaction_id)
                .await?;

            app.renderer()
                .render_success_box("Reaction deleted", &reaction_id.to_string());

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_reaction_list() {
        let app = test_helpers::make_app();

        run(
            ReactionCmdCommand::List {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_reaction_list_json() {
        let app = test_helpers::make_app_json();

        run(
            ReactionCmdCommand::List {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_reaction_create() {
        let app = test_helpers::make_app();

        run(
            ReactionCmdCommand::Create {
                number: 1,
                content: "+1".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_reaction_create_json() {
        let app = test_helpers::make_app_json();

        run(
            ReactionCmdCommand::Create {
                number: 1,
                content: "+1".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_reaction_delete() {
        let app = test_helpers::make_app();

        run(
            ReactionCmdCommand::Delete {
                reaction_id: 1,
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
    async fn test_reaction_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            ReactionCmdCommand::Delete {
                reaction_id: 1,
                number: 1,
                repo: Some("org/repo".into()),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_reaction_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ReactionCmdCommand::List {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
