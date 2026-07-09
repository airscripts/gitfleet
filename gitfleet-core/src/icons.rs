pub struct Icons;

impl Icons {
    pub fn repo() -> &'static str {
        "\u{25a9}"
    }

    pub fn issue() -> &'static str {
        "\u{25cb}"
    }

    pub fn issue_closed() -> &'static str {
        "\u{25cf}"
    }

    pub fn pull_request() -> &'static str {
        "\u{2192}"
    }

    pub fn pull_merged() -> &'static str {
        "\u{2261}"
    }

    pub fn pipeline() -> &'static str {
        "\u{25c7}"
    }

    pub fn release() -> &'static str {
        "\u{25c6}"
    }

    pub fn discussion() -> &'static str {
        "\u{25b8}"
    }

    pub fn webhook() -> &'static str {
        "\u{26a1}"
    }

    pub fn label() -> &'static str {
        "\u{2588}"
    }

    pub fn secret() -> &'static str {
        "\u{2022}"
    }

    pub fn variable() -> &'static str {
        "\u{03d5}"
    }

    pub fn runner() -> &'static str {
        "\u{25b7}"
    }

    pub fn deployment() -> &'static str {
        "\u{25c9}"
    }

    pub fn environment() -> &'static str {
        "\u{25c8}"
    }

    pub fn wiki() -> &'static str {
        "\u{201c}"
    }

    pub fn notification() -> &'static str {
        "\u{2709}"
    }

    pub fn search() -> &'static str {
        "\u{2315}"
    }

    pub fn package() -> &'static str {
        "\u{25a3}"
    }

    pub fn project() -> &'static str {
        "\u{25a4}"
    }

    pub fn milestone() -> &'static str {
        "\u{25d0}"
    }

    pub fn snippet() -> &'static str {
        "\u{2702}"
    }

    pub fn template() -> &'static str {
        "\u{2260}"
    }

    pub fn generic() -> &'static str {
        "\u{2022}"
    }

    pub fn check() -> &'static str {
        "\u{2713}"
    }

    pub fn cross() -> &'static str {
        "\u{2717}"
    }

    pub fn warning() -> &'static str {
        "\u{26a0}"
    }

    pub fn info() -> &'static str {
        "\u{2139}"
    }

    pub fn arrow_right() -> &'static str {
        "\u{2192}"
    }

    pub fn dot() -> &'static str {
        "\u{00b7}"
    }

    pub fn state_open(state: &str) -> &'static str {
        if state.eq_ignore_ascii_case("closed") {
            Self::issue_closed()
        } else {
            Self::issue()
        }
    }

    pub fn pr_state(state: &str) -> &'static str {
        match state.to_lowercase().as_str() {
            "merged" => Self::pull_merged(),
            "closed" => Self::cross(),
            _ => Self::pull_request(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icons_non_empty() {
        assert!(!Icons::repo().is_empty());

        assert!(!Icons::issue().is_empty());
        assert!(!Icons::pull_request().is_empty());

        assert!(!Icons::pipeline().is_empty());
        assert!(!Icons::release().is_empty());

        assert!(!Icons::check().is_empty());
        assert!(!Icons::cross().is_empty());

        assert!(!Icons::warning().is_empty());
        assert!(!Icons::info().is_empty());
    }

    #[test]
    fn test_state_open() {
        assert_eq!(Icons::state_open("open"), Icons::issue());

        assert_eq!(Icons::state_open("closed"), Icons::issue_closed());
        assert_eq!(Icons::state_open("OPEN"), Icons::issue());
    }

    #[test]
    fn test_pr_state() {
        assert_eq!(Icons::pr_state("open"), Icons::pull_request());

        assert_eq!(Icons::pr_state("merged"), Icons::pull_merged());
        assert_eq!(Icons::pr_state("closed"), Icons::cross());
    }
}
