use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum DependencyCommand {
    #[command(about = "List dependencies (SBOM).")]
    List {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Review dependencies.")]
    Review {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        base: String,
        #[arg(long)]
        head: String,
    },
}

pub async fn run(cmd: DependencyCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        DependencyCommand::List { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.dependency_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Dependencies,
                ))
            })?;

            let data = ops.sbom(&repo_str).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer().write_value("Dependencies listed.");
            }

            Ok(())
        }

        DependencyCommand::Review { repo, base, head } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.dependency_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Dependencies,
                ))
            })?;

            let data = ops.review_dependencies(&repo_str, &base, &head).await?;

            if app.renderer().is_json() {
                let json = serde_json::to_value(&data).map_err(|e| {
                    GitfleetError::new(format!("Failed to serialize dependency review: {e}"))
                })?;

                app.renderer().write_result(&json);
            } else {
                let rows: Vec<serde_json::Value> = data
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "CHANGE": c.change_type,
                            "PACKAGE": c.package,
                            "ECOSYSTEM": c.ecosystem,
                            "VERSION": c.version,
                            "SEVERITY": c.severity,
                            "VULNS": c.vulnerabilities,
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No dependency review changes found."),
                    Some("Dependency Changes"),
                    Some(&[
                        "CHANGE",
                        "PACKAGE",
                        "ECOSYSTEM",
                        "VERSION",
                        "SEVERITY",
                        "VULNS",
                    ]),
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
    async fn test_dependency_list() {
        let app = test_helpers::make_app();

        run(
            DependencyCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dependency_list_json() {
        let app = test_helpers::make_app_json();

        run(
            DependencyCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dependency_list_human() {
        let app = test_helpers::make_app_human();

        run(
            DependencyCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dependency_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            DependencyCommand::List {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dependency_review() {
        let app = test_helpers::make_app();

        run(
            DependencyCommand::Review {
                repo: Some("org/repo".into()),
                base: "main".into(),
                head: "feature".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dependency_review_json() {
        let app = test_helpers::make_app_json();

        run(
            DependencyCommand::Review {
                repo: Some("org/repo".into()),
                base: "main".into(),
                head: "feature".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dependency_review_human() {
        let app = test_helpers::make_app_human();

        run(
            DependencyCommand::Review {
                repo: Some("org/repo".into()),
                base: "main".into(),
                head: "feature".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_dependency_review_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            DependencyCommand::Review {
                repo: Some("org/repo".into()),
                base: "main".into(),
                head: "feature".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
