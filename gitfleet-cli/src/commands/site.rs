use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum SiteCommand {
    #[command(about = "Get Pages site information for a repository.")]
    Get {
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Enable Pages for a repository.")]
    Create {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, help = "Source branch or directory (e.g. main or main/docs)")]
        source: String,
        #[arg(long, help = "Build type: workflow or legacy")]
        build_type: Option<String>,
    },

    #[command(about = "Disable Pages for a repository.")]
    Delete {
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(cmd: SiteCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    let ops = p.site_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            p.id(),
            ProviderCapability::Site,
        ))
    })?;

    match cmd {
        SiteCommand::Get { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let data = ops.get_pages(&repo_str).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer()
                    .render_success_box("Pages", &format!("{data}"));
            }

            Ok(())
        }

        SiteCommand::Create {
            repo,
            source,
            build_type,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let data = ops
                .create_pages(&repo_str, &source, build_type.as_deref())
                .await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&data);
            } else {
                app.renderer()
                    .render_success_box("Pages Created", &format!("{data}"));
            }

            Ok(())
        }

        SiteCommand::Delete { repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            ops.remove_pages(&repo_str).await?;

            if app.renderer().is_json() {
                app.renderer()
                    .write_result(&serde_json::json!({"deleted": true}));
            } else {
                app.renderer()
                    .render_success_box("Pages site deleted", &repo_str);
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
    async fn test_site_get() {
        let app = test_helpers::make_app();

        run(
            SiteCommand::Get {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_site_get_json() {
        let app = test_helpers::make_app_json();

        run(
            SiteCommand::Get {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_site_get_human() {
        let app = test_helpers::make_app_human();

        run(
            SiteCommand::Get {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_site_get_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SiteCommand::Get {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_site_create() {
        let app = test_helpers::make_app();

        run(
            SiteCommand::Create {
                repo: Some("org/repo".into()),
                source: "main".into(),
                build_type: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_site_create_json() {
        let app = test_helpers::make_app_json();

        run(
            SiteCommand::Create {
                repo: Some("org/repo".into()),
                source: "main".into(),
                build_type: Some("workflow".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_site_create_human() {
        let app = test_helpers::make_app_human();

        run(
            SiteCommand::Create {
                repo: Some("org/repo".into()),
                source: "main/docs".into(),
                build_type: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_site_create_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SiteCommand::Create {
                repo: Some("org/repo".into()),
                source: "main".into(),
                build_type: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_site_delete() {
        let app = test_helpers::make_app();

        run(
            SiteCommand::Delete {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_site_delete_json() {
        let app = test_helpers::make_app_json();

        run(
            SiteCommand::Delete {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_site_delete_human() {
        let app = test_helpers::make_app_human();

        run(
            SiteCommand::Delete {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_site_delete_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SiteCommand::Delete {
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
