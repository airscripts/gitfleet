use gitfleet_core::errors::{GitfleetError, UnprocessableError};

pub fn resolve_repo(repo: &Option<String>) -> Result<String, GitfleetError> {
    match repo {
        Some(r) => Ok(r.clone()),
        None => {
            let remote = gitfleet_core::git::get_remote_url(None)?;

            let parsed = gitfleet_core::repository::repository_ref_from_remote(&remote)?;
            Ok(parsed.full_name())
        }
    }
}

pub fn split_repo(repo: &str) -> Result<(&str, &str), GitfleetError> {
    repo.split_once('/').ok_or_else(|| {
        GitfleetError::from(UnprocessableError::new(
            "Repository must be in owner/repo format",
        ))
    })
}
