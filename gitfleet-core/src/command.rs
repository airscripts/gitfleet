use crate::errors::GitfleetError;
use crate::output::Renderer;

pub async fn run<F, Fut>(renderer: &Renderer, f: F) -> Result<(), GitfleetError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), GitfleetError>>,
{
    match f().await {
        Ok(()) => Ok(()),
        Err(e) => {
            renderer.write_error_for(&e);

            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_state::OutputMode;

    #[tokio::test]
    async fn test_run_ok() {
        let renderer = Renderer::new(OutputMode::Silent);

        let result = run(&renderer, || async { Ok(()) }).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_err() {
        let renderer = Renderer::new(OutputMode::Silent);

        let result = run(&renderer, || async {
            Err(GitfleetError::new("test error"))
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_err_propagates_message() {
        let renderer = Renderer::new(OutputMode::Silent);

        let result = run(&renderer, || async {
            Err(GitfleetError::new("specific error"))
        })
        .await;

        assert_eq!(result.unwrap_err().to_string(), "specific error");
    }

    #[tokio::test]
    async fn test_run_with_json_renderer() {
        let renderer = Renderer::new(OutputMode::Json);

        let result = run(&renderer, || async { Ok(()) }).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_with_human_renderer() {
        let renderer = Renderer::new(OutputMode::Human);

        let result = run(&renderer, || async { Ok(()) }).await;

        assert!(result.is_ok());
    }
}
