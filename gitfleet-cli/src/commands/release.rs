use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum ReleaseCommand {
    #[command(about = "List releases.")]
    List {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, default_value = "10")]
        limit: u32,
    },

    #[command(about = "View a release.")]
    View {
        tag: String,
        #[arg(long)]
        repo: Option<String>,
    },

    #[command(about = "Create a release.")]
    Create {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        tag: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        prerelease: bool,
        #[arg(long)]
        draft: bool,
    },

    #[command(about = "Delete a release.")]
    Delete {
        #[arg(help = "Release tag (or numeric GitHub release ID).")]
        release: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(cmd: ReleaseCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        ReleaseCommand::List { repo, limit } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::releases::list(p, app.renderer(), &repo_str, limit).await
        }

        ReleaseCommand::View { tag, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::releases::view(p, app.renderer(), &repo_str, &tag).await
        }

        ReleaseCommand::Create {
            repo,
            tag,
            title,
            body,
            prerelease,
            draft,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let mut create_body = serde_json::json!({
                "tag_name": tag,
            });

            if let Some(t) = title {
                create_body["name"] = serde_json::Value::String(t);
            }

            if let Some(b) = body {
                create_body["body"] = serde_json::Value::String(b);
            }

            if prerelease {
                create_body["prerelease"] = serde_json::Value::Bool(true);
            }

            if draft {
                create_body["draft"] = serde_json::Value::Bool(true);
            }

            service::releases::create(p, app.renderer(), &repo_str, create_body).await
        }

        ReleaseCommand::Delete { release, repo, yes } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            if app.dry_run() {
                if app.renderer().is_json() {
                    app.renderer().write_result(&serde_json::json!({
                        "dry_run": true,
                        "action": "delete",
                        "target": format!("{repo_str} release {release}"),
                    }));
                } else {
                    app.renderer().render_box(
                        &format!("Would delete release {release} from {repo_str}"),
                        "warning",
                    );
                }

                return Ok(());
            }

            gitfleet_core::prompt::confirm_destructive(
                &format!("Delete release {release} permanently?"),
                app.renderer().mode(),
                app.renderer().yes() || yes,
            )?;

            service::releases::delete(p, app.renderer(), &repo_str, &release).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_release_list() {
        let app = test_helpers::make_app();

        run(
            ReleaseCommand::List {
                repo: Some("org/repo".into()),
                limit: 10,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_release_list_json() {
        let app = test_helpers::make_app_json();

        run(
            ReleaseCommand::List {
                repo: Some("org/repo".into()),
                limit: 5,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_release_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ReleaseCommand::List {
                repo: Some("org/repo".into()),
                limit: 10,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_release_view() {
        let app = test_helpers::make_app();

        run(
            ReleaseCommand::View {
                tag: "v1.0".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_release_view_json() {
        let app = test_helpers::make_app_json();

        run(
            ReleaseCommand::View {
                tag: "v1.0".into(),
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_release_create() {
        let app = test_helpers::make_app();

        run(
            ReleaseCommand::Create {
                repo: Some("org/repo".into()),
                tag: "v1.0".into(),
                title: Some("Release 1.0".into()),
                body: Some("Release notes".into()),
                prerelease: false,
                draft: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_release_create_json() {
        let app = test_helpers::make_app_json();

        run(
            ReleaseCommand::Create {
                repo: Some("org/repo".into()),
                tag: "v1.0".into(),
                title: None,
                body: None,
                prerelease: false,
                draft: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_release_create_prerelease() {
        let app = test_helpers::make_app();

        run(
            ReleaseCommand::Create {
                repo: Some("org/repo".into()),
                tag: "v2.0-pre".into(),
                title: Some("Pre-release".into()),
                body: None,
                prerelease: true,
                draft: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_release_create_draft() {
        let app = test_helpers::make_app();

        run(
            ReleaseCommand::Create {
                repo: Some("org/repo".into()),
                tag: "v3.0-draft".into(),
                title: None,
                body: Some("Draft notes".into()),
                prerelease: false,
                draft: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_release_delete_with_yes() {
        let app = test_helpers::make_app();

        run(
            ReleaseCommand::Delete {
                release: "1".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_release_delete_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(
            ReleaseCommand::Delete {
                release: "1".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_release_delete_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ReleaseCommand::Delete {
                release: "1".into(),
                repo: Some("org/repo".into()),
                yes: true,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
