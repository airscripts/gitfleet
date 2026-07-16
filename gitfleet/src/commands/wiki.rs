use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum WikiCommand {
    #[command(about = "List wiki pages.")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "View a wiki page.")]
    View {
        page: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Create a wiki page.")]
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Edit a wiki page.")]
    Edit {
        page: String,
        #[arg(long)]
        content: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Delete a wiki page.")]
    Delete {
        page: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: WikiCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    let ops = p.wiki_ops().ok_or_else(|| {
        GitfleetError::UnsupportedCapability(UnsupportedCapabilityError::new(
            app.provider_id(),
            ProviderCapability::Wiki,
        ))
    })?;

    match cmd {
        WikiCommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::wiki::list_pages(p, app.renderer(), &repo_str).await
        }

        WikiCommand::View { page, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let result = ops.get_wiki_page(&repo_str, &page).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize wiki page: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("Wiki page", &result.page.title);
            }

            Ok(())
        }

        WikiCommand::Create {
            title,
            content,
            repo,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let result = ops.create_wiki_page(&repo_str, &title, &content).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize wiki page: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("Wiki page created", &title);
            }

            Ok(())
        }

        WikiCommand::Edit {
            page,
            content,
            repo,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let result = ops.update_wiki_page(&repo_str, &page, &content).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize wiki page: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("Wiki page updated", &page);
            }

            Ok(())
        }

        WikiCommand::Delete { page, repo, yes } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("wiki page {page}"),
                    }));
                } else {
                    app.renderer()
                        .render_box(&format!("Would delete wiki page {page}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete wiki page {page}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            ops.delete_wiki_page(&repo_str, &page).await?;

            app.renderer()
                .render_success_box("Wiki page deleted", &page);

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_wiki_list() {
        let app = test_helpers::make_app();

        run(
            WikiCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_wiki_view() {
        let app = test_helpers::make_app();

        run(
            WikiCommand::View {
                page: "Home".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_wiki_view_json() {
        let app = test_helpers::make_app_json();

        run(
            WikiCommand::View {
                page: "Home".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_wiki_create() {
        let app = test_helpers::make_app();

        run(
            WikiCommand::Create {
                title: "New Page".into(),
                content: "# Content".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_wiki_create_json() {
        let app = test_helpers::make_app_json();

        run(
            WikiCommand::Create {
                title: "New Page".into(),
                content: "# Content".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_wiki_edit() {
        let app = test_helpers::make_app();

        run(
            WikiCommand::Edit {
                page: "Home".into(),
                content: "Updated".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_wiki_edit_json() {
        let app = test_helpers::make_app_json();

        run(
            WikiCommand::Edit {
                page: "Home".into(),
                content: "Updated".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_wiki_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            WikiCommand::Delete {
                page: "Home".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_wiki_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            WikiCommand::Delete {
                page: "Home".into(),
                repo: Some("org/repo".into()),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_wiki_delete_non_interactive_no_yes() {
        let app = test_helpers::make_app_json();

        let result = run(
            WikiCommand::Delete {
                page: "Home".into(),
                repo: Some("org/repo".into()),
                yes: false,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_wiki_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            WikiCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
