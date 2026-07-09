use gitfleet_core::errors::GitfleetError;
use gitfleet_core::output::Renderer;
use gitfleet_core::provider::GitProvider;

pub async fn open(
    _provider: &dyn GitProvider,
    renderer: &Renderer,
    repo: &str,
    path: Option<&str>,
) -> Result<(), GitfleetError> {
    let base = format!("https://github.com/{repo}");

    let url = match path {
        Some(p) => format!("{base}/blob/main/{p}"),
        None => base,
    };

    if renderer.is_json() {
        renderer.write_result(&serde_json::json!({ "url": url }));
    } else {
        renderer.write_value(&url);
    }

    Ok(())
}
