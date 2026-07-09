use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum SnippetCommand {
    #[command(about = "List snippets.")]
    List {
        #[arg(long)]
        owner: Option<String>,
    },

    #[command(about = "View a snippet.")]
    View { id: String },

    #[command(about = "Create a snippet.")]
    Create {
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        public: bool,
        #[arg(long)]
        file: Option<String>,
    },

    #[command(about = "Delete a snippet.")]
    Delete {
        id: String,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: SnippetCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    let ops = p.snippet_ops().ok_or_else(|| {
        GitfleetError::UnsupportedCapability(UnsupportedCapabilityError::new(
            app.provider_id(),
            ProviderCapability::Snippets,
        ))
    })?;

    match cmd {
        SnippetCommand::List { owner } => {
            let data = if let Some(o) = owner {
                ops.list_snippets(&o).await?
            } else {
                ops.list_snippets("").await?
            };

            if app.renderer().is_json() {
                let json = serde_json::to_value(&data).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize snippets: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = data
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "ID": s.id,
                            "DESCRIPTION": s.description,
                            "PUBLIC": s.public,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No snippets found."),
                    Some("Snippets"),
                    Some(&["ID", "DESCRIPTION", "PUBLIC"]),
                );
            }

            Ok(())
        }

        SnippetCommand::View { id } => {
            let data = ops.get_snippet(&id).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer()
                    .render_success_box("Snippet", &format!("{}", data));
            }

            Ok(())
        }

        SnippetCommand::Create {
            description,
            public,
            file,
        } => {
            let desc = description.as_deref().unwrap_or("");

            let filename = file
                .as_deref()
                .and_then(|f| std::path::Path::new(f).file_name().and_then(|n| n.to_str()))
                .unwrap_or("snippet.txt");

            let content = file
                .as_deref()
                .map(|f| std::fs::read_to_string(f).unwrap_or_default())
                .unwrap_or_else(|| "gitfleet test snippet".to_string());

            let files = serde_json::json!({
                filename: { "content": content }
            });

            let result = ops.create_snippet(desc, public, files).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&result)
                    .map_err(|e| GitfleetError::new(format!("Failed to serialize snippet: {e}")))?;

                app.renderer().write_result(&json);
            } else {
                app.renderer()
                    .render_success_box("Snippet created", &result.id);
            }

            Ok(())
        }

        SnippetCommand::Delete { id, yes } => {
            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("snippet {id}"),
                    }));
                } else {
                    app.renderer()
                        .render_box(&format!("Would delete snippet {id}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete snippet {id}?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            ops.delete_snippet(&id).await?;

            app.renderer().render_success_box("Snippet deleted", &id);

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_snippet_list() {
        let app = test_helpers::make_app();

        run(
            SnippetCommand::List {
                owner: Some("dev".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_snippet_list_default_owner() {
        let app = test_helpers::make_app();

        run(SnippetCommand::List { owner: None }, &app)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_snippet_list_json() {
        let app = test_helpers::make_app_json();

        run(
            SnippetCommand::List {
                owner: Some("dev".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_snippet_view() {
        let app = test_helpers::make_app();

        run(SnippetCommand::View { id: "1".into() }, &app)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_snippet_view_json() {
        let app = test_helpers::make_app_json();

        run(SnippetCommand::View { id: "1".into() }, &app)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_snippet_create() {
        let app = test_helpers::make_app();

        run(
            SnippetCommand::Create {
                description: Some("snippet".into()),
                public: true,
                file: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_snippet_create_json() {
        let app = test_helpers::make_app_json();

        run(
            SnippetCommand::Create {
                description: None,
                public: false,
                file: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_snippet_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            SnippetCommand::Delete {
                id: "1".into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_snippet_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            SnippetCommand::Delete {
                id: "1".into(),
                yes: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_snippet_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(SnippetCommand::List { owner: None }, &app).await;

        assert!(result.is_err());
    }
}
