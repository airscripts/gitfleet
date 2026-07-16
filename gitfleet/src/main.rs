mod alias_expansion;
mod app;
mod commands;
mod repo_util;
pub mod service;

use std::sync::Arc;

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};

use crate::app::App;

#[derive(Parser)]
#[command(name = "gitfleet")]
#[command(bin_name = "gitfleet")]
#[command(about = "Command every repository as one fleet.")]
#[command(version)]
pub struct Cli {
    #[arg(long, global = true)]
    json: bool,

    #[arg(long, global = true)]
    debug: bool,

    #[arg(long, global = true, default_value = "auto")]
    theme: String,

    #[arg(long, global = true)]
    yes: bool,

    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Organization access management")]
    Access {
        #[command(subcommand)]
        subcommand: commands::access::AccessCommand,
    },

    #[command(about = "Security advisory operations")]
    Advisory {
        #[command(subcommand)]
        subcommand: commands::advisory::AdvisoryCommand,
    },

    #[command(about = "Command aliases")]
    Alias {
        #[command(subcommand)]
        subcommand: commands::alias::AliasCommand,
    },

    #[command(about = "Repository and pipeline analytics")]
    Analytics {
        #[command(subcommand)]
        subcommand: commands::analytics::AnalyticsCommand,
    },

    #[command(about = "Raw provider API access")]
    Api {
        #[command(subcommand)]
        subcommand: commands::api::ApiCommand,
    },

    #[command(about = "Attestation operations")]
    Attestation {
        #[command(subcommand)]
        subcommand: commands::attestation::AttestationCommand,
    },

    #[command(about = "Provider authentication and profiles")]
    Auth {
        #[command(subcommand)]
        subcommand: commands::auth::AuthCommand,
    },

    #[command(about = "Browse repository resources")]
    Browse {
        #[command(subcommand)]
        subcommand: commands::browse::BrowseCommand,
    },

    #[command(about = "Proposed changes and merge operations")]
    Change {
        #[command(subcommand)]
        subcommand: commands::change::ChangeCommand,
    },

    #[command(about = "Code navigation and search")]
    Code {
        #[command(subcommand)]
        subcommand: commands::code::CodeCommand,
    },

    #[command(about = "Shell completion generation")]
    Completion {
        #[command(subcommand)]
        subcommand: commands::completion::CompletionCommand,
    },

    #[command(about = "Configuration management")]
    Config {
        #[command(subcommand)]
        subcommand: commands::config::ConfigCommand,
    },

    #[command(name = "deps", about = "Dependency review and analysis")]
    Deps {
        #[command(subcommand)]
        subcommand: commands::dependency::DependencyCommand,
    },

    #[command(about = "Deployment management")]
    Deploy {
        #[command(subcommand)]
        subcommand: commands::deploy::DeployCommand,
    },

    #[command(about = "Development environments")]
    Dev {
        #[command(subcommand)]
        subcommand: commands::dev::DevCommand,
    },

    #[command(about = "Discussion management")]
    Discussion {
        #[command(subcommand)]
        subcommand: commands::discussion::DiscussionCommand,
    },

    #[command(about = "Environment management")]
    Environment {
        #[command(subcommand)]
        subcommand: commands::environment::EnvironmentCommand,
    },

    #[command(hide = true, name = "git-credential")]
    GitCredential {
        #[command(subcommand)]
        subcommand: commands::git_credential::GitCredentialCommand,
    },

    #[command(about = "Repository ruleset governance")]
    Govern {
        #[command(subcommand)]
        subcommand: commands::govern::GovernCommand,
    },

    #[command(about = "Account keys (SSH, GPG)")]
    Identity {
        #[command(subcommand)]
        subcommand: commands::identity::IdentityCommand,
    },

    #[command(about = "Notification management")]
    Inbox {
        #[command(subcommand)]
        subcommand: commands::inbox::InboxCommand,
    },

    #[command(about = "Issue management")]
    Issue {
        #[command(subcommand)]
        subcommand: commands::issue::IssueCommand,
    },

    #[command(about = "Label management")]
    Label {
        #[command(subcommand)]
        subcommand: commands::label_cmd::LabelCmdCommand,
    },

    #[command(about = "License discovery")]
    License {
        #[command(subcommand)]
        subcommand: commands::license::LicenseCommand,
    },

    #[command(about = "Planning boards, milestones, and projects")]
    Planning {
        #[command(subcommand)]
        subcommand: PlanningCommand,
    },

    #[command(about = "CI/CD pipeline definitions and runs")]
    Pipeline {
        #[command(subcommand)]
        subcommand: commands::pipeline::PipelineCommand,
    },

    #[command(about = "Repository protection policies")]
    Policy {
        #[command(subcommand)]
        subcommand: commands::policy::PolicyCommand,
    },

    #[command(about = "Package registry")]
    Registry {
        #[command(subcommand)]
        subcommand: commands::package::PackageCommand,
    },

    #[command(about = "Release management")]
    Release {
        #[command(subcommand)]
        subcommand: commands::release::ReleaseCommand,
    },

    #[command(about = "Repository lifecycle and forks")]
    Repo {
        #[command(subcommand)]
        subcommand: commands::repo::RepoCommand,
    },

    #[command(about = "Reviews, comments, and reactions")]
    Review {
        #[command(subcommand)]
        subcommand: ReviewCommand,
    },

    #[command(about = "CI/CD runner management")]
    Runner {
        #[command(subcommand)]
        subcommand: commands::runner::RunnerCommand,
    },

    #[command(about = "Search repositories and code")]
    Search {
        #[command(subcommand)]
        subcommand: commands::search::SearchCommand,
    },

    #[command(about = "Repository secrets")]
    Secret {
        #[command(subcommand)]
        subcommand: commands::secret_cmd::SecretCmdCommand,
    },

    #[command(about = "Security advisories and scanning")]
    Security {
        #[command(subcommand)]
        subcommand: commands::security::SecurityCommand,
    },

    #[command(about = "Repository pages")]
    Site {
        #[command(subcommand)]
        subcommand: commands::site::SiteCommand,
    },

    #[command(about = "Hosted snippets")]
    Snippet {
        #[command(subcommand)]
        subcommand: commands::snippet::SnippetCommand,
    },

    #[command(about = "Repository templates")]
    Template {
        #[command(subcommand)]
        subcommand: commands::template::TemplateCommand,
    },

    #[command(about = "Repository variables")]
    Variable {
        #[command(subcommand)]
        subcommand: commands::variable::VariableCommand,
    },

    #[command(name = "version", about = "Version information")]
    Version,

    #[command(about = "Webhook management")]
    Webhook {
        #[command(subcommand)]
        subcommand: commands::webhook::WebhookCommand,
    },

    #[command(about = "Repository wiki pages")]
    Wiki {
        #[command(subcommand)]
        subcommand: commands::wiki::WikiCommand,
    },

    #[command(about = "Named fleets and multi-repository execution")]
    Workspace {
        #[command(subcommand)]
        subcommand: commands::workspace::WorkspaceCommand,
    },
}

#[derive(Subcommand)]
enum PlanningCommand {
    #[command(about = "Manage milestones.")]
    Milestone {
        #[command(subcommand)]
        subcommand: commands::milestone_cmd::MilestoneCmdCommand,
    },

    #[command(about = "Manage projects.")]
    Project {
        #[command(subcommand)]
        subcommand: commands::project_cmd::ProjectCmdCommand,
    },
}

#[derive(Subcommand)]
enum ReviewCommand {
    #[command(about = "Manage comments.")]
    Comment {
        #[command(subcommand)]
        subcommand: commands::comment_cmd::CommentCmdCommand,
    },

    #[command(about = "Manage reactions.")]
    Reaction {
        #[command(subcommand)]
        subcommand: commands::reaction_cmd::ReactionCmdCommand,
    },
}

fn requires_provider_context(command: &Commands) -> bool {
    !matches!(
        command,
        Commands::Alias { .. }
            | Commands::Completion { .. }
            | Commands::Config { .. }
            | Commands::GitCredential { .. }
            | Commands::Version
            | Commands::Auth {
                subcommand: commands::auth::AuthCommand::Login { .. }
                    | commands::auth::AuthCommand::Logout { .. }
            }
    )
}

#[tokio::main]
async fn main() {
    let banner = gitfleet_core::banner::banner();

    let cmd = Cli::command().before_help(&banner);
    let canonical_commands: std::collections::HashSet<String> = cmd
        .get_subcommands()
        .map(|subcommand| subcommand.get_name().to_string())
        .collect();

    let args = alias_expansion::expand_args(
        std::env::args_os().collect(),
        gitfleet_core::config::get_alias,
        |name| canonical_commands.contains(name),
    )
    .unwrap_or_else(|error| {
        cmd.clone()
            .error(clap::error::ErrorKind::InvalidValue, error)
            .exit()
    });

    let matches = cmd.get_matches_from(args);

    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    gitfleet_core::logger::init(cli.debug);

    let mode = if cli.json {
        gitfleet_core::output_state::OutputMode::Json
    } else {
        gitfleet_core::output_state::OutputMode::Human
    };

    let theme = gitfleet_core::theme::Theme::parse(&cli.theme);

    let renderer = gitfleet_core::output::Renderer::new(mode)
        .with_theme(theme)
        .with_yes(cli.yes);

    let app = if requires_provider_context(&cli.command) {
        match App::from_config(renderer, cli.dry_run) {
            Ok(app) => Arc::new(app),
            Err(e) => {
                let mode = if cli.json {
                    gitfleet_core::output_state::OutputMode::Json
                } else {
                    gitfleet_core::output_state::OutputMode::Human
                };

                let theme = gitfleet_core::theme::Theme::parse(&cli.theme);

                let fallback_renderer = gitfleet_core::output::Renderer::new(mode)
                    .with_theme(theme)
                    .with_yes(cli.yes);
                fallback_renderer.write_error_for(&e);
                std::process::exit(1);
            }
        }
    } else {
        Arc::new(App::without_config(renderer, cli.dry_run))
    };

    let result = match cli.command {
        Commands::Access { subcommand } => commands::access::run(subcommand, &app).await,
        Commands::Advisory { subcommand } => commands::advisory::run(subcommand, &app).await,
        Commands::Alias { subcommand } => commands::alias::run(subcommand, &app).await,
        Commands::Analytics { subcommand } => commands::analytics::run(subcommand, &app).await,
        Commands::Api { subcommand } => commands::api::run(subcommand, &app).await,
        Commands::Attestation { subcommand } => commands::attestation::run(subcommand, &app).await,
        Commands::Auth { subcommand } => commands::auth::run(subcommand, &app).await,
        Commands::Browse { subcommand } => commands::browse::run(subcommand, &app).await,
        Commands::Change { subcommand } => commands::change::run(subcommand, &app).await,
        Commands::Code { subcommand } => commands::code::run(subcommand, &app).await,
        Commands::Completion { subcommand } => commands::completion::run(subcommand, &app).await,
        Commands::Config { subcommand } => commands::config::run(subcommand, &app).await,
        Commands::Deps { subcommand } => commands::dependency::run(subcommand, &app).await,
        Commands::Deploy { subcommand } => commands::deploy::run(subcommand, &app).await,
        Commands::Dev { subcommand } => commands::dev::run(subcommand, &app).await,
        Commands::Discussion { subcommand } => commands::discussion::run(subcommand, &app).await,
        Commands::Environment { subcommand } => commands::environment::run(subcommand, &app).await,
        Commands::GitCredential { subcommand } => commands::git_credential::run(subcommand).await,
        Commands::Govern { subcommand } => commands::govern::run(subcommand, &app).await,
        Commands::Identity { subcommand } => commands::identity::run(subcommand, &app).await,
        Commands::Inbox { subcommand } => commands::inbox::run(subcommand, &app).await,
        Commands::Issue { subcommand } => commands::issue::run(subcommand, &app).await,
        Commands::Label { subcommand } => commands::label_cmd::run(subcommand, &app).await,
        Commands::License { subcommand } => commands::license::run(subcommand, &app).await,
        Commands::Pipeline { subcommand } => commands::pipeline::run(subcommand, &app).await,
        Commands::Policy { subcommand } => commands::policy::run(subcommand, &app).await,
        Commands::Planning { subcommand } => match subcommand {
            PlanningCommand::Milestone { subcommand } => {
                commands::milestone_cmd::run(subcommand, &app).await
            }

            PlanningCommand::Project { subcommand } => {
                commands::project_cmd::run(subcommand, &app).await
            }
        },

        Commands::Registry { subcommand } => commands::package::run(subcommand, &app).await,
        Commands::Release { subcommand } => commands::release::run(subcommand, &app).await,
        Commands::Repo { subcommand } => commands::repo::run(subcommand, &app).await,
        Commands::Review { subcommand } => match subcommand {
            ReviewCommand::Comment { subcommand } => {
                commands::comment_cmd::run(subcommand, &app).await
            }

            ReviewCommand::Reaction { subcommand } => {
                commands::reaction_cmd::run(subcommand, &app).await
            }
        },

        Commands::Runner { subcommand } => commands::runner::run(subcommand, &app).await,
        Commands::Search { subcommand } => commands::search::run(subcommand, &app).await,
        Commands::Secret { subcommand } => commands::secret_cmd::run(subcommand, &app).await,
        Commands::Security { subcommand } => commands::security::run(subcommand, &app).await,
        Commands::Site { subcommand } => commands::site::run(subcommand, &app).await,
        Commands::Snippet { subcommand } => commands::snippet::run(subcommand, &app).await,
        Commands::Template { subcommand } => commands::template::run(subcommand, &app).await,
        Commands::Variable { subcommand } => commands::variable::run(subcommand, &app).await,
        Commands::Version => {
            commands::version::run(&app);

            Ok(())
        }

        Commands::Webhook { subcommand } => commands::webhook::run(subcommand, &app).await,
        Commands::Wiki { subcommand } => commands::wiki::run(subcommand, &app).await,
        Commands::Workspace { subcommand } => commands::workspace::run(subcommand, &app).await,
    };

    if let Err(e) = result {
        app.renderer().write_error_for(&e);
        std::process::exit(1);
    }
}
