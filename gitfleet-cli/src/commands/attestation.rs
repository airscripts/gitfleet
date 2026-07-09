use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum AttestationCommand {
    #[command(about = "List attestations.")]
    List {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        subject_digest: String,
    },

    #[command(about = "View an attestation.")]
    View {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        number: u64,
    },
}

pub async fn run(cmd: AttestationCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        AttestationCommand::List {
            repo,
            subject_digest,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.attestation_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Attestations,
                ))
            })?;

            let data = ops.list_attestations(&repo_str, &subject_digest).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                let items = data.as_array().cloned().unwrap_or_default();

                let rows: Vec<serde_json::Value> = items
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "TYPE": a.get("attestation_type"),
                            "BUNDLE": a.get("bundle_type"),
                        })
                    })
                    .collect();

                app.renderer().render_table_titled(
                    &rows,
                    Some("No attestations found."),
                    Some("Attestations"),
                    Some(&["TYPE", "BUNDLE"]),
                );
            }

            Ok(())
        }

        AttestationCommand::View { repo, number } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let ops = p.attestation_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Attestations,
                ))
            })?;

            let data = ops.get_attestation(&repo_str, number).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer()
                    .render_success_box("Attestation", &format!("{data}"));
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
    async fn test_attestation_list() {
        let app = test_helpers::make_app();

        run(
            AttestationCommand::List {
                repo: Some("org/repo".into()),
                subject_digest: "sha256:abc123".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_attestation_list_json() {
        let app = test_helpers::make_app_json();

        run(
            AttestationCommand::List {
                repo: Some("org/repo".into()),
                subject_digest: "sha256:abc123".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_attestation_list_human() {
        let app = test_helpers::make_app_human();

        run(
            AttestationCommand::List {
                repo: Some("org/repo".into()),
                subject_digest: "sha256:abc123".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_attestation_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            AttestationCommand::List {
                repo: Some("org/repo".into()),
                subject_digest: "sha256:abc123".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_attestation_view() {
        let app = test_helpers::make_app();

        run(
            AttestationCommand::View {
                repo: Some("org/repo".into()),
                number: 1,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_attestation_view_json() {
        let app = test_helpers::make_app_json();

        run(
            AttestationCommand::View {
                repo: Some("org/repo".into()),
                number: 1,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_attestation_view_human() {
        let app = test_helpers::make_app_human();

        run(
            AttestationCommand::View {
                repo: Some("org/repo".into()),
                number: 1,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_attestation_view_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            AttestationCommand::View {
                repo: Some("org/repo".into()),
                number: 1,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
