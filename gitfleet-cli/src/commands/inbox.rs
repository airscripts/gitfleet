use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum InboxCommand {
    #[command(about = "List notifications.")]
    List {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        participating: bool,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Mark notifications as read.")]
    MarkRead {
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: InboxCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        InboxCommand::List {
            all,
            participating,
            repo,
        } => service::inbox::list(p, app.renderer(), all, participating, repo.as_deref()).await,
        InboxCommand::MarkRead { yes } => {
            let ops = p.notification_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Notifications,
                ))
            })?;

            if app.dry_run() {
                let preview = serde_json::json!({
                    "dry_run": true,
                    "action": "mark-read",
                    "target": "all notifications",
                });

                if app.renderer().is_json() {
                    app.renderer().write_result(&preview);
                } else {
                    app.renderer()
                        .render_box("Would mark all notifications as read", "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                "Mark all notifications as read?",
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            ops.mark_notifications_read().await?;

            app.renderer()
                .write_value("All notifications marked as read.");

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_inbox_list() {
        let app = test_helpers::make_app();

        run(
            InboxCommand::List {
                all: false,
                participating: false,
                repo: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_inbox_list_all() {
        let app = test_helpers::make_app();

        run(
            InboxCommand::List {
                all: true,
                participating: false,
                repo: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_inbox_list_participating() {
        let app = test_helpers::make_app();

        run(
            InboxCommand::List {
                all: false,
                participating: true,
                repo: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_inbox_list_with_repo() {
        let app = test_helpers::make_app();

        run(
            InboxCommand::List {
                all: false,
                participating: false,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_inbox_list_all_and_participating() {
        let app = test_helpers::make_app();

        run(
            InboxCommand::List {
                all: true,
                participating: true,
                repo: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_inbox_list_json() {
        let app = test_helpers::make_app_json();

        run(
            InboxCommand::List {
                all: false,
                participating: false,
                repo: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_inbox_list_human() {
        let app = test_helpers::make_app_human();

        run(
            InboxCommand::List {
                all: false,
                participating: false,
                repo: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_inbox_mark_read() {
        let app = test_helpers::make_app();

        run(InboxCommand::MarkRead { yes: true }, &app)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_inbox_mark_read_json() {
        let app = test_helpers::make_app_json();

        run(InboxCommand::MarkRead { yes: true }, &app)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_inbox_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            InboxCommand::List {
                all: false,
                participating: false,
                repo: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_inbox_mark_read_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(InboxCommand::MarkRead { yes: true }, &app).await;

        assert!(result.is_err());
    }
}
