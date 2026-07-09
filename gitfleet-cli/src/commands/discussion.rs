use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum DiscussionCommand {
    #[command(about = "List discussions.")]
    List {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long, default_value = "10")]
        limit: u32,
    },

    #[command(about = "View a discussion.")]
    View {
        number: u64,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Create a discussion.")]
    Create {
        title: String,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        category_id: Option<String>,
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(cmd: DiscussionCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    let ops = p.discussion_ops().ok_or_else(|| {
        GitfleetError::UnsupportedCapability(UnsupportedCapabilityError::new(
            app.provider_id(),
            ProviderCapability::Discussions,
        ))
    })?;

    match cmd {
        DiscussionCommand::List {
            repo,
            category,
            limit,
        } => {
            let repo_str = match repo {
                Some(r) => r,
                None => {
                    let remote = gitfleet_core::git::get_remote_url(None)?;

                    let parsed = gitfleet_core::repository::repository_ref_from_remote(&remote)?;
                    parsed.full_name()
                }
            };

            let (owner, name) = crate::repo_util::split_repo(&repo_str)?;

            service::discussions::list(p, app.renderer(), owner, name, category.as_deref(), limit)
                .await
        }

        DiscussionCommand::View { number, repo } => {
            let repo_str = match repo {
                Some(r) => r,
                None => {
                    let remote = gitfleet_core::git::get_remote_url(None)?;

                    let parsed = gitfleet_core::repository::repository_ref_from_remote(&remote)?;
                    parsed.full_name()
                }
            };

            let (owner, name) = crate::repo_util::split_repo(&repo_str)?;

            let result = ops.get_discussion(owner, name, number).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize discussion: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("Discussion", &result.title);
            }

            Ok(())
        }

        DiscussionCommand::Create {
            title,
            body,
            category_id,
            repo,
        } => {
            let repo_str = match repo {
                Some(r) => r,
                None => {
                    let remote = gitfleet_core::git::get_remote_url(None)?;

                    let parsed = gitfleet_core::repository::repository_ref_from_remote(&remote)?;
                    parsed.full_name()
                }
            };

            let (owner, name) = crate::repo_util::split_repo(&repo_str)?;

            let body_str = body.as_deref().unwrap_or("");
            let result = ops
                .create_discussion(owner, name, &title, body_str, category_id.as_deref())
                .await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize discussion: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("Discussion created", &title);
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
    async fn test_discussion_list() {
        let app = test_helpers::make_app();

        run(
            DiscussionCommand::List {
                repo: Some("org/repo".into()),
                category: None,
                limit: 10,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_discussion_list_with_category() {
        let app = test_helpers::make_app();

        run(
            DiscussionCommand::List {
                repo: Some("org/repo".into()),
                category: Some("Q&A".into()),
                limit: 5,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_discussion_list_bad_repo_format() {
        let app = test_helpers::make_app();

        let result = run(
            DiscussionCommand::List {
                repo: Some("invalidrepo".into()),
                category: None,
                limit: 10,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_discussion_view() {
        let app = test_helpers::make_app();

        run(
            DiscussionCommand::View {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_discussion_view_json() {
        let app = test_helpers::make_app_json();

        run(
            DiscussionCommand::View {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_discussion_create() {
        let app = test_helpers::make_app();

        run(
            DiscussionCommand::Create {
                title: "New discussion".into(),
                body: Some("Body text".into()),
                category_id: Some("general".into()),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_discussion_create_json() {
        let app = test_helpers::make_app_json();

        run(
            DiscussionCommand::Create {
                title: "New discussion".into(),
                body: None,
                category_id: None,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_discussion_create_bad_repo_format() {
        let app = test_helpers::make_app();

        let result = run(
            DiscussionCommand::Create {
                title: "New discussion".into(),
                body: None,
                category_id: None,
                repo: Some("invalidrepo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_discussion_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            DiscussionCommand::List {
                repo: Some("org/repo".into()),
                category: None,
                limit: 10,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
