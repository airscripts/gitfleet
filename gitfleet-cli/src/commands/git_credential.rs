use std::io::{self, BufRead, Write};

use gitfleet_core::errors::GitfleetError;

#[derive(clap::Subcommand, Debug)]
pub enum GitCredentialCommand {
    #[command(about = "Provide credentials to git (get).")]
    Get,

    #[command(about = "Store credentials (no-op, gitfleet is source of truth).")]
    Store,

    #[command(about = "Erase credentials (no-op).")]
    Erase,
}

pub async fn run(cmd: GitCredentialCommand) -> Result<(), GitfleetError> {
    match cmd {
        GitCredentialCommand::Get => {
            let attrs = read_credential_attrs()?;

            let _host = attrs.get("host").cloned().unwrap_or_default();

            let token = gitfleet_core::config::get_token_optional();

            if let Some(token) = token {
                let username = attrs
                    .get("username")
                    .cloned()
                    .unwrap_or_else(|| "oauth2".to_string());

                let mut stdout = io::stdout().lock();
                writeln!(stdout, "username={username}").ok();
                writeln!(stdout, "password={token}").ok();
                writeln!(stdout).ok();
            }

            Ok(())
        }

        GitCredentialCommand::Store | GitCredentialCommand::Erase => {
            drain_stdin();

            Ok(())
        }
    }
}

fn read_credential_attrs() -> Result<std::collections::HashMap<String, String>, GitfleetError> {
    let stdin = io::stdin();

    let mut attrs = std::collections::HashMap::new();

    for line in stdin.lock().lines() {
        let line = line.map_err(|e| GitfleetError::new(format!("Failed to read stdin: {e}")))?;

        if line.is_empty() {
            break;
        }

        if let Some((key, value)) = line.split_once('=') {
            attrs.insert(key.to_string(), value.to_string());
        }
    }

    Ok(attrs)
}

fn drain_stdin() {
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        if line.map(|l| l.is_empty()).unwrap_or(true) {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_credential_attrs_empty() {
        let attrs: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        assert!(attrs.is_empty());
    }

    #[test]
    fn test_git_credential_command_variants() {
        let get = GitCredentialCommand::Get;
        let store = GitCredentialCommand::Store;
        let erase = GitCredentialCommand::Erase;

        assert!(matches!(get, GitCredentialCommand::Get));

        assert!(matches!(store, GitCredentialCommand::Store));
        assert!(matches!(erase, GitCredentialCommand::Erase));
    }
}
