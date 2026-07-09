use gitfleet_core::output::Renderer;
use gitfleet_core::output_state::OutputMode;

#[test]
fn test_renderer_json_mode_writes_result() {
    let renderer = Renderer::new(OutputMode::Json);

    assert!(renderer.is_json());

    assert!(!renderer.is_human());
    assert!(!renderer.is_silent());

    let val = serde_json::json!({"key": "value", "number": 42});

    renderer.write_result(&val);
}

#[test]
fn test_renderer_human_mode_writes_value() {
    let renderer = Renderer::new(OutputMode::Human);

    assert!(renderer.is_human());

    assert!(!renderer.is_json());
    assert!(!renderer.is_silent());

    renderer.write_value("hello world");
}

#[test]
fn test_renderer_silent_mode_suppresses_output() {
    let renderer = Renderer::new(OutputMode::Silent);

    assert!(renderer.is_silent());

    renderer.write_value("should not appear");

    renderer.write_result(&serde_json::json!({"test": true}));
    renderer.write_error("should not appear", None);
}

#[test]
fn test_renderer_with_yes_flag() {
    let renderer = Renderer::new(OutputMode::Human).with_yes(true);

    assert!(renderer.yes());

    let renderer_no = Renderer::new(OutputMode::Human).with_yes(false);

    assert!(!renderer_no.yes());
}

#[test]
fn test_renderer_render_table_with_data() {
    let renderer = Renderer::new(OutputMode::Human);

    let rows = vec![
        serde_json::json!({"name": "repo1", "private": false, "stars": 100}),
        serde_json::json!({"name": "repo2", "private": true, "stars": 50}),
    ];
    renderer.render_table(&rows, Some("No repositories found."));
}

#[test]
fn test_renderer_render_table_empty() {
    let renderer = Renderer::new(OutputMode::Human);

    renderer.render_table(&[], Some("No items."));
}

#[test]
fn test_renderer_render_table_json_noop() {
    let renderer = Renderer::new(OutputMode::Json);

    renderer.render_table(
        &[serde_json::json!({"name": "test"})],
        Some("Should not render"),
    );
}

#[test]
fn test_renderer_render_summary() {
    let renderer = Renderer::new(OutputMode::Human);

    renderer.render_summary(
        "Repository",
        &[
            ("Name", "org/repo".to_string()),
            ("Private", "false".to_string()),
            ("Default Branch", "main".to_string()),
        ],
    );
}

#[test]
fn test_renderer_render_list() {
    let renderer = Renderer::new(OutputMode::Human);

    renderer.render_list(
        &[
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
        ],
        Some("No items."),
    );
}

#[test]
fn test_renderer_render_list_empty() {
    let renderer = Renderer::new(OutputMode::Human);

    renderer.render_list(&[], Some("Nothing to show."));
}

#[test]
fn test_renderer_render_boxes() {
    let renderer = Renderer::new(OutputMode::Human);

    renderer.render_success_box("Created", "org/new-repo");
    renderer.render_error_box("Failed", "could not delete");

    renderer.render_info_box("Info", "details here");
}

#[test]
fn test_renderer_write_error_with_hint() {
    let renderer = Renderer::new(OutputMode::Human);

    renderer.write_error(
        "Something went wrong",
        Some("Try running gitfleet auth login"),
    );
}

#[test]
fn test_renderer_write_error_json_mode() {
    let renderer = Renderer::new(OutputMode::Json);

    renderer.write_error("Something went wrong", Some("Try again"));
}

#[test]
fn test_renderer_write_error_silent_mode() {
    let renderer = Renderer::new(OutputMode::Silent);

    renderer.write_error("Should not appear", None);
}

#[test]
fn test_renderer_log() {
    let renderer = Renderer::new(OutputMode::Human);

    renderer.log("Processing...");
}

#[test]
fn test_renderer_log_json_noop() {
    let renderer = Renderer::new(OutputMode::Json);

    renderer.log("Should not appear");
}
