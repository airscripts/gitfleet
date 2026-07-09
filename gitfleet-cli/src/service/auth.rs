use gitfleet_core::errors::GitfleetError;
use gitfleet_core::output::Renderer;
use gitfleet_core::types::AuthStatus;

pub async fn get_authenticated_user(
    status: &AuthStatus,
    renderer: &Renderer,
) -> Result<(), GitfleetError> {
    if renderer.is_json() {
        let json = serde_json::to_value(status)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize auth status: {e}")))?;

        renderer.write_result(&json);
    } else {
        renderer.render_summary(
            "Authentication",
            &[
                ("User", status.user.login.clone()),
                (
                    "Name",
                    status
                        .user
                        .name
                        .as_deref()
                        .unwrap_or("(not set)")
                        .to_string(),
                ),
                ("URL", status.user.html_url.clone()),
                ("Scopes", status.scopes.join(", ")),
            ],
        );
    }

    Ok(())
}
