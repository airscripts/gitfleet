use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnprocessableError, UnsupportedCapabilityError};
use gitfleet_core::provider::ProviderCapability;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum RepoCommand {
    #[command(about = "Create a repository.")]
    Create {
        name: String,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long, default_value = "user")]
        owner_type: String,
        #[arg(long)]
        public: bool,
        #[arg(long)]
        private: bool,
        #[arg(long)]
        internal: bool,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        template: Option<String>,
    },

    #[command(about = "List repositories.")]
    List {
        #[arg(long)]
        owner: Option<String>,
        #[arg(long, default_value = "user")]
        owner_type: String,
        #[arg(long, default_value = "all")]
        r#type: String,
    },

    #[command(about = "View repository details.")]
    View { repository: Option<String> },

    #[command(about = "Clone a repository.")]
    Clone {
        repository: String,
        #[arg(long)]
        depth: Option<u32>,
    },

    #[command(about = "Delete a repository.")]
    Delete {
        repository: String,
        #[arg(long)]
        yes: bool,
    },

    #[command(about = "Archive a repository.")]
    Archive { repository: String },

    #[command(about = "Unarchive a repository.")]
    Unarchive { repository: String },

    #[command(about = "Rename a repository.")]
    Rename {
        repository: String,
        new_name: String,
    },

    #[command(about = "Star a repository.")]
    Star { repository: String },

    #[command(about = "Remove a star from a repository.")]
    Unstar { repository: String },

    #[command(about = "Manage repository forks.")]
    Fork {
        #[command(subcommand)]
        subcommand: crate::commands::fork_cmd::ForkCmdCommand,
    },

    #[command(about = "Edit repository metadata.")]
    Edit {
        repository: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        homepage: Option<String>,
        #[arg(long)]
        visibility: Option<String>,
    },
}

pub async fn run(cmd: RepoCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        RepoCommand::Create {
            name,
            owner,
            owner_type,
            public,
            private,
            internal,
            description,
            template: _template,
        } => {
            if [public, private, internal].iter().filter(|&&f| f).count() > 1 {
                return Err(GitfleetError::from(UnprocessableError::new(
                    "Visibility flags are mutually exclusive.",
                )));
            }

            let visibility = if private {
                "private"
            } else if internal {
                "internal"
            } else {
                "public"
            };

            let owner_arg = owner.as_deref();

            let owner_type_arg = if owner_type == "org" { "org" } else { "user" };
            service::repos::create(
                p,
                app.renderer(),
                &name,
                owner_arg,
                Some(owner_type_arg),
                visibility,
                description.as_deref(),
            )
            .await
        }

        RepoCommand::List {
            owner, owner_type, ..
        } => {
            let (org, username) = match (owner.as_deref(), owner_type.as_str()) {
                (Some(o), "org") => (Some(o), None),
                (Some(o), _) => (None, Some(o)),
                (None, _) => (None, None),
            };

            service::repos::list(p, app.renderer(), org, username).await
        }

        RepoCommand::View { repository } => {
            let repo = repository.as_deref().unwrap_or("");

            if repo.is_empty() {
                return Err(GitfleetError::from(UnprocessableError::new(
                    gitfleet_core::constants::ERROR_NO_REPO,
                )));
            }

            service::repos::view(p, app.renderer(), repo).await
        }

        RepoCommand::Clone { repository, depth } => {
            let host = p.default_host();

            let url = format!("https://{host}/{repository}");
            let depth_display = match depth {
                Some(d) => format!("depth: {d}"),
                None => "full depth".to_string(),
            };

            gitfleet_core::git::clone_repository(&url, depth, None, None)?;

            app.renderer()
                .render_success_box("Cloned", &format!("{repository} ({depth_display})"));

            Ok(())
        }

        RepoCommand::Delete { repository, yes } => {
            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": repository,
                    }));
                } else {
                    app.renderer()
                        .render_box(&format!("Would delete {repository}"), "warning");
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete {repository} permanently?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            service::repos::delete(p, app.renderer(), &repository).await
        }

        RepoCommand::Archive { repository } => {
            let ops = p.repo_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Repositories,
                ))
            })?;

            ops.archive_repo(&repository).await?;

            app.renderer().render_success_box("Archived", &repository);

            Ok(())
        }

        RepoCommand::Unarchive { repository } => {
            let ops = p.repo_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Repositories,
                ))
            })?;

            ops.unarchive_repo(&repository).await?;

            app.renderer().render_success_box("Unarchived", &repository);

            Ok(())
        }

        RepoCommand::Rename {
            repository,
            new_name,
        } => {
            let ops = p.repo_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Repositories,
                ))
            })?;

            let result = ops
                .update_repo(&repository, serde_json::json!({"name": new_name}))
                .await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&result);
            } else {
                app.renderer()
                    .render_success_box("Renamed", &format!("{repository} -> {new_name}"));
            }

            Ok(())
        }

        RepoCommand::Star { repository } => {
            service::repos::star(p, app.renderer(), &repository).await
        }

        RepoCommand::Unstar { repository } => {
            service::repos::unstar(p, app.renderer(), &repository).await
        }

        RepoCommand::Fork { subcommand } => crate::commands::fork_cmd::run(subcommand, app).await,
        RepoCommand::Edit {
            repository,
            description,
            homepage,
            visibility,
        } => {
            let ops = p.repo_ops().ok_or_else(|| {
                GitfleetError::from(UnsupportedCapabilityError::new(
                    p.id(),
                    ProviderCapability::Repositories,
                ))
            })?;

            let mut update = serde_json::json!({});

            if let Some(desc) = description {
                update["description"] = serde_json::json!(desc);
            }

            if let Some(hp) = homepage {
                update["homepage"] = serde_json::json!(hp);
            }

            if let Some(vis) = visibility {
                update["visibility"] = serde_json::json!(vis);
            }

            let result = ops.update_repo(&repository, update).await?;

            if app.renderer().is_json() {
                app.renderer().write_result(&result);
            } else {
                app.renderer().render_success_box("Updated", &repository);
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
    async fn test_repo_create_public() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::Create {
                name: "new-repo".into(),
                owner: None,
                owner_type: "user".into(),
                public: true,
                private: false,
                internal: false,
                description: None,
                template: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_create_private() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::Create {
                name: "new-repo".into(),
                owner: Some("org".into()),
                owner_type: "org".into(),
                public: false,
                private: true,
                internal: false,
                description: Some("Test".into()),
                template: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_create_mutually_exclusive() {
        let app = test_helpers::make_app();

        let result = run(
            RepoCommand::Create {
                name: "new-repo".into(),
                owner: None,
                owner_type: "user".into(),
                public: true,
                private: true,
                internal: false,
                description: None,
                template: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_repo_list() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::List {
                owner: None,
                owner_type: "user".into(),
                r#type: "all".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_list_org() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::List {
                owner: Some("org".into()),
                owner_type: "org".into(),
                r#type: "all".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_view() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::View {
                repository: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_view_empty() {
        let app = test_helpers::make_app();

        let result = run(RepoCommand::View { repository: None }, &app).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_repo_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            RepoCommand::Delete {
                repository: "org/repo".into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::Delete {
                repository: "org/repo".into(),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_archive() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::Archive {
                repository: "org/repo".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_unarchive() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::Unarchive {
                repository: "org/repo".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_rename() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::Rename {
                repository: "org/repo".into(),
                new_name: "new-repo".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_rename_json() {
        let app = test_helpers::make_app_json();

        run(
            RepoCommand::Rename {
                repository: "org/repo".into(),
                new_name: "new-repo".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_star() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::Star {
                repository: "org/repo".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_unstar() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::Unstar {
                repository: "org/repo".into(),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_edit() {
        let app = test_helpers::make_app();

        run(
            RepoCommand::Edit {
                repository: "org/repo".into(),
                description: Some("Updated".into()),
                homepage: Some("https://example.com".into()),
                visibility: Some("private".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_edit_json() {
        let app = test_helpers::make_app_json();

        run(
            RepoCommand::Edit {
                repository: "org/repo".into(),
                description: None,
                homepage: None,
                visibility: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_repo_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            RepoCommand::Archive {
                repository: "org/repo".into(),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
