use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum AdvisoryCommand {
    #[command(about = "List security advisories.")]
    List {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        state: Option<String>,
    },

    #[command(about = "View a security advisory.")]
    View {
        number: u64,
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(cmd: AdvisoryCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        AdvisoryCommand::List { repo, state } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.advisory_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Advisories,
                ))
            })?;

            let data = ops
                .list_dependabot_alerts(&repo_str, state.as_deref())
                .await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let items = data.as_array().cloned().unwrap_or_default();

                let rows: Vec<serde_json::Value> = items
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "NUMBER": a.get("number"),
                            "SEVERITY": a.get("security_advisory").and_then(|s| s.get("severity")),
                            "STATE": a.get("state"),
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No advisories found."),
                    Some("Advisories"),
                    None,
                );
            }

            Ok(())
        }

        AdvisoryCommand::View { number, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.advisory_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Advisories,
                ))
            })?;

            let data = ops.get_dependabot_alert(&repo_str, number).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer()
                    .render_success_box("Advisory", &number.to_string());
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
    async fn test_advisory_list() {
        let app = test_helpers::make_app();

        run(
            AdvisoryCommand::List {
                repo: Some("org/repo".into()),
                state: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_advisory_list_with_state() {
        let app = test_helpers::make_app();

        run(
            AdvisoryCommand::List {
                repo: Some("org/repo".into()),
                state: Some("open".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_advisory_list_json() {
        let app = test_helpers::make_app_json();

        run(
            AdvisoryCommand::List {
                repo: Some("org/repo".into()),
                state: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_advisory_list_human() {
        let app = test_helpers::make_app_human();

        run(
            AdvisoryCommand::List {
                repo: Some("org/repo".into()),
                state: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_advisory_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            AdvisoryCommand::List {
                repo: Some("org/repo".into()),
                state: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_advisory_view() {
        let app = test_helpers::make_app();

        run(
            AdvisoryCommand::View {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_advisory_view_json() {
        let app = test_helpers::make_app_json();

        run(
            AdvisoryCommand::View {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_advisory_view_human() {
        let app = test_helpers::make_app_human();

        run(
            AdvisoryCommand::View {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_advisory_view_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            AdvisoryCommand::View {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
