use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum SecurityCommand {
    #[command(about = "List security advisories (Dependabot alerts).")]
    Advisories {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        state: Option<String>,
    },

    #[command(about = "List secret scanning alerts.")]
    SecretScans {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "List CodeQL alerts.")]
    Codeql {
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(cmd: SecurityCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        SecurityCommand::Advisories { repo, state } => {
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

        SecurityCommand::SecretScans { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.advisory_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Advisories,
                ))
            })?;

            let data = ops.list_secret_scanning_alerts(&repo_str, None).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let items = data.as_array().cloned().unwrap_or_default();

                let rows: Vec<serde_json::Value> = items
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "NUMBER": a.get("number"),
                            "SECRET": a.get("secret_type_display_name"),
                            "STATE": a.get("state"),
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No secret scanning alerts found."),
                    Some("Secret Scanning Alerts"),
                    None,
                );
            }

            Ok(())
        }

        SecurityCommand::Codeql { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.advisory_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Advisories,
                ))
            })?;

            let data = ops.list_codeql_alerts(&repo_str, None).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let items = data.as_array().cloned().unwrap_or_default();

                let rows: Vec<serde_json::Value> = items
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "NUMBER": a.get("number"),
                            "SEVERITY": a.get("rule").and_then(|r| r.get("security_severity_level")),
                            "STATE": a.get("state"),
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No CodeQL alerts found."),
                    Some("CodeQL Alerts"),
                    None,
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
    async fn test_security_advisories() {
        let app = test_helpers::make_app();

        run(
            SecurityCommand::Advisories {
                repo: Some("org/repo".into()),
                state: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_security_advisories_with_state() {
        let app = test_helpers::make_app();

        run(
            SecurityCommand::Advisories {
                repo: Some("org/repo".into()),
                state: Some("open".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_security_advisories_json() {
        let app = test_helpers::make_app_json();

        run(
            SecurityCommand::Advisories {
                repo: Some("org/repo".into()),
                state: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_security_advisories_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SecurityCommand::Advisories {
                repo: Some("org/repo".into()),
                state: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_security_secret_scans() {
        let app = test_helpers::make_app();

        run(
            SecurityCommand::SecretScans {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_security_secret_scans_json() {
        let app = test_helpers::make_app_json();

        run(
            SecurityCommand::SecretScans {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_security_secret_scans_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SecurityCommand::SecretScans {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_security_codeql() {
        let app = test_helpers::make_app();

        run(
            SecurityCommand::Codeql {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_security_codeql_json() {
        let app = test_helpers::make_app_json();

        run(
            SecurityCommand::Codeql {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }
}
