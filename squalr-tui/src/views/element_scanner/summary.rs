use crate::views::element_scanner::pane_state::ElementScannerPaneState;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;

pub fn build_element_scanner_summary_lines(element_scanner_pane_state: &ElementScannerPaneState) -> Vec<String> {
    let mut summary_lines = vec![
        "Actions: s start, n reset/new, c collect, t/T data type, a add, x remove.".to_string(),
        "Constraint edit: j/k select, m/M compare type, digits/-/. append, Backspace, Ctrl+u clear.".to_string(),
        format!("data_type={}", element_scanner_pane_state.selected_data_type_name()),
        format!("constraints={}", element_scanner_pane_state.active_constraint_count()),
        format!("selected_constraint={}", element_scanner_pane_state.selected_constraint_row_index + 1),
        format!("pending_scan={}", element_scanner_pane_state.has_pending_scan_request),
        format!("has_results={}", element_scanner_pane_state.has_scan_results),
        format!("last_result_count={}", element_scanner_pane_state.last_result_count),
        format!("last_total_bytes={}", element_scanner_pane_state.last_total_size_in_bytes),
        format!("status={}", element_scanner_pane_state.status_message),
    ];

    for (constraint_row_index, constraint_row) in element_scanner_pane_state.constraint_rows.iter().enumerate() {
        let selected_marker = if element_scanner_pane_state.selected_constraint_row_index == constraint_row_index {
            ">"
        } else {
            " "
        };
        summary_lines.push(format!(
            "{} {} {}",
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
