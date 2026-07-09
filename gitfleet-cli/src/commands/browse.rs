use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum BrowseCommand {
    #[command(about = "Open a repository in the browser.")]
    Open {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        path: Option<String>,
    },
}

pub async fn run(cmd: BrowseCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        BrowseCommand::Open { repo, path } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::browse::open(p, app.renderer(), &repo_str, path.as_deref()).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_browse_open() {
        let app = test_helpers::make_app();

        run(
            BrowseCommand::Open {
                repo: Some("org/repo".into()),
                path: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_browse_open_with_path() {
        let app = test_helpers::make_app();

        run(
            BrowseCommand::Open {
                repo: Some("org/repo".into()),
                path: Some("README.md".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_browse_open_path_subdir() {
        let app = test_helpers::make_app();

        run(
            BrowseCommand::Open {
                repo: Some("org/repo".into()),
                path: Some("src/main.rs".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_browse_open_json() {
        let app = test_helpers::make_app_json();

        run(
            BrowseCommand::Open {
                repo: Some("org/repo".into()),
                path: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_browse_open_json_with_path() {
        let app = test_helpers::make_app_json();

        run(
            BrowseCommand::Open {
                repo: Some("org/repo".into()),
                path: Some("src/main.rs".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_browse_open_human() {
        let app = test_helpers::make_app_human();

        run(
            BrowseCommand::Open {
                repo: Some("org/repo".into()),
                path: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_browse_open_no_repo_fallback() {
        let app = test_helpers::make_app();

        let result = run(
            BrowseCommand::Open {
                repo: None,
                path: None,
            },
            &app,
        )
        .await;

        let _ = result;
    }

    #[tokio::test]
    async fn test_browse_open_no_repo_fallback_json() {
        let app = test_helpers::make_app_json();

        let result = run(
            BrowseCommand::Open {
                repo: None,
                path: None,
            },
            &app,
        )
        .await;

        let _ = result;
    }
}
