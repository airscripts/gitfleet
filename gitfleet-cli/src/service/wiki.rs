use gitfleet_core::errors::{GitfleetError, UnsupportedCapabilityError};
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::{GitProvider, ProviderCapability};

pub async fn list_pages(
    provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
) -> Result<(), GitfleetError> {
    let ops = provider.wiki_ops().ok_or_else(|| {
        GitfleetError::from(UnsupportedCapabilityError::new(
            provider.id(),
            ProviderCapability::Wiki,
        ))
    })?;

    let pages = ops.list_wiki_pages(repo).await?;

    if renderer.is_json() {
        let json = serde_json::to_value(&pages)
            .map_err(|e| GitfleetError::new(format!("Failed to serialize wiki pages: {e}")))?;

        renderer.write_result(&json);
    } else {
        let rows: Vec<serde_json::Value> = pages
            .iter()
            .map(|p| {
                serde_json::json!({
                    "TITLE": p.title,
                    "PATH": p.path,
                    "FORMAT": p.format,
                })
            })
            .collect();

        renderer.render_table_titled(
            &rows,
            Some("No wiki pages found."),
            Some("Wiki Pages"),
            Some(&["TITLE", "PATH", "FORMAT"]),
        );
    }

    Ok(())
}
