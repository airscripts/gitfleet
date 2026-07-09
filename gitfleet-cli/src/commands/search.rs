use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum SearchCommand {
    #[command(about = "Search issues.")]
    Issues {
        query: String,
        #[arg(long)]
        sort: Option<String>,
        #[arg(long)]
        order: Option<String>,
        #[arg(long, default_value = "30")]
        limit: u32,
    },

    #[command(about = "Search repositories.")]
    Repos {
        query: String,
        #[arg(long)]
        sort: Option<String>,
        #[arg(long)]
        order: Option<String>,
        #[arg(long, default_value = "30")]
        limit: u32,
    },

    #[command(about = "Search code.")]
    Code {
        query: String,
        #[arg(long, default_value = "30")]
        limit: u32,
    },
}

pub async fn run(cmd: SearchCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        SearchCommand::Issues {
            query,
            sort,
            order,
            limit,
        } => {
            service::search::search_issues(
                p,
                app.renderer(),
                &query,
                sort.as_deref(),
                order.as_deref(),
                limit,
            )
            .await
        }

        SearchCommand::Repos {
            query,
            sort,
            order,
            limit,
        } => {
            service::search::search_repos(
                p,
                app.renderer(),
                &query,
                sort.as_deref(),
                order.as_deref(),
                limit,
            )
            .await
        }

        SearchCommand::Code { query, limit } => {
            service::search::search_code(p, app.renderer(), &query, limit).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_search_issues() {
        let app = test_helpers::make_app();

        run(
            SearchCommand::Issues {
                query: "bug".into(),
                sort: None,
                order: None,
                limit: 30,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_issues_with_sort_order() {
        let app = test_helpers::make_app();

        run(
            SearchCommand::Issues {
                query: "bug".into(),
                sort: Some("created".into()),
                order: Some("desc".into()),
                limit: 10,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_issues_json() {
        let app = test_helpers::make_app_json();

        run(
            SearchCommand::Issues {
                query: "bug".into(),
                sort: None,
                order: None,
                limit: 30,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_issues_human() {
        let app = test_helpers::make_app_human();

        run(
            SearchCommand::Issues {
                query: "bug".into(),
                sort: None,
                order: None,
                limit: 30,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_issues_default_limit() {
        let app = test_helpers::make_app();

        run(
            SearchCommand::Issues {
                query: "label:bug".into(),
                sort: None,
                order: None,
                limit: 30,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_repos() {
        let app = test_helpers::make_app();

        run(
            SearchCommand::Repos {
                query: "rust".into(),
                sort: None,
                order: None,
                limit: 30,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_repos_with_sort_order() {
        let app = test_helpers::make_app();

        run(
            SearchCommand::Repos {
                query: "rust".into(),
                sort: Some("stars".into()),
                order: Some("asc".into()),
                limit: 5,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_repos_json() {
        let app = test_helpers::make_app_json();

        run(
            SearchCommand::Repos {
                query: "rust".into(),
                sort: None,
                order: None,
                limit: 30,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_repos_human() {
        let app = test_helpers::make_app_human();

        run(
            SearchCommand::Repos {
                query: "rust".into(),
                sort: None,
                order: None,
                limit: 30,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_code() {
        let app = test_helpers::make_app();

        run(
            SearchCommand::Code {
                query: "fn main".into(),
                limit: 30,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_code_custom_limit() {
        let app = test_helpers::make_app();

        run(
            SearchCommand::Code {
                query: "fn main".into(),
                limit: 5,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_code_json() {
        let app = test_helpers::make_app_json();

        run(
            SearchCommand::Code {
                query: "fn main".into(),
                limit: 30,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_code_human() {
        let app = test_helpers::make_app_human();

        run(
            SearchCommand::Code {
                query: "fn main".into(),
                limit: 30,
            },
            &app,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_issues_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SearchCommand::Issues {
                query: "bug".into(),
                sort: None,
                order: None,
                limit: 30,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_repos_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SearchCommand::Repos {
                query: "rust".into(),
                sort: None,
                order: None,
                limit: 30,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_code_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(
            SearchCommand::Code {
                query: "fn main".into(),
                limit: 30,
            },
            &app,
        )
        .await;

        assert!(result.is_err());
    }
}
