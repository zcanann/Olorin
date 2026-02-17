use squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;
use squalr_engine_api::structures::settings::general_settings::GeneralSettings;
use squalr_engine_api::structures::settings::memory_settings::MemorySettings;
use squalr_engine_api::structures::settings::scan_settings::ScanSettings;

/// Category selection for settings-pane routing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SettingsCategory {
    #[default]
    General,
    Memory,
    Scan,
}

impl SettingsCategory {
    pub fn all_categories() -> [SettingsCategory; 3] {
        [
            SettingsCategory::General,
            SettingsCategory::Memory,
            SettingsCategory::Scan,
        ]
    }

    pub fn title(self) -> &'static str {
        match self {
            SettingsCategory::General => "General",
            SettingsCategory::Memory => "Memory",
            SettingsCategory::Scan => "Scan",
        }
    }
}

/// Stores state for settings pages and staged changes.
#[derive(Clone, Debug)]
pub struct SettingsPaneState {
    pub selected_category: SettingsCategory,
    pub selected_field_index: usize,
    pub has_pending_changes: bool,
    pub has_loaded_settings_once: bool,
    pub is_refreshing_settings: bool,
    pub is_applying_settings: bool,
    pub general_settings: GeneralSettings,
    pub memory_settings: MemorySettings,
    pub scan_settings: ScanSettings,
    pub status_message: String,
}

impl SettingsPaneState {
    pub fn cycle_category_forward(&mut self) {
        let all_categories = SettingsCategory::all_categories();
        let selected_category_position = all_categories
            .iter()
            .position(|category| *category == self.selected_category)
            .unwrap_or(0);
        let next_category_position = (selected_category_position + 1) % all_categories.len();
        self.selected_category = all_categories[next_category_position];
        self.selected_field_index = 0;
    }

    pub fn cycle_category_backward(&mut self) {
        let all_categories = SettingsCategory::all_categories();
        let selected_category_position = all_categories
            .iter()
            .position(|category| *category == self.selected_category)
            .unwrap_or(0);
        let previous_category_position = if selected_category_position == 0 {
            all_categories.len() - 1
        } else {
            selected_category_position - 1
        };
        self.selected_category = all_categories[previous_category_position];
        self.selected_field_index = 0;
    }

    pub fn select_next_field(&mut self) {
        let field_count = self.field_count_for_selected_category();
        if field_count == 0 {
            self.selected_field_index = 0;
            return;
        }

        self.selected_field_index = (self.selected_field_index + 1) % field_count;
    }

    pub fn select_previous_field(&mut self) {
        let field_count = self.field_count_for_selected_category();
        if field_count == 0 {
            self.selected_field_index = 0;
            return;
        }

        self.selected_field_index = if self.selected_field_index == 0 {
            field_count - 1
        } else {
            self.selected_field_index - 1
        };
    }

    pub fn toggle_selected_boolean_field(&mut self) -> bool {
        let mut did_change_value = false;

        match self.selected_category {
            SettingsCategory::General => {}
            SettingsCategory::Memory => match self.selected_field_index {
                0 => {
                    self.memory_settings.memory_type_none = !self.memory_settings.memory_type_none;
                    did_change_value = true;
                }
                1 => {
                    self.memory_settings.memory_type_private = !self.memory_settings.memory_type_private;
                    did_change_value = true;
                }
                2 => {
                    self.memory_settings.memory_type_image = !self.memory_settings.memory_type_image;
                    did_change_value = true;
                }
                3 => {
                    self.memory_settings.memory_type_mapped = !self.memory_settings.memory_type_mapped;
                    did_change_value = true;
                }
                4 => {
                    self.memory_settings.required_write = !self.memory_settings.required_write;
                    did_change_value = true;
                }
                5 => {
                    self.memory_settings.required_execute = !self.memory_settings.required_execute;
                    did_change_value = true;
                }
                6 => {
                    self.memory_settings.required_copy_on_write = !self.memory_settings.required_copy_on_write;
                    did_change_value = true;
                }
                7 => {
                    self.memory_settings.excluded_write = !self.memory_settings.excluded_write;
                    did_change_value = true;
                }
                8 => {
                    self.memory_settings.excluded_execute = !self.memory_settings.excluded_execute;
                    did_change_value = true;
                }
                9 => {
                    self.memory_settings.excluded_copy_on_write = !self.memory_settings.excluded_copy_on_write;
                    did_change_value = true;
                }
                12 => {
                    self.memory_settings.only_query_usermode = !self.memory_settings.only_query_usermode;
                    did_change_value = true;
                }
                _ => {}
            },
            SettingsCategory::Scan => match self.selected_field_index {
                7 => {
                    self.scan_settings.is_single_threaded_scan = !self.scan_settings.is_single_threaded_scan;
                    did_change_value = true;
                }
                8 => {
                    self.scan_settings.debug_perform_validation_scan = !self.scan_settings.debug_perform_validation_scan;
                    did_change_value = true;
                }
                _ => {}
            },
        }

        if did_change_value {
            self.has_pending_changes = true;
        }

        did_change_value
    }

    pub fn step_selected_numeric_field(
        &mut self,
        increase_value: bool,
    ) -> bool {
        let mut did_change_value = false;

        match self.selected_category {
            SettingsCategory::General => {
                if self.selected_field_index == 0 {
                    self.general_settings.engine_request_delay_ms =
                        Self::step_u64_clamped(self.general_settings.engine_request_delay_ms, increase_value, 25, 0, 5_000);
                    did_change_value = true;
                }
            }
            SettingsCategory::Memory => match self.selected_field_index {
                10 => {
                    self.memory_settings.start_address = Self::step_u64_clamped(self.memory_settings.start_address, increase_value, 0x1000, 0, u64::MAX);
                    did_change_value = true;
                }
                11 => {
                    self.memory_settings.end_address = Self::step_u64_clamped(self.memory_settings.end_address, increase_value, 0x1000, 0, u64::MAX);
                    did_change_value = true;
                }
                _ => {}
            },
            SettingsCategory::Scan => match self.selected_field_index {
                0 => {
                    self.scan_settings.results_page_size = Self::step_u32_clamped(self.scan_settings.results_page_size, increase_value, 1, 1, 1_024);
                    did_change_value = true;
                }
                1 => {
                    self.scan_settings.freeze_interval_ms = Self::step_u64_clamped(self.scan_settings.freeze_interval_ms, increase_value, 25, 0, 5_000);
                    did_change_value = true;
                }
                2 => {
                    self.scan_settings.project_read_interval_ms =
                        Self::step_u64_clamped(self.scan_settings.project_read_interval_ms, increase_value, 25, 0, 5_000);
                    did_change_value = true;
                }
                3 => {
                    self.scan_settings.results_read_interval_ms =
                        Self::step_u64_clamped(self.scan_settings.results_read_interval_ms, increase_value, 25, 0, 5_000);
                    did_change_value = true;
                }
                _ => {}
            },
        }

        if did_change_value {
            self.has_pending_changes = true;
        }

        did_change_value
    }

    pub fn cycle_selected_enum_field(
        &mut self,
        move_forward: bool,
    ) -> bool {
        let mut did_change_value = false;

        if self.selected_category == SettingsCategory::Scan {
            match self.selected_field_index {
                4 => {
                    self.scan_settings.memory_alignment = Some(Self::next_memory_alignment(self.scan_settings.memory_alignment, move_forward));
                    did_change_value = true;
                }
                5 => {
                    self.scan_settings.memory_read_mode = Self::next_memory_read_mode(self.scan_settings.memory_read_mode, move_forward);
                    did_change_value = true;
                }
                6 => {
                    self.scan_settings.floating_point_tolerance =
                        Self::next_floating_point_tolerance(self.scan_settings.floating_point_tolerance, move_forward);
                    did_change_value = true;
                }
                _ => {}
            }
        }

        if did_change_value {
            self.has_pending_changes = true;
        }

        did_change_value
    }

    pub fn apply_general_settings(
        &mut self,
        general_settings: GeneralSettings,
    ) {
        self.general_settings = general_settings;
        self.has_pending_changes = false;
    }

    pub fn apply_memory_settings(
        &mut self,
        memory_settings: MemorySettings,
    ) {
        self.memory_settings = memory_settings;
        self.has_pending_changes = false;
    }

    pub fn apply_scan_settings(
        &mut self,
        scan_settings: ScanSettings,
    ) {
        self.scan_settings = scan_settings;
        self.has_pending_changes = false;
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let mut summary_lines = vec![
            "Category: ]/[ cycle, r refresh all.".to_string(),
            "Field nav: j/k select row.".to_string(),
            "Mutate: Space toggle bool, +/- step numeric, </> cycle enum, Enter apply category.".to_string(),
            format!("category={}", self.selected_category.title()),
            format!("selected_field={}", self.selected_field_index),
            format!("pending_changes={}", self.has_pending_changes),
            format!("loaded_once={}", self.has_loaded_settings_once),
            format!("refreshing={}", self.is_refreshing_settings),
            format!("applying={}", self.is_applying_settings),
            format!("status={}", self.status_message),
        ];

        let selected_category_lines = self.selected_category_lines();
        summary_lines.extend(selected_category_lines);
        summary_lines
    }

    fn selected_category_lines(&self) -> Vec<String> {
        match self.selected_category {
            SettingsCategory::General => self.general_summary_lines(),
            SettingsCategory::Memory => self.memory_summary_lines(),
            SettingsCategory::Scan => self.scan_summary_lines(),
        }
    }

    fn general_summary_lines(&self) -> Vec<String> {
        let selected_marker = Self::selection_marker(self.selected_field_index, 0);
        vec![format!(
            "{} engine_request_delay_ms={}",
            selected_marker, self.general_settings.engine_request_delay_ms
        )]
    }

    fn memory_summary_lines(&self) -> Vec<String> {
        vec![
            format!(
                "{} memory_type_none={}",
                Self::selection_marker(self.selected_field_index, 0),
                self.memory_settings.memory_type_none
            ),
            format!(
                "{} memory_type_private={}",
                Self::selection_marker(self.selected_field_index, 1),
                self.memory_settings.memory_type_private
            ),
            format!(
                "{} memory_type_image={}",
                Self::selection_marker(self.selected_field_index, 2),
                self.memory_settings.memory_type_image
            ),
            format!(
                "{} memory_type_mapped={}",
                Self::selection_marker(self.selected_field_index, 3),
                self.memory_settings.memory_type_mapped
            ),
            format!(
                "{} required_write={}",
                Self::selection_marker(self.selected_field_index, 4),
                self.memory_settings.required_write
            ),
            format!(
                "{} required_execute={}",
                Self::selection_marker(self.selected_field_index, 5),
                self.memory_settings.required_execute
            ),
            format!(
                "{} required_copy_on_write={}",
                Self::selection_marker(self.selected_field_index, 6),
                self.memory_settings.required_copy_on_write
            ),
            format!(
                "{} excluded_write={}",
                Self::selection_marker(self.selected_field_index, 7),
                self.memory_settings.excluded_write
            ),
            format!(
                "{} excluded_execute={}",
                Self::selection_marker(self.selected_field_index, 8),
                self.memory_settings.excluded_execute
            ),
            format!(
                "{} excluded_copy_on_write={}",
                Self::selection_marker(self.selected_field_index, 9),
                self.memory_settings.excluded_copy_on_write
            ),
            format!(
                "{} start_address=0x{:X}",
                Self::selection_marker(self.selected_field_index, 10),
                self.memory_settings.start_address
            ),
            format!(
                "{} end_address=0x{:X}",
                Self::selection_marker(self.selected_field_index, 11),
                self.memory_settings.end_address
            ),
            format!(
                "{} only_query_usermode={}",
                Self::selection_marker(self.selected_field_index, 12),
                self.memory_settings.only_query_usermode
            ),
        ]
    }

    fn scan_summary_lines(&self) -> Vec<String> {
        vec![
            format!(
                "{} results_page_size={}",
                Self::selection_marker(self.selected_field_index, 0),
                self.scan_settings.results_page_size
            ),
            format!(
                "{} freeze_interval_ms={}",
                Self::selection_marker(self.selected_field_index, 1),
                self.scan_settings.freeze_interval_ms
            ),
            format!(
                "{} project_read_interval_ms={}",
                Self::selection_marker(self.selected_field_index, 2),
                self.scan_settings.project_read_interval_ms
            ),
            format!(
                "{} results_read_interval_ms={}",
                Self::selection_marker(self.selected_field_index, 3),
                self.scan_settings.results_read_interval_ms
            ),
            format!(
                "{} memory_alignment={}",
                Self::selection_marker(self.selected_field_index, 4),
                Self::memory_alignment_label(self.scan_settings.memory_alignment)
            ),
            format!(
                "{} memory_read_mode={}",
                Self::selection_marker(self.selected_field_index, 5),
                Self::memory_read_mode_label(self.scan_settings.memory_read_mode)
            ),
            format!(
                "{} floating_point_tolerance={}",
                Self::selection_marker(self.selected_field_index, 6),
                Self::floating_point_tolerance_label(self.scan_settings.floating_point_tolerance)
            ),
            format!(
                "{} is_single_threaded_scan={}",
                Self::selection_marker(self.selected_field_index, 7),
                self.scan_settings.is_single_threaded_scan
            ),
            format!(
                "{} debug_perform_validation_scan={}",
                Self::selection_marker(self.selected_field_index, 8),
                self.scan_settings.debug_perform_validation_scan
            ),
        ]
    }

    fn field_count_for_selected_category(&self) -> usize {
        match self.selected_category {
            SettingsCategory::General => 1,
            SettingsCategory::Memory => 13,
            SettingsCategory::Scan => 9,
        }
    }

    fn selection_marker(
        selected_field_index: usize,
        field_position: usize,
    ) -> &'static str {
        if selected_field_index == field_position { ">" } else { " " }
    }

    fn step_u64_clamped(
        current_value: u64,
        increase_value: bool,
        step_size: u64,
        minimum_value: u64,
        maximum_value: u64,
    ) -> u64 {
        if increase_value {
            current_value.saturating_add(step_size).min(maximum_value)
        } else {
            current_value.saturating_sub(step_size).max(minimum_value)
        }
    }

    fn step_u32_clamped(
        current_value: u32,
        increase_value: bool,
        step_size: u32,
        minimum_value: u32,
        maximum_value: u32,
    ) -> u32 {
        if increase_value {
            current_value.saturating_add(step_size).min(maximum_value)
        } else {
            current_value.saturating_sub(step_size).max(minimum_value)
        }
    }

    fn next_memory_alignment(
        current_alignment: Option<MemoryAlignment>,
        move_forward: bool,
    ) -> MemoryAlignment {
        let all_alignments = [
            MemoryAlignment::Alignment1,
            MemoryAlignment::Alignment2,
            MemoryAlignment::Alignment4,
            MemoryAlignment::Alignment8,
        ];
        let current_position = current_alignment
            .and_then(|selected_alignment| {
                all_alignments
                    .iter()
                    .position(|alignment| *alignment == selected_alignment)
            })
            .unwrap_or(0);

        let next_position = if move_forward {
            (current_position + 1) % all_alignments.len()
        } else if current_position == 0 {
            all_alignments.len() - 1
        } else {
            current_position - 1
        };

        all_alignments[next_position]
    }

    fn next_memory_read_mode(
        current_mode: MemoryReadMode,
        move_forward: bool,
    ) -> MemoryReadMode {
        let all_modes = [
            MemoryReadMode::Skip,
            MemoryReadMode::ReadBeforeScan,
            MemoryReadMode::ReadInterleavedWithScan,
        ];
        let current_position = all_modes
            .iter()
            .position(|memory_read_mode| *memory_read_mode == current_mode)
            .unwrap_or(0);
        let next_position = if move_forward {
            (current_position + 1) % all_modes.len()
        } else if current_position == 0 {
            all_modes.len() - 1
        } else {
            current_position - 1
        };

        all_modes[next_position]
    }

    fn next_floating_point_tolerance(
        current_tolerance: FloatingPointTolerance,
        move_forward: bool,
    ) -> FloatingPointTolerance {
        let all_tolerances = [
            FloatingPointTolerance::Tolerance10E1,
            FloatingPointTolerance::Tolerance10E2,
            FloatingPointTolerance::Tolerance10E3,
            FloatingPointTolerance::Tolerance10E4,
            FloatingPointTolerance::Tolerance10E5,
            FloatingPointTolerance::ToleranceEpsilon,
        ];
        let current_position = all_tolerances
            .iter()
            .position(|floating_point_tolerance| *floating_point_tolerance == current_tolerance)
            .unwrap_or(0);
        let next_position = if move_forward {
            (current_position + 1) % all_tolerances.len()
        } else if current_position == 0 {
            all_tolerances.len() - 1
        } else {
            current_position - 1
        };

        all_tolerances[next_position]
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
}

impl Default for SettingsPaneState {
    fn default() -> Self {
        Self {
            selected_category: SettingsCategory::General,
            selected_field_index: 0,
            has_pending_changes: false,
            has_loaded_settings_once: false,
            is_refreshing_settings: false,
            is_applying_settings: false,
            general_settings: GeneralSettings::default(),
            memory_settings: MemorySettings::default(),
            scan_settings: ScanSettings::default(),
            status_message: "Ready.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::settings_pane_state::{SettingsCategory, SettingsPaneState};
    use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;

    #[test]
    fn category_cycle_wraps_with_field_reset() {
        let mut settings_pane_state = SettingsPaneState::default();
        settings_pane_state.selected_field_index = 5;

        settings_pane_state.cycle_category_forward();
        assert_eq!(settings_pane_state.selected_category, SettingsCategory::Memory);
        assert_eq!(settings_pane_state.selected_field_index, 0);

        settings_pane_state.cycle_category_backward();
        assert_eq!(settings_pane_state.selected_category, SettingsCategory::General);
        assert_eq!(settings_pane_state.selected_field_index, 0);
    }

    #[test]
    fn toggling_memory_field_marks_pending_changes() {
        let mut settings_pane_state = SettingsPaneState::default();
        settings_pane_state.selected_category = SettingsCategory::Memory;
        settings_pane_state.selected_field_index = 4;
        let original_required_write_setting = settings_pane_state.memory_settings.required_write;

        let did_toggle_value = settings_pane_state.toggle_selected_boolean_field();

        assert!(did_toggle_value);
        assert_ne!(settings_pane_state.memory_settings.required_write, original_required_write_setting);
        assert!(settings_pane_state.has_pending_changes);
    }

    #[test]
    fn cycling_scan_alignment_promotes_none_to_concrete_alignment() {
        let mut settings_pane_state = SettingsPaneState::default();
        settings_pane_state.selected_category = SettingsCategory::Scan;
        settings_pane_state.selected_field_index = 4;
        settings_pane_state.scan_settings.memory_alignment = None;

        let did_cycle_value = settings_pane_state.cycle_selected_enum_field(true);

        assert!(did_cycle_value);
        assert_eq!(settings_pane_state.scan_settings.memory_alignment, Some(MemoryAlignment::Alignment2));
    }
}
