use squalr_engine_api::structures::logging::log_event::LogEvent;

/// Stores recent log lines and output-pane configuration.
#[derive(Clone, Debug)]
pub struct OutputPaneState {
    pub log_lines: Vec<String>,
    pub max_log_line_count: usize,
    pub did_auto_scroll_to_latest: bool,
    pub status_message: String,
}

impl OutputPaneState {
    pub fn apply_log_history_with_feedback(
        &mut self,
        log_history: Vec<LogEvent>,
        should_update_status_message: bool,
    ) {
        self.log_lines = log_history
            .into_iter()
            .map(|log_event| format!("[{}] {}", log_event.level, log_event.message))
            .collect();
        self.trim_to_max_line_count();
        self.did_auto_scroll_to_latest = true;
        if should_update_status_message {
            self.status_message = format!("Loaded {} log lines.", self.log_lines.len());
        }
    }

    pub fn clear_log_lines(&mut self) {
        self.log_lines.clear();
        self.did_auto_scroll_to_latest = true;
        self.status_message = "Cleared output pane log lines.".to_string();
    }

    pub fn increase_max_line_count(&mut self) {
        self.max_log_line_count = (self.max_log_line_count + 25).min(2_000);
        self.trim_to_max_line_count();
    }

    pub fn decrease_max_line_count(&mut self) {
        self.max_log_line_count = self.max_log_line_count.saturating_sub(25).max(25);
        self.trim_to_max_line_count();
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let mut summary_lines = vec![
            "Actions: r refresh log history, x clear, +/- max lines.".to_string(),
            format!("log_line_count={}", self.log_lines.len()),
            format!("max_log_lines={}", self.max_log_line_count),
            format!("auto_scroll_latest={}", self.did_auto_scroll_to_latest),
            format!("status={}", self.status_message),
        ];

        let preview_line_count = self.log_lines.len().min(8);
        if preview_line_count > 0 {
            summary_lines.push("Recent:".to_string());
            let start_line_index = self.log_lines.len().saturating_sub(preview_line_count);
            for preview_line in &self.log_lines[start_line_index..] {
                summary_lines.push(preview_line.clone());
            }
        }

        summary_lines
    }

    fn trim_to_max_line_count(&mut self) {
        if self.log_lines.len() <= self.max_log_line_count {
            return;
        }

        let remove_line_count = self.log_lines.len() - self.max_log_line_count;
        self.log_lines.drain(0..remove_line_count);
    }
}

impl Default for OutputPaneState {
    fn default() -> Self {
        Self {
            log_lines: Vec::new(),
            max_log_line_count: 200,
            did_auto_scroll_to_latest: false,
            status_message: "Ready.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::views::output_pane_state::OutputPaneState;
    use log::Level;
    use squalr_engine_api::structures::logging::log_event::LogEvent;

    #[test]
    fn apply_log_history_trims_to_max_line_count() {
        let mut output_pane_state = OutputPaneState {
            max_log_line_count: 2,
            ..OutputPaneState::default()
        };

        output_pane_state.apply_log_history_with_feedback(
            vec![
                LogEvent {
                    level: Level::Info,
                    message: "line1".to_string(),
                },
                LogEvent {
                    level: Level::Warn,
                    message: "line2".to_string(),
                },
                LogEvent {
                    level: Level::Error,
                    message: "line3".to_string(),
                },
            ],
            true,
        );

        assert_eq!(output_pane_state.log_lines.len(), 2);
        assert!(output_pane_state.log_lines[0].contains("line2"));
        assert!(output_pane_state.log_lines[1].contains("line3"));
    }

    #[test]
    fn clear_log_lines_resets_collection() {
        let mut output_pane_state = OutputPaneState::default();
        output_pane_state.log_lines = vec!["existing".to_string()];

        output_pane_state.clear_log_lines();

        assert!(output_pane_state.log_lines.is_empty());
    }

    #[test]
    fn apply_log_history_can_preserve_status_message() {
        let mut output_pane_state = OutputPaneState::default();
        output_pane_state.status_message = "Manual status should remain.".to_string();

        output_pane_state.apply_log_history_with_feedback(
            vec![LogEvent {
                level: Level::Info,
                message: "line".to_string(),
            }],
            false,
        );

        assert_eq!(output_pane_state.status_message, "Manual status should remain.");
    }
}
