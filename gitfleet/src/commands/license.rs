use clap::Subcommand;
use gitfleet_core::errors::GitfleetError;

use crate::app::App;
use crate::service;

#[derive(Subcommand, Debug)]
pub enum LicenseCommand {
    #[command(about = "List licenses.")]
    List,

    #[command(about = "View a license.")]
    View { key: String },
}

pub async fn run(cmd: LicenseCommand, app: &App) -> Result<(), GitfleetError> {
    let p = app.provider()?;

    match cmd {
        LicenseCommand::List => service::licenses::list(p, app.renderer()).await,
        LicenseCommand::View { key } => service::licenses::view(p, app.renderer(), &key).await,
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers;
    use super::*;

    #[tokio::test]
    async fn test_license_list() {
        let app = test_helpers::make_app();

        run(LicenseCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    async fn test_license_list_json() {
        let app = test_helpers::make_app_json();

        run(LicenseCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    async fn test_license_list_human() {
        let app = test_helpers::make_app_human();

        run(LicenseCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    async fn test_license_list_dry_run() {
        let app = test_helpers::make_app_dry_run();

        run(LicenseCommand::List, &app).await.unwrap();
    }

    #[tokio::test]
    async fn test_license_view() {
        let app = test_helpers::make_app();

        run(LicenseCommand::View { key: "mit".into() }, &app)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_license_view_json() {
        let app = test_helpers::make_app_json();

        run(LicenseCommand::View { key: "mit".into() }, &app)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_license_view_human() {
        let app = test_helpers::make_app_human();

        run(LicenseCommand::View { key: "mit".into() }, &app)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_license_list_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(LicenseCommand::List, &app).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_license_view_no_caps() {
        let app = test_helpers::make_app_no_caps();

        let result = run(LicenseCommand::View { key: "mit".into() }, &app).await;

        assert!(result.is_err());
    }
}
