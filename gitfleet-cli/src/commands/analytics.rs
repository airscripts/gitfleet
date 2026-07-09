use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum AnalyticsCommand {
    #[command(about = "Show traffic views.")]
    Views {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Show traffic clones.")]
    Clones {
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(cmd: AnalyticsCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        AnalyticsCommand::Views { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.analytics_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Analytics,
                ))
            })?;

            let data = ops.get_traffic_views(&repo_str).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let count = data.get("count").and_then(|v| v.as_u64()).unwrap_or(0);

                let uniques = data.get("uniques").and_then(|v| v.as_u64()).unwrap_or(0);

                app.renderer().render_summary(
                    "Traffic Views",
                    &[
                        ("Total views", count.to_string()),
                        ("Unique visitors", uniques.to_string()),
                    ],
                );
            }

            Ok(())
        }

        AnalyticsCommand::Clones { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.analytics_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Analytics,
                ))
            })?;

            let data = ops.get_traffic_clones(&repo_str).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let count = data.get("count").and_then(|v| v.as_u64()).unwrap_or(0);

                let uniques = data.get("uniques").and_then(|v| v.as_u64()).unwrap_or(0);

                app.renderer().render_summary(
                    "Traffic Clones",
                    &[
                        ("Total clones", count.to_string()),
                        ("Unique cloners", uniques.to_string()),
                    ],
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
    async fn test_analytics_views() {
        let app = test_helpers::make_app();

        run(
            AnalyticsCommand::Views {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_analytics_views_json() {
        let app = test_helpers::make_app_json();

        run(
            AnalyticsCommand::Views {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_analytics_views_human() {
        let app = test_helpers::make_app_human();

        run(
            AnalyticsCommand::Views {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_analytics_views_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            AnalyticsCommand::Views {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analytics_clones() {
        let app = test_helpers::make_app();

        run(
            AnalyticsCommand::Clones {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_analytics_clones_json() {
        let app = test_helpers::make_app_json();

        run(
            AnalyticsCommand::Clones {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_analytics_clones_human() {
        let app = test_helpers::make_app_human();

        run(
            AnalyticsCommand::Clones {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_analytics_clones_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            AnalyticsCommand::Clones {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
