fn is_unreserved(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~' | ':')
}

pub fn encode_segment(s: &str) -> String {
    let mut encoded = String::new();

    for byte in s.bytes() {
        let character = byte as char;
        if is_unreserved(character) {
            encoded.push(character);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }

    encoded
}

pub fn encode_repo(repo: &str) -> String {
    repo.split('/')
        .map(encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

pub fn encode_path(path: &str) -> String {
    path.split('/')
        .map(|part| {
            if matches!(part, "." | "..") {
                part.bytes().map(|byte| format!("%{byte:02X}")).collect()
            } else {
                encode_segment(part)
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub fn repo_path(repo: &str, parts: &[&str]) -> String {
    let mut path = format!("/repos/{}", encode_repo(repo));

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

    #[test]
    fn test_repo_path_encodes_reserved_and_unicode_bytes() {
        assert_eq!(
            repo_path("org/repo?name", &["issues", "café"]),
            "/repos/org/repo%3Fname/issues/caf%C3%A9"
        );
    }

    #[test]
    fn test_encode_path_preserves_separators_and_escapes_dot_segments() {
        assert_eq!(encode_path("src/main.rs"), "src/main.rs");
        assert_eq!(encode_path("src/../secret"), "src/%2E%2E/secret");
    }
}
