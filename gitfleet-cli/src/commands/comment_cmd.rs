use clap::{Subcommand, ValueEnum};
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CommentTargetArg {
    Issue,

    Change,
}

#[derive(Subcommand, Debug)]
pub enum CommentCmdCommand {
    #[command(about = "List comments on an issue or change request.")]
    List {
        number: u64,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, value_enum, default_value = "change")]
        target: CommentTargetArg,
    },

    #[command(about = "Create a comment on an issue or change request.")]
    Create {
        number: u64,
        body: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, value_enum, default_value = "change")]
        target: CommentTargetArg,
    },
}

pub async fn run(cmd: CommentCmdCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        CommentCmdCommand::List {
            number,
            repo,
            target,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let data = match target {
                CommentTargetArg::Issue => {
                    let ops = p.issue_ops().ok_or_else(|| {
                        GitfleetError::from(UnsupportedCapabilityError::new(
                            p.id(),
                            ProviderCapability::Issues,
                        ))
                    })?;

                    ops.list_issue_comments(&repo_str, number).await?
                }

                CommentTargetArg::Change => {
                    let ops = p.change_ops().ok_or_else(|| {
                        GitfleetError::from(UnsupportedCapabilityError::new(
                            p.id(),
                            ProviderCapability::Changes,
                        ))
                    })?;

                    ops.list_change_comments(&repo_str, number).await?
                }
            };

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let items = data.as_array().cloned().unwrap_or_default();

                let rows: Vec<serde_json::Value> = items
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "ID": c.get("id"),
                            "USER": c.get("user").and_then(|u| u.get("login")),
                            "CREATED": c.get("created_at"),
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No comments found."),
                    Some("Comments"),
                    Some(&["ID", "USER", "CREATED"]),
                );
            }

            Ok(())
        }

        CommentCmdCommand::Create {
            number,
            body,
            repo,
            target,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let result = match target {
                CommentTargetArg::Issue => {
                    let ops = p.issue_ops().ok_or_else(|| {
                        GitfleetError::from(UnsupportedCapabilityError::new(
                            p.id(),
                            ProviderCapability::Issues,
                        ))
                    })?;

                    ops.comment_on_issue(&repo_str, number, &body).await?
                }

                CommentTargetArg::Change => {
                    let ops = p.change_ops().ok_or_else(|| {
                        GitfleetError::from(UnsupportedCapabilityError::new(
                            p.id(),
                            ProviderCapability::Changes,
                        ))
                    })?;

                    ops.comment_on_change(&repo_str, number, &body).await?
                }
            };

            if app.renderer().is_json() {
                app.renderer().write_result(&result);
            } else {
                app.renderer()
                    .render_success_box("Comment created", &format!("on #{}", number));
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_comment_list() {
        let app = test_helpers::make_app();

        run(
            CommentCmdCommand::List {
                number: 42,
                repo: Some("org/repo1".into()),
                target: CommentTargetArg::Change,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_comment_list_json() {
        let app = test_helpers::make_app_json();

        run(
            CommentCmdCommand::List {
                number: 42,
                repo: Some("org/repo1".into()),
                target: CommentTargetArg::Change,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_comment_list_human() {
        let app = test_helpers::make_app_human();

        run(
            CommentCmdCommand::List {
                number: 42,
                repo: Some("org/repo1".into()),
                target: CommentTargetArg::Change,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_comment_create() {
        let app = test_helpers::make_app();

        run(
            CommentCmdCommand::Create {
                number: 42,
                body: "Looks good!".into(),
                repo: Some("org/repo1".into()),
                target: CommentTargetArg::Change,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_comment_create_json() {
        let app = test_helpers::make_app_json();

        run(
            CommentCmdCommand::Create {
                number: 42,
                body: "LGTM".into(),
                repo: Some("org/repo1".into()),
                target: CommentTargetArg::Change,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_comment_create_human() {
        let app = test_helpers::make_app_human();

        run(
            CommentCmdCommand::Create {
                number: 42,
                body: "Nice work".into(),
                repo: Some("org/repo1".into()),
                target: CommentTargetArg::Change,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_comment_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            CommentCmdCommand::List {
                number: 42,
                repo: Some("org/repo1".into()),
                target: CommentTargetArg::Change,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_comment_create_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            CommentCmdCommand::Create {
                number: 42,
                body: "text".into(),
                repo: Some("org/repo1".into()),
                target: CommentTargetArg::Change,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_comment_list_issue() {
        let app = test_helpers::make_app();

        run(
            CommentCmdCommand::List {
                number: 42,
                repo: Some("org/repo1".into()),
                target: CommentTargetArg::Issue,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_comment_create_issue() {
        let app = test_helpers::make_app();

        run(
            CommentCmdCommand::Create {
                number: 42,
                body: "Reproduced".into(),
                repo: Some("org/repo1".into()),
                target: CommentTargetArg::Issue,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_comment_issue_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            CommentCmdCommand::List {
                number: 42,
                repo: Some("org/repo1".into()),
                target: CommentTargetArg::Issue,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
