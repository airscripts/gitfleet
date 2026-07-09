use crate::app::App;

pub fn run(app: &App) {
    if app.renderer().is_json() {
        app.renderer()
            .write_result(&serde_json::json!({"version": env!("CARGO_PKG_VERSION")}));
    } else {
        app.renderer().write_value(env!("CARGO_PKG_VERSION"));
    }
}
