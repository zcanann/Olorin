/// Stores recent log lines and output-pane configuration.
#[derive(Clone, Debug)]
pub struct OutputPaneState {
    pub log_lines: Vec<String>,
    pub max_log_line_count: usize,
}

impl Default for OutputPaneState {
    fn default() -> Self {
        Self {
            log_lines: Vec::new(),
            max_log_line_count: 200,
        }
    }
}
