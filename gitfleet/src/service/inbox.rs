use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    all: bool,
    participating: bool,
    repo: Option<&str>,
) -> Result<(), GitfleetError> {
    let ops = provider.notification_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Notifications,
        ))
    })?;

    let notifications = ops.list_notifications(all, participating, repo).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&notifications)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize notifications: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = notifications
            .iter()
            .map(|n| {
                serde_json::json!({
                    "TYPE": n.subject_type,
                    "TITLE": n.subject_title,
                    "REPO": n.repository,
                    "REASON": n.reason,
                    "UNREAD": n.unread,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No notifications."),
            Some("Notifications"),
            Some(&["TYPE", "TITLE", "REPO", "REASON", "UNREAD"]),
        );
    }

    Ok(())
}
