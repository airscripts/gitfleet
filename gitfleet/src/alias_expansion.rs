use std::collections::HashSet;
use std::ffi::OsString;

const MAX_ALIAS_DEPTH: usize = 16;

pub fn expand_args<F, C>(
    args: Vec<OsString>,
    get_alias: F,
    is_canonical_command: C,
) -> Result<Vec<OsString>, String>
where
    F: Fn(&str) -> Option<String>,
    C: Fn(&str) -> bool,
{
    let Some(command_index) = command_index(&args) else {
        return Ok(args);
    };

    let Some(command) = args[command_index].to_str() else {
        return Ok(args);
    };

    if is_canonical_command(command) {
        return Ok(args);
    }

    let mut expanded = args;
    let mut seen = HashSet::new();

    for _ in 0..MAX_ALIAS_DEPTH {
        let Some(alias) = expanded[command_index].to_str() else {
            return Ok(expanded);
        };

        if is_canonical_command(alias) {
            return Ok(expanded);
        }

        let Some(value) = get_alias(alias) else {
            return Ok(expanded);
        };

        if !seen.insert(alias.to_string()) {
            return Err(format!("Alias cycle detected while expanding '{alias}'."));
        }

        let replacement = split_expansion(&value)?;

        if replacement.is_empty() {
            return Err(format!("Alias '{alias}' has an empty expansion."));
        }

        expanded.splice(
            command_index..=command_index,
            replacement.into_iter().map(OsString::from),
        );
    }

    Err(format!(
        "Alias expansion exceeded the maximum depth of {MAX_ALIAS_DEPTH}."
    ))
}

fn command_index(args: &[OsString]) -> Option<usize> {
    let mut index = 1;

    while index < args.len() {
        let argument = args[index].to_str()?;

        match argument {
            "--json" | "--debug" | "--yes" | "--dry-run" => index += 1,
            "--theme" => index += 2,
            value if value.starts_with("--theme=") => index += 1,
            value if value.starts_with('-') => return None,
            _ => return Some(index),
        }
    }

    None
}

fn split_expansion(value: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut started = false;

    for character in value.chars() {
        if escaped {
            word.push(character);
            escaped = false;
            started = true;

            continue;
        }

        if character == '\\' && quote != Some('\'') {
            escaped = true;
            started = true;

            continue;
        }

        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            } else {
                word.push(character);
            }

            started = true;

            continue;
        }

        match character {
            '\'' | '"' => {
                quote = Some(character);
                started = true;
            }

            character if character.is_whitespace() => {
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
            }

            _ => {
                word.push(character);
                started = true;
            }
        }
    }

    if escaped {
        return Err("Alias expansion ends with an incomplete escape.".to_string());
    }

    if quote.is_some() {
        return Err("Alias expansion contains an unterminated quote.".to_string());
    }

    if started {
        words.push(word);
    }

    Ok(words)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn expands_alias_and_forwards_arguments() {
        let expanded = expand_args(
            args(&["gitfleet", "prs", "--limit", "5"]),
            |name| (name == "prs").then(|| "change list".to_string()),
            |name| matches!(name, "change" | "repo"),
        )
        .unwrap();

        assert_eq!(
            expanded,
            args(&["gitfleet", "change", "list", "--limit", "5"])
        );
    }

    #[test]
    fn expands_alias_after_global_options() {
        let expanded = expand_args(
            args(&["gitfleet", "--json", "--theme", "plain", "repos"]),
            |name| (name == "repos").then(|| "repo list".to_string()),
            |name| name == "repo",
        )
        .unwrap();

        assert_eq!(
            expanded,
            args(&["gitfleet", "--json", "--theme", "plain", "repo", "list"])
        );
    }

    #[test]
    fn expands_nested_aliases() {
        let expanded = expand_args(
            args(&["gitfleet", "mine"]),
            |name| match name {
                "mine" => Some("repos --owner me".to_string()),
                "repos" => Some("repo list".to_string()),
                _ => None,
            },
            |name| name == "repo",
        )
        .unwrap();

        assert_eq!(
            expanded,
            args(&["gitfleet", "repo", "list", "--owner", "me"])
        );
    }

    #[test]
    fn preserves_quoted_values_without_running_a_shell() {
        let expanded = expand_args(
            args(&["gitfleet", "find"]),
            |name| (name == "find").then(|| "search repos 'fleet tools'".to_string()),
            |name| name == "search",
        )
        .unwrap();

        assert_eq!(
            expanded,
            args(&["gitfleet", "search", "repos", "fleet tools"])
        );
    }

    #[test]
    fn detects_alias_cycles() {
        let error = expand_args(
            args(&["gitfleet", "one"]),
            |name| match name {
                "one" => Some("two".to_string()),
                "two" => Some("one".to_string()),
                _ => None,
            },
            |_| false,
        )
        .unwrap_err();

        assert!(error.contains("cycle"));
    }

    #[test]
    fn rejects_malformed_expansions() {
        let error = expand_args(
            args(&["gitfleet", "broken"]),
            |_| Some("repo view 'unterminated".to_string()),
            |name| name == "repo",
        )
        .unwrap_err();

        assert!(error.contains("unterminated quote"));
    }

    #[test]
    fn does_not_expand_canonical_commands() {
        let original = args(&["gitfleet", "repo", "list"]);
        let expanded = expand_args(
            original.clone(),
            |_| Some("change list".to_string()),
            |name| name == "repo",
        )
        .unwrap();

        assert_eq!(expanded, original);
    }
}
