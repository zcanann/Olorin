use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use crate::views::settings::pane_state::{SettingsCategory, SettingsPaneState};
use squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;

pub fn build_settings_summary_lines(settings_pane_state: &SettingsPaneState) -> Vec<String> {
    build_settings_summary_lines_with_capacity(settings_pane_state, usize::MAX)
}

pub fn build_settings_summary_lines_with_capacity(
    settings_pane_state: &SettingsPaneState,
    line_capacity: usize,
) -> Vec<String> {
    if line_capacity == 0 {
        return Vec::new();
    }

    let category_lines = selected_category_lines(settings_pane_state);
    let selected_field_lines = selected_field_window_lines(settings_pane_state, &category_lines, 1);
    let additional_field_capacity = line_capacity.saturating_sub(7);
    let additional_field_lines = selected_field_window_lines(settings_pane_state, &category_lines, additional_field_capacity);

    let mut prioritized_lines = vec![
        "[CAT] ] next | [ prev | r refresh-all.".to_string(),
        "[NAV] j/k field.".to_string(),
        "[ACT] Space toggle | +/- step | </> cycle enum | Enter apply category.".to_string(),
    ];
    prioritized_lines.extend(selected_field_lines);
    prioritized_lines.push(format!("[STAT] {}.", settings_pane_state.status_message));
    prioritized_lines.push(format!(
        "[META] category={} | selected_field={}.",
        settings_pane_state.selected_category.title(),
        settings_pane_state.selected_field_index
    ));
    prioritized_lines.push(format!(
        "[LOAD] pending_changes={} | loaded_once={} | refreshing={} | applying={}.",
        settings_pane_state.has_pending_changes,
        settings_pane_state.has_loaded_settings_once,
        settings_pane_state.is_refreshing_settings,
        settings_pane_state.is_applying_settings
    ));
    prioritized_lines.extend(additional_field_lines);

    prioritized_lines.into_iter().take(line_capacity).collect()
}

fn selected_field_window_lines(
    settings_pane_state: &SettingsPaneState,
    category_lines: &[String],
    line_capacity: usize,
) -> Vec<String> {
    if line_capacity == 0 {
        return Vec::new();
    }

    let selection_window_range = build_selection_relative_viewport_range(category_lines.len(), Some(settings_pane_state.selected_field_index), line_capacity);
    category_lines[selection_window_range].to_vec()
}

fn selected_category_lines(settings_pane_state: &SettingsPaneState) -> Vec<String> {
    match settings_pane_state.selected_category {
        SettingsCategory::General => general_summary_lines(settings_pane_state),
        SettingsCategory::Memory => memory_summary_lines(settings_pane_state),
        SettingsCategory::Scan => scan_summary_lines(settings_pane_state),
    }
}

fn general_summary_lines(settings_pane_state: &SettingsPaneState) -> Vec<String> {
    vec![format!(
        "{} [FLD] engine_request_delay_ms={}.",
        selection_marker(settings_pane_state.selected_field_index, 0),
        settings_pane_state.general_settings.engine_request_delay_ms
    )]
}

fn memory_summary_lines(settings_pane_state: &SettingsPaneState) -> Vec<String> {
    vec![
        format!(
            "{} [FLD] memory_type_none={}.",
            selection_marker(settings_pane_state.selected_field_index, 0),
            settings_pane_state.memory_settings.memory_type_none
        ),
        format!(
            "{} [FLD] memory_type_private={}.",
            selection_marker(settings_pane_state.selected_field_index, 1),
            settings_pane_state.memory_settings.memory_type_private
        ),
        format!(
            "{} [FLD] memory_type_image={}.",
            selection_marker(settings_pane_state.selected_field_index, 2),
            settings_pane_state.memory_settings.memory_type_image
        ),
        format!(
            "{} [FLD] memory_type_mapped={}.",
            selection_marker(settings_pane_state.selected_field_index, 3),
            settings_pane_state.memory_settings.memory_type_mapped
        ),
        format!(
            "{} [FLD] required_write={}.",
            selection_marker(settings_pane_state.selected_field_index, 4),
            settings_pane_state.memory_settings.required_write
        ),
        format!(
            "{} [FLD] required_execute={}.",
            selection_marker(settings_pane_state.selected_field_index, 5),
            settings_pane_state.memory_settings.required_execute
        ),
        format!(
            "{} [FLD] required_copy_on_write={}.",
            selection_marker(settings_pane_state.selected_field_index, 6),
            settings_pane_state.memory_settings.required_copy_on_write
        ),
        format!(
            "{} [FLD] excluded_write={}.",
            selection_marker(settings_pane_state.selected_field_index, 7),
            settings_pane_state.memory_settings.excluded_write
        ),
        format!(
            "{} [FLD] excluded_execute={}.",
            selection_marker(settings_pane_state.selected_field_index, 8),
            settings_pane_state.memory_settings.excluded_execute
        ),
        format!(
            "{} [FLD] excluded_copy_on_write={}.",
            selection_marker(settings_pane_state.selected_field_index, 9),
            settings_pane_state.memory_settings.excluded_copy_on_write
        ),
        format!(
            "{} [FLD] start_address=0x{:X}.",
            selection_marker(settings_pane_state.selected_field_index, 10),
            settings_pane_state.memory_settings.start_address
        ),
        format!(
            "{} [FLD] end_address=0x{:X}.",
            selection_marker(settings_pane_state.selected_field_index, 11),
            settings_pane_state.memory_settings.end_address
        ),
        format!(
            "{} [FLD] only_query_usermode={}.",
            selection_marker(settings_pane_state.selected_field_index, 12),
            settings_pane_state.memory_settings.only_query_usermode
        ),
    ]
}

fn scan_summary_lines(settings_pane_state: &SettingsPaneState) -> Vec<String> {
    vec![
        format!(
            "{} [FLD] results_page_size={}.",
            selection_marker(settings_pane_state.selected_field_index, 0),
            settings_pane_state.scan_settings.results_page_size
        ),
        format!(
            "{} [FLD] freeze_interval_ms={}.",
            selection_marker(settings_pane_state.selected_field_index, 1),
            settings_pane_state.scan_settings.freeze_interval_ms
        ),
        format!(
            "{} [FLD] project_read_interval_ms={}.",
            selection_marker(settings_pane_state.selected_field_index, 2),
            settings_pane_state.scan_settings.project_read_interval_ms
        ),
        format!(
            "{} [FLD] results_read_interval_ms={}.",
            selection_marker(settings_pane_state.selected_field_index, 3),
            settings_pane_state.scan_settings.results_read_interval_ms
        ),
        format!(
            "{} [FLD] memory_alignment={}.",
            selection_marker(settings_pane_state.selected_field_index, 4),
            memory_alignment_label(settings_pane_state.scan_settings.memory_alignment)
        ),
        format!(
            "{} [FLD] memory_read_mode={}.",
            selection_marker(settings_pane_state.selected_field_index, 5),
            memory_read_mode_label(settings_pane_state.scan_settings.memory_read_mode)
        ),
        format!(
            "{} [FLD] floating_point_tolerance={}.",
            selection_marker(settings_pane_state.selected_field_index, 6),
            floating_point_tolerance_label(settings_pane_state.scan_settings.floating_point_tolerance)
        ),
        format!(
            "{} [FLD] is_single_threaded_scan={}.",
            selection_marker(settings_pane_state.selected_field_index, 7),
            settings_pane_state.scan_settings.is_single_threaded_scan
        ),
        format!(
            "{} [FLD] debug_perform_validation_scan={}.",
            selection_marker(settings_pane_state.selected_field_index, 8),
            settings_pane_state.scan_settings.debug_perform_validation_scan
        ),
    ]
}

fn selection_marker(
    selected_field_index: usize,
    field_position: usize,
) -> &'static str {
    if selected_field_index == field_position { ">" } else { " " }
}

fn memory_alignment_label(memory_alignment: Option<MemoryAlignment>) -> &'static str {
    match memory_alignment {
        Some(MemoryAlignment::Alignment1) => "1",
        Some(MemoryAlignment::Alignment2) => "2",
        Some(MemoryAlignment::Alignment4) => "4",
        Some(MemoryAlignment::Alignment8) => "8",
        None => "none",
    }
}

fn memory_read_mode_label(memory_read_mode: MemoryReadMode) -> &'static str {
    match memory_read_mode {
        MemoryReadMode::Skip => "skip",
        MemoryReadMode::ReadBeforeScan => "before_scan",
        MemoryReadMode::ReadInterleavedWithScan => "interleaved",
    }
}

fn floating_point_tolerance_label(floating_point_tolerance: FloatingPointTolerance) -> &'static str {
    match floating_point_tolerance {
        FloatingPointTolerance::Tolerance10E1 => "0.1",
        FloatingPointTolerance::Tolerance10E2 => "0.01",
        FloatingPointTolerance::Tolerance10E3 => "0.001",
        FloatingPointTolerance::Tolerance10E4 => "0.0001",
        FloatingPointTolerance::Tolerance10E5 => "0.00001",
        FloatingPointTolerance::ToleranceEpsilon => "epsilon",
    }
}
