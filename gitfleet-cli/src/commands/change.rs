use clap::Subcommand;
use gitfleet_core::errors::{GitfleetError, UnprocessableError};

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum ChangeCommand {
    #[command(about = "Create a change request.")]
    Create {
        #[arg(long)]
        repo: Option<String>,
        title: String,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        head: Option<String>,
        #[arg(long)]
        draft: bool,
    },

    #[command(about = "List change requests.")]
    List {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, default_value = "open")]
        state: String,
        #[arg(long)]
        base_branch: Option<String>,
        #[arg(long, default_value = "10")]
        limit: u32,
    },

    #[command(about = "View a change request.")]
    View {
        number: u64,
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(cmd: ChangeCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        ChangeCommand::Create {
            repo,
            title,
            body,
            base,
            head,
            draft,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            let base_str = base.as_deref().unwrap_or("main");
            let head_str = head.as_deref().unwrap_or("");

            if head_str.is_empty() {
                return Err(GitfleetError::from(UnprocessableError::new(
                    "--head is required",
                )));
            }

            service::changes::create(
                p,
                app.renderer(),
                &repo_str,
                &title,
                head_str,
                base_str,
                body.as_deref(),
                draft,
            )
            .await
        }

        ChangeCommand::List {
            repo,
            state,
            base_branch,
            limit,
        } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::changes::list(
                p,
                app.renderer(),
                &repo_str,
                &state,
                limit,
                base_branch.as_deref(),
                None,
            )
            .await
        }

        ChangeCommand::View { number, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::changes::view(p, app.renderer(), &repo_str, number).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_change_create() {
        let app = test_helpers::make_app();

        run(
            ChangeCommand::Create {
                repo: Some("org/repo".into()),
                title: "Fix bug".into(),
                body: None,
                base: Some("main".into()),
                head: Some("feature".into()),
                draft: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_change_create_draft() {
        let app = test_helpers::make_app();

        run(
            ChangeCommand::Create {
                repo: Some("org/repo".into()),
                title: "WIP".into(),
                body: Some("Work in progress".into()),
                base: None,
                head: Some("feature".into()),
                draft: true,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_change_create_missing_head() {
        let app = test_helpers::make_app();

        let result = run(
            ChangeCommand::Create {
                repo: Some("org/repo".into()),
                title: "Fix bug".into(),
                body: None,
                base: Some("main".into()),
                head: None,
                draft: false,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_change_create_json() {
        let app = test_helpers::make_app_json();

        run(
            ChangeCommand::Create {
                repo: Some("org/repo".into()),
                title: "Fix bug".into(),
                body: None,
                base: Some("main".into()),
                head: Some("feature".into()),
                draft: false,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_change_create_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ChangeCommand::Create {
                repo: Some("org/repo".into()),
                title: "Fix bug".into(),
                body: None,
                base: Some("main".into()),
                head: Some("feature".into()),
                draft: false,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_change_list() {
        let app = test_helpers::make_app();

        run(
            ChangeCommand::List {
                repo: Some("org/repo".into()),
                state: "open".into(),
                base_branch: None,
                limit: 10,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_change_list_with_base() {
        let app = test_helpers::make_app();

        run(
            ChangeCommand::List {
                repo: Some("org/repo".into()),
                state: "closed".into(),
                base_branch: Some("main".into()),
                limit: 5,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_change_list_json() {
        let app = test_helpers::make_app_json();

        run(
            ChangeCommand::List {
                repo: Some("org/repo".into()),
                state: "open".into(),
                base_branch: None,
                limit: 10,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_change_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ChangeCommand::List {
                repo: Some("org/repo".into()),
                state: "open".into(),
                base_branch: None,
                limit: 10,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_change_view() {
        let app = test_helpers::make_app();

        run(
            ChangeCommand::View {
                number: 42,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_change_view_json() {
        let app = test_helpers::make_app_json();

        run(
            ChangeCommand::View {
                number: 42,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_change_view_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            ChangeCommand::View {
                number: 42,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
