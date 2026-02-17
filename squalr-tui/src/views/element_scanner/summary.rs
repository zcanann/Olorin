use crate::views::element_scanner::pane_state::ElementScannerPaneState;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;

pub fn build_element_scanner_summary_lines(element_scanner_pane_state: &ElementScannerPaneState) -> Vec<String> {
    let mut summary_lines = vec![
        "[ACT] s scan | n new/reset | c collect | a add | x remove.".to_string(),
        "[TYPE] t next | T prev data type.".to_string(),
        "[EDIT] j/k row | m/M compare | digits - . append | Backspace | Ctrl+u clear.".to_string(),
        format!("[DATA] type={}.", element_scanner_pane_state.selected_data_type_name()),
        format!(
            "[META] constraints={} | selected_row={} | pending_scan={} | has_results={}.",
            element_scanner_pane_state.active_constraint_count(),
            element_scanner_pane_state.selected_constraint_row_index + 1,
            element_scanner_pane_state.has_pending_scan_request,
            element_scanner_pane_state.has_scan_results
        ),
        format!(
            "[LAST] result_count={} | total_bytes={}.",
            element_scanner_pane_state.last_result_count, element_scanner_pane_state.last_total_size_in_bytes
        ),
        format!("[STAT] {}.", element_scanner_pane_state.status_message),
    ];

    for (constraint_row_index, constraint_row) in element_scanner_pane_state.constraint_rows.iter().enumerate() {
        let selected_marker = if element_scanner_pane_state.selected_constraint_row_index == constraint_row_index {
            ">"
        } else {
            " "
        };
        summary_lines.push(format!(
            "{} [ROW] {} {}.",
            selected_marker,
            scan_compare_type_label(constraint_row.scan_compare_type),
            constraint_row.scan_value_text
        ));
    }

    summary_lines
}

fn scan_compare_type_label(scan_compare_type: ScanCompareType) -> &'static str {
    match scan_compare_type {
        ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal) => "==",
        ScanCompareType::Immediate(ScanCompareTypeImmediate::NotEqual) => "!=",
        ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThan) => ">",
        ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThanOrEqual) => ">=",
        ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThan) => "<",
        ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual) => "<=",
        ScanCompareType::Relative(ScanCompareTypeRelative::Changed) => "changed",
        ScanCompareType::Relative(ScanCompareTypeRelative::Unchanged) => "unchanged",
        ScanCompareType::Relative(ScanCompareTypeRelative::Increased) => "increased",
        ScanCompareType::Relative(ScanCompareTypeRelative::Decreased) => "decreased",
        ScanCompareType::Delta(ScanCompareTypeDelta::IncreasedByX) => "+x",
        ScanCompareType::Delta(ScanCompareTypeDelta::DecreasedByX) => "-x",
        ScanCompareType::Delta(ScanCompareTypeDelta::MultipliedByX) => "*x",
        ScanCompareType::Delta(ScanCompareTypeDelta::DividedByX) => "/x",
        ScanCompareType::Delta(ScanCompareTypeDelta::ModuloByX) => "%x",
        ScanCompareType::Delta(ScanCompareTypeDelta::ShiftLeftByX) => "<<x",
        ScanCompareType::Delta(ScanCompareTypeDelta::ShiftRightByX) => ">>x",
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalAndByX) => "&x",
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalOrByX) => "|x",
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalXorByX) => "^x",
    }
}

#[cfg(test)]
mod tests {
    use super::build_element_scanner_summary_lines;
    use crate::views::element_scanner::pane_state::ElementScannerPaneState;

    #[test]
    fn summary_uses_condensed_marker_group_lead_lines() {
        let element_scanner_pane_state = ElementScannerPaneState::default();
        let summary_lines = build_element_scanner_summary_lines(&element_scanner_pane_state);

        assert!(summary_lines[0].starts_with("[ACT]"));
        assert!(summary_lines[1].starts_with("[TYPE]"));
        assert!(summary_lines[2].starts_with("[EDIT]"));
    }
}
