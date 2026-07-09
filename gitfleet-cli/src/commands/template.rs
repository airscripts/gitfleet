use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum TemplateCommand {
    #[command(about = "List issue templates.")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(cmd: TemplateCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        TemplateCommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.template_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Templates,
                ))
            })?;

            let templates = ops.list_issue_templates(&repo_str).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&templates).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize templates: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = templates
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "NAME": t.name,
                            "ABOUT": t.about,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No issue templates found."),
                    Some("Templates"),
                    Some(&["NAME", "ABOUT"]),
                );
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
    async fn test_template_list() {
        let app = test_helpers::make_app();

        run(
            TemplateCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_template_list_json() {
        let app = test_helpers::make_app_json();

        run(
            TemplateCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_template_list_human() {
        let app = test_helpers::make_app_human();

        run(
            TemplateCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_template_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            TemplateCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
