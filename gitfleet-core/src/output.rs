use crate::icons::Icons;
use crate::output_state::OutputMode;
use crate::theme::{Palette, Theme};

pub struct Renderer {
    mode: OutputMode,
    yes: bool,
    palette: Palette,
}

impl Renderer {
    pub fn new(mode: OutputMode) -> Self {
        Self {
            mode,
            yes: false,
            palette: Palette::new(),
        }
    }

    pub fn with_yes(mut self, yes: bool) -> Self {
        self.yes = yes;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        let _resolved = crate::theme::init_colors(theme);

        self.palette = Palette::new();
        self
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn mode(&self) -> OutputMode {
        self.mode
    }

    pub fn yes(&self) -> bool {
        self.yes
    }

    pub fn is_json(&self) -> bool {
        matches!(self.mode, OutputMode::Json)
    }

    pub fn is_human(&self) -> bool {
        matches!(self.mode, OutputMode::Human)
    }

    pub fn is_silent(&self) -> bool {
        matches!(self.mode, OutputMode::Silent)
    }

    pub fn write_result(&self, result: &serde_json::Value) {
        if !self.is_json() {
            return;
        }

        match serde_json::to_string_pretty(result) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("Error serializing output: {e}"),
        }
    }

    pub fn write_value(&self, value: &str) {
        if self.is_silent() {
            return;
        }

        println!("{value}");
    }

    pub fn write_error(&self, message: &str, hint: Option<&str>) {
        if self.is_silent() {
            return;
        }

        if self.is_json() {
            let mut obj = serde_json::json!({
                "success": false,
                "error": message,
            });

            if let Some(h) = hint {
                obj["hint"] = serde_json::Value::String(h.to_string());
            }

            if let Ok(json) = serde_json::to_string_pretty(&obj) {
                eprintln!("{json}");
            }

            return;
        }

        let p = &self.palette;
        eprintln!("{} {}", p.error_bold("Error:"), p.error(message));

        if let Some(h) = hint {
            eprintln!("{} {}", p.muted("Hint:"), p.dim(h));
        }
    }

    pub fn render_table(&self, rows: &[serde_json::Value], empty_message: Option<&str>) {
        self.render_table_titled(rows, empty_message, None, None);
    }

    pub fn render_table_titled(
        &self,
        rows: &[serde_json::Value],
        empty_message: Option<&str>,
        title: Option<&str>,
        columns: Option<&[&str]>,
    ) {
        if !self.is_human() {
            return;
        }

        if rows.is_empty() {
            if let Some(msg) = empty_message {
                println!("{}", self.palette.muted(msg));
            }

            return;
        }

        let p = &self.palette;

        if let Some(t) = title {
            println!(
                "{} {}",
                p.primary_bold(t),
                p.dim(&format!("({})", rows.len()))
            );
        }

        let keys: Vec<&str> = if let Some(cols) = columns {
            cols.to_vec()
        } else {
            let mut collected: Vec<&str> = Vec::new();

            for row in rows {
                if let Some(obj) = row.as_object() {
                    for key in obj.keys() {
                        if !collected.contains(&key.as_str()) {
                            collected.push(key.as_str());
                        }
                    }
                }
            }

            collected
        };

        if keys.is_empty() {
            return;
        }

        fn pad_right(s: &str, width: usize) -> String {
            let visible_len = strip_ansi(s).len();

            let padding = width.saturating_sub(visible_len);
            format!("{}{}", s, " ".repeat(padding))
        }

        let mut widths: Vec<usize> = keys.iter().map(|k| k.len()).collect();

        let rendered_rows: Vec<Vec<String>> = rows
            .iter()
            .filter_map(|row| row.as_object())
            .map(|obj| {
                keys.iter()
                    .enumerate()
                    .map(|(i, k)| {
                        let val = obj.get(*k).map(json_to_string).unwrap_or_default();

                        if val.len() > widths[i] {
                            widths[i] = val.len();
                        }

                        val
                    })
                    .collect()
            })
            .collect();

        let header_line: String = keys
            .iter()
            .enumerate()
            .map(|(i, k)| pad_right(&p.primary_bold(k), widths[i]))
            .collect::<Vec<_>>()
            .join("  ");
        println!("{header_line}");

        for row in &rendered_rows {
            let line: String = row
                .iter()
                .enumerate()
                .map(|(i, v)| pad_right(v, widths[i]))
                .collect::<Vec<_>>()
                .join("  ");
            println!("{line}");
        }
    }

    pub fn render_section(&self, title: &str) {
        if !self.is_human() {
            return;
        }

        let p = &self.palette;
        println!("{}", p.primary_bold(title));

        let line = "\u{2500}".repeat(title.len().max(24));
        println!("{}", p.muted(&line));
    }

    pub fn render_key_values(&self, entries: &[(&str, String)]) {
        if !self.is_human() {
            return;
        }

        let p = &self.palette;

        for (label, value) in entries {
            println!("{:<16} {}", p.muted(label), value);
        }

        println!();
    }

    pub fn render_summary(&self, title: &str, entries: &[(&str, String)]) {
        self.render_section(title);

        self.render_key_values(entries);
    }

    pub fn render_list(&self, items: &[String], empty_message: Option<&str>) {
        if !self.is_human() {
            return;
        }

        if items.is_empty() {
            if let Some(msg) = empty_message {
                println!("{}", self.palette.muted(msg));
            }

            return;
        }

        let p = &self.palette;

        for (i, item) in items.iter().enumerate() {
            println!("{} {item}", p.muted(&format!("{}.", i + 1)));
        }
    }

    pub fn render_box(&self, content: &str, style: &str) {
        if !self.is_human() {
            return;
        }

        let p = &self.palette;
        let (icon, _label) = match style {
            "success" => (Icons::check(), p.success("Success")),
            "error" => (Icons::cross(), p.error("Error")),
            "warning" => (Icons::warning(), p.warning("Warning")),
            _ => (Icons::info(), p.info("Info")),
        };

        let lines: Vec<&str> = content.split('\n').collect();

        if lines.len() <= 1 {
            println!("{} {}", p.bold(icon), content);
        } else {
            println!("{} {}", p.bold(icon), p.bold(lines[0]));

            for line in &lines[1..] {
                println!("  {line}");
            }
        }
    }

    pub fn render_success_box(&self, title: &str, message: &str) {
        self.render_box(&format!("{title}\n{message}"), "success");
    }

    pub fn render_error_box(&self, title: &str, message: &str) {
        self.render_box(&format!("{title}\n{message}"), "error");
    }

    pub fn render_info_box(&self, title: &str, message: &str) {
        self.render_box(&format!("{title}\n{message}"), "info");
    }

    pub fn render_warning_box(&self, title: &str, message: &str) {
        self.render_box(&format!("{title}\n{message}"), "warning");
    }

    pub fn render_panel(&self, title: &str, body: &str) {
        if !self.is_human() {
            return;
        }

        let p = &self.palette;
        let width = title
            .len()
            .max(body.lines().map(|l| l.len()).max().unwrap_or(0))
            .max(20);

        let top = format!(
            "\u{256d}\u{2500}{}\u{2500}\u{256e}",
            "\u{2500}".repeat(width)
        );

        let bottom = format!(
            "\u{2570}\u{2500}{}\u{2500}\u{256f}",
            "\u{2500}".repeat(width)
        );

        let title_line = format!(
            "\u{2502} {}{}",
            p.primary_bold(title),
            " ".repeat(width.saturating_sub(title.len() + 1))
        );

        println!("{}", p.muted(&top));
        println!("{}", p.muted(&title_line));

        for line in body.lines() {
            let padded = format!(
                "\u{2502} {}{}",
                line,
                " ".repeat(width.saturating_sub(line.len() + 1))
            );

            println!("{}", p.muted(&padded));
        }

        println!("{}", p.muted(&bottom));
    }

    pub fn render_status(&self, state: &str) -> String {
        let p = &self.palette;
        let state_lower = state.to_lowercase();

        match state_lower.as_str() {
            "open" | "active" | "running" | "success" | "passed" | "ok" => {
                p.success(&format!("{} {}", Icons::check(), state))
            }
            "closed" | "failed" | "error" | "cancelled" => {
                p.error(&format!("{} {}", Icons::cross(), state))
            }
            "merged" => p.accent(&format!("{} {}", Icons::pull_merged(), state)),
            "pending" | "waiting" | "queued" => {
                p.warning(&format!("{} {}", Icons::warning(), state))
            }

            _ => p.muted(state),
        }
    }

    pub fn spinner(&self, message: &str) -> indicatif::ProgressBar {
        let pb = crate::progress::create_spinner(message);

        if self.is_json() || self.is_silent() {
            pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
        }

        pb
    }

    pub fn progress_bar(&self, total: u64, message: &str) -> indicatif::ProgressBar {
        let pb = crate::progress::create_progress_bar(total, message);

        if self.is_json() || self.is_silent() {
            pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
        }

        pb
    }

    pub fn log(&self, message: &str) {
        if !self.is_human() {
            return;
        }

        println!("{message}");
    }
}

fn json_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn strip_ansi(s: &str) -> String {
    regex::Regex::new(r"\x1b\[[0-9;]*m")
        .expect("ANSI escape regex is valid")
        .replace_all(s, "")
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_state::OutputMode;

    #[test]
    fn test_renderer_new_json() {
        let r = Renderer::new(OutputMode::Json);

        assert!(r.is_json());

        assert!(!r.is_human());
        assert!(!r.is_silent());

        assert_eq!(r.mode(), OutputMode::Json);
    }

    #[test]
    fn test_renderer_new_human() {
        let r = Renderer::new(OutputMode::Human);

        assert!(!r.is_json());

        assert!(r.is_human());
        assert!(!r.is_silent());

        assert_eq!(r.mode(), OutputMode::Human);
    }

    #[test]
    fn test_renderer_new_silent() {
        let r = Renderer::new(OutputMode::Silent);

        assert!(!r.is_json());

        assert!(!r.is_human());
        assert!(r.is_silent());

        assert_eq!(r.mode(), OutputMode::Silent);
    }

    #[test]
    fn test_renderer_with_theme() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);

        assert!(r.is_human());
    }

    #[test]
    fn test_write_result_json_outputs_value() {
        let r = Renderer::new(OutputMode::Json);

        let val = serde_json::json!({"key": "value"});
        r.write_result(&val);
    }

    #[test]
    fn test_write_result_human_is_noop() {
        let r = Renderer::new(OutputMode::Human);

        let val = serde_json::json!({"key": "value"});
        r.write_result(&val);
    }

    #[test]
    fn test_write_result_silent_is_noop() {
        let r = Renderer::new(OutputMode::Silent);

        let val = serde_json::json!({"key": "value"});
        r.write_result(&val);
    }

    #[test]
    fn test_write_value_human() {
        let r = Renderer::new(OutputMode::Human);
        r.write_value("hello");
    }

    #[test]
    fn test_write_value_silent_is_noop() {
        let r = Renderer::new(OutputMode::Silent);
        r.write_value("hello");
    }

    #[test]
    fn test_write_error_json() {
        let r = Renderer::new(OutputMode::Json);
        r.write_error("something failed", Some("try again"));
    }

    #[test]
    fn test_write_error_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.write_error("something failed", Some("try again"));
    }

    #[test]
    fn test_write_error_silent_is_noop() {
        let r = Renderer::new(OutputMode::Silent);
        r.write_error("something failed", None);
    }

    #[test]
    fn test_write_error_without_hint() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.write_error("something failed", None);
    }

    #[test]
    fn test_render_table_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);

        let rows = vec![
            serde_json::json!({"name": "repo1", "private": false}),
            serde_json::json!({"name": "repo2", "private": true}),
        ];
        r.render_table(&rows, Some("No repos."));
    }

    #[test]
    fn test_render_table_no_quotes() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);

        let rows = vec![serde_json::json!({"name": "myrepo", "count": 42})];
        r.render_table(&rows, None);
    }

    #[test]
    fn test_render_table_json_is_noop() {
        let r = Renderer::new(OutputMode::Json);

        let rows = vec![serde_json::json!({"name": "repo1"})];
        r.render_table(&rows, Some("No repos."));
    }

    #[test]
    fn test_render_table_empty_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_table(&[], Some("No items."));
    }

    #[test]
    fn test_render_table_empty_no_message() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_table(&[], None);
    }

    #[test]
    fn test_render_summary_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_summary(
            "Repository",
            &[
                ("Name", "org/repo".to_string()),
                ("Private", "false".to_string()),
            ],
        );
    }

    #[test]
    fn test_render_summary_json_is_noop() {
        let r = Renderer::new(OutputMode::Json);
        r.render_summary("Repository", &[("Name", "org/repo".to_string())]);
    }

    #[test]
    fn test_render_success_box_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_success_box("Created", "org/new-repo");
    }

    #[test]
    fn test_render_success_box_json_is_noop() {
        let r = Renderer::new(OutputMode::Json);
        r.render_success_box("Created", "org/new-repo");
    }

    #[test]
    fn test_render_error_box_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_error_box("Failed", "could not delete");
    }

    #[test]
    fn test_render_info_box_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_info_box("Info", "details here");
    }

    #[test]
    fn test_render_warning_box_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_warning_box("Warning", "be careful");
    }

    #[test]
    fn test_render_panel() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_panel("Repository", "org/repo\nprivate: false");
    }

    #[test]
    fn test_render_status_open() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);

        let s = r.render_status("open");

        assert!(!s.is_empty());
    }

    #[test]
    fn test_render_status_closed() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);

        let s = r.render_status("closed");

        assert!(!s.is_empty());
    }

    #[test]
    fn test_render_status_merged() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);

        let s = r.render_status("merged");

        assert!(!s.is_empty());
    }

    #[test]
    fn test_render_status_pending() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);

        let s = r.render_status("pending");

        assert!(!s.is_empty());
    }

    #[test]
    fn test_render_status_unknown() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);

        let s = r.render_status("unknown");

        assert!(!s.is_empty());
    }

    #[test]
    fn test_render_list_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_list(
            &["item1".to_string(), "item2".to_string()],
            Some("No items."),
        );
    }

    #[test]
    fn test_render_list_empty_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_list(&[], Some("No items."));
    }

    #[test]
    fn test_render_list_json_is_noop() {
        let r = Renderer::new(OutputMode::Json);
        r.render_list(&["a".to_string()], None);
    }

    #[test]
    fn test_render_section_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_section("Details");
    }

    #[test]
    fn test_render_section_json_is_noop() {
        let r = Renderer::new(OutputMode::Json);
        r.render_section("Details");
    }

    #[test]
    fn test_render_key_values_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_key_values(&[("Key", "value".to_string())]);
    }

    #[test]
    fn test_render_key_values_json_is_noop() {
        let r = Renderer::new(OutputMode::Json);
        r.render_key_values(&[("Key", "value".to_string())]);
    }

    #[test]
    fn test_log_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.log("some message");
    }

    #[test]
    fn test_log_json_is_noop() {
        let r = Renderer::new(OutputMode::Json);
        r.log("some message");
    }

    #[test]
    fn test_render_box_human_success() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_box("content", "success");
    }

    #[test]
    fn test_render_box_human_error() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_box("content", "error");
    }

    #[test]
    fn test_render_box_human_warning() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_box("content", "warning");
    }

    #[test]
    fn test_render_box_human_info() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_box("content", "info");
    }

    #[test]
    fn test_render_box_human_unknown_style() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);
        r.render_box("content", "unknown");
    }

    #[test]
    fn test_render_box_json_is_noop() {
        let r = Renderer::new(OutputMode::Json);
        r.render_box("content", "success");
    }

    #[test]
    fn test_json_to_string_string() {
        assert_eq!(json_to_string(&serde_json::json!("hello")), "hello");
    }

    #[test]
    fn test_json_to_string_bool() {
        assert_eq!(json_to_string(&serde_json::json!(true)), "true");
    }

    #[test]
    fn test_json_to_string_number() {
        assert_eq!(json_to_string(&serde_json::json!(42)), "42");
    }

    #[test]
    fn test_json_to_string_null() {
        assert_eq!(json_to_string(&serde_json::Value::Null), "");
    }

    #[test]
    fn test_spinner_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);

        let pb = r.spinner("Loading...");
        pb.finish_and_clear();
    }

    #[test]
    fn test_spinner_json_hidden() {
        let r = Renderer::new(OutputMode::Json);

        let pb = r.spinner("Loading...");
        pb.finish_and_clear();
    }

    #[test]
    fn test_progress_bar_human() {
        let r = Renderer::new(OutputMode::Human).with_theme(Theme::Dark);

        let pb = r.progress_bar(10, "Processing...");
        pb.finish_and_clear();
    }
}
