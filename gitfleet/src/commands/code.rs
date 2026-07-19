use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum CodeCommand {
    #[command(about = "Search code.")]
    Search {
        query: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        language: Option<String>,
        #[arg(long, default_value = "30")]
        limit: u32,
        #[arg(long)]
        page: Option<u32>,
    },

    #[command(about = "View file contents.")]
    View {
        path: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        r#ref: Option<String>,
    },
}

pub async fn run(cmd: CodeCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        CodeCommand::Search {
            query,
            repo,
            language,
            limit,
            page,
        } => {
            crate::commands::validate_page(page)?;
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.code_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Code,
                ))
            })?;

            let results = ops
                .search_code(&query, Some(&repo_str), language.as_deref(), limit, page)
                .await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&results)
                    .map_err(|e| GitfleetError::new(format!("Failed to serialize results: {e}")))?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = results
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "FILE": r.file,
                            "REPO": r.repo,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No code results found."),
                    Some("Code Results"),
                    None,
                );
            }

            Ok(())
        }

        CodeCommand::View { path, repo, r#ref } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.code_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Code,
                ))
            })?;

            let data = ops
                .get_file_contents(&repo_str, &path, r#ref.as_deref())
                .await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let content = data
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no content)");

                app.renderer().write_value(content);
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
    async fn test_code_search() {
        let app = test_helpers::make_app();

        run(
            CodeCommand::Search {
                query: "fn main".into(),
                repo: Some("org/repo".into()),
                language: None,
                limit: 10,
                page: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_code_search_with_language() {
        let app = test_helpers::make_app();

        run(
            CodeCommand::Search {
                query: "fn main".into(),
                repo: Some("org/repo".into()),
                language: Some("rust".into()),
                limit: 5,
                page: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_code_search_json() {
        let app = test_helpers::make_app_json();

        run(
            CodeCommand::Search {
                query: "fn main".into(),
                repo: Some("org/repo".into()),
                language: None,
                limit: 10,
                page: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_code_search_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            CodeCommand::Search {
                query: "fn main".into(),
                repo: Some("org/repo".into()),
                language: None,
                limit: 10,
                page: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_code_view() {
        let app = test_helpers::make_app();

        run(
            CodeCommand::View {
                path: "src/main.rs".into(),
                repo: Some("org/repo".into()),
                r#ref: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_code_view_with_ref() {
        let app = test_helpers::make_app();

        run(
            CodeCommand::View {
                path: "src/main.rs".into(),
                repo: Some("org/repo".into()),
                r#ref: Some("main".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_code_view_json() {
        let app = test_helpers::make_app_json();

        run(
            CodeCommand::View {
                path: "README.md".into(),
                repo: Some("org/repo".into()),
                r#ref: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_code_view_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            CodeCommand::View {
                path: "src/main.rs".into(),
                repo: Some("org/repo".into()),
                r#ref: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
