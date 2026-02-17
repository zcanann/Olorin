use crate::views::element_scanner::pane_state::{ElementScannerConstraintState, ElementScannerPaneState};
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;

pub fn build_element_scanner_summary_lines_with_capacity(
    element_scanner_pane_state: &ElementScannerPaneState,
    line_capacity: usize,
) -> Vec<String> {
    if line_capacity == 0 {
        return Vec::new();
    }

    let constraint_row_lines = build_constraint_row_lines(element_scanner_pane_state);
    let constraint_line_capacity = line_capacity.saturating_sub(6);
    let visible_constraint_lines = selected_constraint_window_lines(element_scanner_pane_state, &constraint_row_lines, constraint_line_capacity);

    let mut prioritized_lines = vec![
        "[ACT] s scan | n reset | c collect | a add | x remove.".to_string(),
        "[CTRL] t/T type | m/M compare | j/k row | type value.".to_string(),
        format!("[DATA] type={}.", element_scanner_pane_state.selected_data_type_name()),
    ];
    prioritized_lines.extend(visible_constraint_lines);
    prioritized_lines.push(format!(
        "[SCAN] constraints={} | selected_row={} | pending={} | has_results={}.",
        element_scanner_pane_state.active_constraint_count(),
        element_scanner_pane_state.selected_constraint_row_index + 1,
        element_scanner_pane_state.has_pending_scan_request,
        element_scanner_pane_state.has_scan_results
    ));
    prioritized_lines.push(format!("[STAT] {}.", element_scanner_pane_state.status_message));
    prioritized_lines.push(format!(
        "[LAST] result_count={} | total_bytes={}.",
        element_scanner_pane_state.last_result_count, element_scanner_pane_state.last_total_size_in_bytes
    ));

    prioritized_lines.into_iter().take(line_capacity).collect()
}

fn build_constraint_row_lines(element_scanner_pane_state: &ElementScannerPaneState) -> Vec<String> {
    element_scanner_pane_state
        .constraint_rows
        .iter()
        .enumerate()
        .map(|(constraint_row_index, constraint_row)| {
            build_constraint_row_line(constraint_row, element_scanner_pane_state.selected_constraint_row_index == constraint_row_index)
        })
        .collect()
}

fn build_constraint_row_line(
    constraint_row: &ElementScannerConstraintState,
    is_selected: bool,
) -> String {
    let selected_marker = if is_selected { ">" } else { " " };
    format!(
        "{} [ROW] {} {}.",
        selected_marker,
        scan_compare_type_label(constraint_row.scan_compare_type),
        constraint_row.scan_value_text
    )
}

fn selected_constraint_window_lines(
    element_scanner_pane_state: &ElementScannerPaneState,
    constraint_row_lines: &[String],
    line_capacity: usize,
) -> Vec<String> {
    if line_capacity == 0 {
        return Vec::new();
    }

    let selection_window_range = build_selection_relative_viewport_range(
        constraint_row_lines.len(),
        Some(element_scanner_pane_state.selected_constraint_row_index),
        line_capacity,
    );
    constraint_row_lines[selection_window_range].to_vec()
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
