use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum IssueCommand {
    #[command(about = "Create an issue.")]
    Create {
        #[arg(long)]
        repo: Option<String>,
        title: String,
        #[arg(long)]
        body: Option<String>,
    },

    #[command(about = "List issues.")]
    List {
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, default_value = "open")]
        state: String,
        #[arg(long, default_value = "10")]
        limit: u32,
    },

    #[command(about = "View an issue.")]
    View {
        number: u64,
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(cmd: IssueCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        IssueCommand::Create { repo, title, body } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::issues::create(
                p,
                app.renderer(),
                &repo_str,
                &title,
                body.as_deref(),
                &[],
                &[],
            )
            .await
        }

        IssueCommand::List { repo, state, limit } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::issues::list(p, app.renderer(), &repo_str, &state, limit).await
        }

        IssueCommand::View { number, repo } => {
            let repo_str = crate::repo_util::resolve_repo(&repo)?;

            service::issues::view(p, app.renderer(), &repo_str, number).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_issue_create() {
        let app = test_helpers::make_app();

        run(
            IssueCommand::Create {
                repo: Some("org/repo".into()),
                title: "Bug".into(),
                body: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_issue_create_with_body() {
        let app = test_helpers::make_app();

        run(
            IssueCommand::Create {
                repo: Some("org/repo".into()),
                title: "Bug".into(),
                body: Some("Something broke".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_issue_create_json() {
        let app = test_helpers::make_app_json();

        run(
            IssueCommand::Create {
                repo: Some("org/repo".into()),
                title: "Bug".into(),
                body: None,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_issue_create_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            IssueCommand::Create {
                repo: Some("org/repo".into()),
                title: "Bug".into(),
                body: None,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_issue_list() {
        let app = test_helpers::make_app();

        run(
            IssueCommand::List {
                repo: Some("org/repo".into()),
                state: "open".into(),
                limit: 10,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_issue_list_closed() {
        let app = test_helpers::make_app();

        run(
            IssueCommand::List {
                repo: Some("org/repo".into()),
                state: "closed".into(),
                limit: 5,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_issue_list_json() {
        let app = test_helpers::make_app_json();

        run(
            IssueCommand::List {
                repo: Some("org/repo".into()),
                state: "open".into(),
                limit: 10,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_issue_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            IssueCommand::List {
                repo: Some("org/repo".into()),
                state: "open".into(),
                limit: 10,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_issue_view() {
        let app = test_helpers::make_app();

        run(
            IssueCommand::View {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_issue_view_json() {
        let app = test_helpers::make_app_json();

        run(
            IssueCommand::View {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_issue_view_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            IssueCommand::View {
                number: 1,
                repo: Some("org/repo".into()),
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
