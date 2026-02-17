use crate::views::settings::summary::build_settings_summary_lines_with_capacity;
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

    pub fn summary_lines_with_capacity(
        &self,
        line_capacity: usize,
    ) -> Vec<String> {
        build_settings_summary_lines_with_capacity(self, line_capacity)
    }

    fn field_count_for_selected_category(&self) -> usize {
        match self.selected_category {
            SettingsCategory::General => 1,
            SettingsCategory::Memory => 13,
            SettingsCategory::Scan => 9,
        }
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
