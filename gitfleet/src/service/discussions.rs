use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    owner: &str,
    name: &str,
    category_id: Option<&str>,
    limit: u32,
) -> Result<(), GitfleetError> {
    let ops = provider.discussion_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Discussions,
        ))
    })?;

    let discussions = ops
        .list_discussions(owner, name, category_id, limit)
        .await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&discussions)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize discussions: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = discussions
            .iter()
            .map(|d| {
                serde_json::json!({
                    "NUMBER": d.number,
                    "TITLE": d.title,
                    "STATE": d.closed,
                    "AUTHOR": d.author,
                    "CATEGORY": d.category,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No discussions found."),
            Some("Discussions"),
            Some(&["NUMBER", "TITLE", "STATE", "AUTHOR", "CATEGORY"]),
        );
    }

    Ok(())
}
