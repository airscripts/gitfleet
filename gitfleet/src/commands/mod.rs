pub mod access;
pub mod advisory;
pub mod alias;
pub mod analytics;
pub mod api;
pub mod attestation;
pub mod auth;
pub mod browse;
pub mod change;
pub mod code;
pub mod comment_cmd;
pub mod completion;
pub mod config;
pub mod dependency;
pub mod deploy;
pub mod dev;
pub mod discussion;
pub mod environment;
pub mod fork_cmd;
pub mod git_credential;
pub mod govern;
pub mod identity;
pub mod inbox;
pub mod issue;
pub mod label_cmd;
pub mod license;
pub mod milestone_cmd;
pub mod package;
pub mod pipeline;
pub mod policy;
pub mod project_cmd;
pub mod reaction_cmd;
pub mod release;
pub mod repo;
pub mod runner;
pub mod search;
pub mod secret_cmd;
pub mod security;
pub mod site;
pub mod snippet;
pub mod template;
pub mod variable;
pub mod version;
pub mod webhook;
pub mod wiki;
pub mod workspace;

pub(crate) fn validate_page(page: Option<u32>) -> Result<(), gitfleet_core::errors::GitfleetError> {
    if page == Some(0) {
        return Err(gitfleet_core::errors::GitfleetError::from(
            gitfleet_core::errors::UnprocessableError::new("--page must be greater than 0."),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_validate_page_rejects_zero() {
        let result = super::validate_page(Some(0));

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_page_accepts_none_and_positive_values() {
        assert!(super::validate_page(None).is_ok());
        assert!(super::validate_page(Some(1)).is_ok());
    }
}

#[cfg(test)]
pub(crate) mod test_helpers;
