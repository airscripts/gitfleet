use clap::{CommandFactory, Subcommand};
use gitfleet_core::errors::{GitfleetError, UnprocessableError};

use crate::app::App;

#[derive(Subcommand, Debug)]
pub enum CompletionCommand {
    #[command(about = "Generate shell completions.")]
    Generate {
        #[arg(value_name = "SHELL")]
        shell: String,
    },

    #[command(about = "Generate man pages.")]
    Mangen {
        #[arg(value_name = "DIR")]
        dir: String,
    },
}

pub async fn run(cmd: CompletionCommand, app: &App) -> Result<(), GitfleetError> {
    match cmd {
        CompletionCommand::Generate { shell } => {
            let shell: clap_complete::Shell = shell.parse().map_err(|_| {
                GitfleetError::from(UnprocessableError::new(
                    "Unsupported shell. Use: bash, elvish, fish, powershell, zsh, or nushell",
                ))
            })?;

            if app.renderer().is_json() {
                let mut buf = Vec::new();

                let mut cli = crate::Cli::command();
                clap_complete::generate(shell, &mut cli, "gitfleet", &mut buf);

                let result = serde_json::json!({
                    "generated": true,
                    "shell": shell.to_string(),
                });

                app.renderer().write_result(&result);
            } else {
                let mut cli = crate::Cli::command();
                clap_complete::generate(shell, &mut cli, "gitfleet", &mut std::io::stdout());
            }

            Ok(())
        }

        CompletionCommand::Mangen { dir } => {
            let cli = crate::Cli::command();

            let man = clap_mangen::Man::new(cli);
            let mut output = Vec::new();
            man.render(&mut output)
                .map_err(|e| GitfleetError::new(format!("Failed to generate man page: {e}")))?;

            std::fs::create_dir_all(&dir)
                .map_err(|e| GitfleetError::new(format!("Failed to create directory: {e}")))?;

            let path = std::path::Path::new(&dir).join("gitfleet.1");
            std::fs::write(&path, &output)
                .map_err(|e| GitfleetError::new(format!("Failed to write man page: {e}")))?;

            if app.renderer().is_json() {
                let result = serde_json::json!({"generated": true, "path": path.to_string_lossy().to_string()});

                app.renderer().write_result(&result);
            } else {
                app.renderer()
                    .write_value(&format!("Man page written to {}", path.to_string_lossy()));
            }

            Ok(())
        }
    }
}
