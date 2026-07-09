fn is_unreserved(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~')
}

fn encode_segment(s: &str) -> String {
    s.split('/')
        .map(|seg| {
            seg.chars()
                .map(|c| {
                    if is_unreserved(c)
                        || matches!(
                            c,
                            ':' | '!'
                                | '$'
                                | '&'
                                | '\''
                                | '('
                                | ')'
                                | '*'
                                | '+'
                                | ','
                                | ';'
                                | '='
                                | '@'
                        )
                    {
                        c.to_string()
                    } else {
                        format!("%{:02X}", c as u8)
                    }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub fn repo_path(repo: &str, parts: &[&str]) -> String {
    let mut path = format!("/repos/{repo}");

    for part in parts {
        path.push_str(&format!("/{}", encode_segment(part)));
    }

    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_path_no_parts() {
        assert_eq!(repo_path("owner/repo", &[]), "/repos/owner/repo");
    }

    #[test]
    fn test_repo_path_single_part() {
        assert_eq!(
            repo_path("owner/repo", &["issues"]),
            "/repos/owner/repo/issues"
        );
    }

    #[test]
    fn test_repo_path_multiple_parts() {
        assert_eq!(
            repo_path("owner/repo", &["issues", "42", "comments"]),
            "/repos/owner/repo/issues/42/comments"
        );
    }

    #[test]
    fn test_repo_path_with_number() {
        assert_eq!(
            repo_path("org/proj", &["actions", "runs", "123"]),
            "/repos/org/proj/actions/runs/123"
        );
    }
}
