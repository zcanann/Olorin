use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use squalr_engine_api::structures::scanning::constraints::anonymous_scan_constraint::AnonymousScanConstraint;

/// Stores one editable scanner constraint row.
#[derive(Clone, Debug)]
pub struct ElementScannerConstraintState {
    pub scan_compare_type: ScanCompareType,
    pub scan_value_text: String,
}

impl Default for ElementScannerConstraintState {
    fn default() -> Self {
        Self {
            scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            scan_value_text: "0".to_string(),
        }
    }
}

/// Stores UI state for element scanner controls.
#[derive(Clone, Debug)]
pub struct ElementScannerPaneState {
    pub selected_data_type_index: usize,
    pub constraint_rows: Vec<ElementScannerConstraintState>,
    pub selected_constraint_row_index: usize,
    pub has_pending_scan_request: bool,
    pub has_scan_results: bool,
    pub last_result_count: u64,
    pub last_total_size_in_bytes: u64,
    pub status_message: String,
}

impl ElementScannerPaneState {
    const MAX_CONSTRAINT_COUNT: usize = 5;
    const SUPPORTED_DATA_TYPE_IDS: [&'static str; 10] = [
        "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64",
    ];
    const SUPPORTED_COMPARE_TYPES: [ScanCompareType; 20] = [
        ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
        ScanCompareType::Immediate(ScanCompareTypeImmediate::NotEqual),
        ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThan),
        ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThanOrEqual),
        ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThan),
        ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual),
        ScanCompareType::Relative(ScanCompareTypeRelative::Changed),
        ScanCompareType::Relative(ScanCompareTypeRelative::Unchanged),
        ScanCompareType::Relative(ScanCompareTypeRelative::Increased),
        ScanCompareType::Relative(ScanCompareTypeRelative::Decreased),
        ScanCompareType::Delta(ScanCompareTypeDelta::IncreasedByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::DecreasedByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::MultipliedByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::DividedByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::ModuloByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::ShiftLeftByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::ShiftRightByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalAndByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalOrByX),
        ScanCompareType::Delta(ScanCompareTypeDelta::LogicalXorByX),
    ];

    pub fn selected_data_type_name(&self) -> &'static str {
        Self::SUPPORTED_DATA_TYPE_IDS[self.selected_data_type_index]
    }

    pub fn selected_data_type_ref(&self) -> DataTypeRef {
        DataTypeRef::new(self.selected_data_type_name())
    }

    pub fn active_constraint_count(&self) -> usize {
        self.constraint_rows.len()
    }

    pub fn cycle_data_type_forward(&mut self) {
        self.selected_data_type_index = (self.selected_data_type_index + 1) % Self::SUPPORTED_DATA_TYPE_IDS.len();
    }

    pub fn cycle_data_type_backward(&mut self) {
        self.selected_data_type_index = if self.selected_data_type_index == 0 {
            Self::SUPPORTED_DATA_TYPE_IDS.len() - 1
        } else {
            self.selected_data_type_index - 1
        };
    }

    pub fn select_next_constraint(&mut self) {
        self.selected_constraint_row_index = (self.selected_constraint_row_index + 1) % self.constraint_rows.len();
    }

    pub fn select_previous_constraint(&mut self) {
        self.selected_constraint_row_index = if self.selected_constraint_row_index == 0 {
            self.constraint_rows.len() - 1
        } else {
            self.selected_constraint_row_index - 1
        };
    }

    pub fn add_constraint(&mut self) -> bool {
        if self.constraint_rows.len() >= Self::MAX_CONSTRAINT_COUNT {
            return false;
        }

        self.constraint_rows.push(if self.constraint_rows.len() == 1 {
            ElementScannerConstraintState {
                scan_compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::LessThanOrEqual),
                ..ElementScannerConstraintState::default()
            }
        } else {
            ElementScannerConstraintState::default()
        });
        self.selected_constraint_row_index = self.constraint_rows.len() - 1;
        true
    }

    pub fn remove_selected_constraint(&mut self) -> bool {
        if self.constraint_rows.len() <= 1 {
            return false;
        }

        self.constraint_rows.remove(self.selected_constraint_row_index);
        if self.selected_constraint_row_index >= self.constraint_rows.len() {
            self.selected_constraint_row_index = self.constraint_rows.len() - 1;
        }
        true
    }

    pub fn cycle_selected_constraint_compare_type_forward(&mut self) {
        let selected_compare_type = self.constraint_rows[self.selected_constraint_row_index].scan_compare_type;
        let current_compare_type_index = Self::SUPPORTED_COMPARE_TYPES
            .iter()
            .position(|compare_type_candidate| *compare_type_candidate == selected_compare_type)
            .unwrap_or(0);
        let next_compare_type_index = (current_compare_type_index + 1) % Self::SUPPORTED_COMPARE_TYPES.len();
        self.constraint_rows[self.selected_constraint_row_index].scan_compare_type = Self::SUPPORTED_COMPARE_TYPES[next_compare_type_index];
    }

    pub fn cycle_selected_constraint_compare_type_backward(&mut self) {
        let selected_compare_type = self.constraint_rows[self.selected_constraint_row_index].scan_compare_type;
        let current_compare_type_index = Self::SUPPORTED_COMPARE_TYPES
            .iter()
            .position(|compare_type_candidate| *compare_type_candidate == selected_compare_type)
            .unwrap_or(0);
        let previous_compare_type_index = if current_compare_type_index == 0 {
            Self::SUPPORTED_COMPARE_TYPES.len() - 1
        } else {
            current_compare_type_index - 1
        };
        self.constraint_rows[self.selected_constraint_row_index].scan_compare_type = Self::SUPPORTED_COMPARE_TYPES[previous_compare_type_index];
    }

    pub fn append_selected_constraint_value_character(
        &mut self,
        value_character: char,
    ) {
        if !Self::is_supported_value_character(value_character) {
            return;
        }

        self.constraint_rows[self.selected_constraint_row_index]
            .scan_value_text
            .push(value_character);
    }

    pub fn backspace_selected_constraint_value(&mut self) {
        let selected_scan_value = &mut self.constraint_rows[self.selected_constraint_row_index].scan_value_text;
        selected_scan_value.pop();

        if selected_scan_value.is_empty() {
            selected_scan_value.push('0');
        }
    }

    pub fn clear_selected_constraint_value(&mut self) {
        self.constraint_rows[self.selected_constraint_row_index].scan_value_text = "0".to_string();
    }

    pub fn build_anonymous_scan_constraints(&self) -> Vec<AnonymousScanConstraint> {
        self.constraint_rows
            .iter()
            .map(|constraint_row| {
                let should_include_value = !matches!(constraint_row.scan_compare_type, ScanCompareType::Relative(_));
                let anonymous_value_string = if should_include_value {
                    Some(AnonymousValueString::new(
                        constraint_row.scan_value_text.clone(),
                        AnonymousValueStringFormat::Decimal,
                        ContainerType::None,
                    ))
                } else {
                    None
                };

                AnonymousScanConstraint::new(constraint_row.scan_compare_type, anonymous_value_string)
            })
            .collect()
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let mut summary_lines = vec![
            "Actions: s start, n reset/new, c collect, t/T data type, a add, x remove.".to_string(),
            "Constraint edit: j/k select, m/M compare type, digits/-/. append, Backspace, Ctrl+u clear.".to_string(),
            format!("data_type={}", self.selected_data_type_name()),
            format!("constraints={}", self.active_constraint_count()),
            format!("selected_constraint={}", self.selected_constraint_row_index + 1),
            format!("pending_scan={}", self.has_pending_scan_request),
            format!("has_results={}", self.has_scan_results),
            format!("last_result_count={}", self.last_result_count),
            format!("last_total_bytes={}", self.last_total_size_in_bytes),
            format!("status={}", self.status_message),
        ];

        for (constraint_row_index, constraint_row) in self.constraint_rows.iter().enumerate() {
            let selected_marker = if self.selected_constraint_row_index == constraint_row_index {
                ">"
            } else {
                " "
            };
            summary_lines.push(format!(
                "{} {} {}",
                selected_marker,
                Self::scan_compare_type_label(constraint_row.scan_compare_type),
                constraint_row.scan_value_text
            ));
        }

        summary_lines
    }

    fn is_supported_value_character(value_character: char) -> bool {
        value_character.is_ascii_digit() || value_character == '-' || value_character == '.'
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
}

impl Default for ElementScannerPaneState {
    fn default() -> Self {
        Self {
            selected_data_type_index: 2,
            constraint_rows: vec![ElementScannerConstraintState::default()],
            selected_constraint_row_index: 0,
            has_pending_scan_request: false,
            has_scan_results: false,
            last_result_count: 0,
            last_total_size_in_bytes: 0,
            status_message: "Ready.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::element_scanner_pane_state::ElementScannerPaneState;
    use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
    use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;

    #[test]
    fn add_constraint_caps_at_five() {
        let mut element_scanner_pane_state = ElementScannerPaneState::default();

        assert!(element_scanner_pane_state.add_constraint());
        assert!(element_scanner_pane_state.add_constraint());
        assert!(element_scanner_pane_state.add_constraint());
        assert!(element_scanner_pane_state.add_constraint());
        assert!(!element_scanner_pane_state.add_constraint());
        assert_eq!(element_scanner_pane_state.active_constraint_count(), 5);
    }

    #[test]
    fn remove_constraint_retains_at_least_one_row() {
        let mut element_scanner_pane_state = ElementScannerPaneState::default();

        assert!(!element_scanner_pane_state.remove_selected_constraint());
        assert_eq!(element_scanner_pane_state.active_constraint_count(), 1);
    }

    #[test]
    fn data_type_cycle_wraps() {
        let mut element_scanner_pane_state = ElementScannerPaneState::default();
        element_scanner_pane_state.selected_data_type_index = 0;

        element_scanner_pane_state.cycle_data_type_backward();
        assert_eq!(element_scanner_pane_state.selected_data_type_name(), "f64");

        element_scanner_pane_state.cycle_data_type_forward();
        assert_eq!(element_scanner_pane_state.selected_data_type_name(), "i8");
    }

    #[test]
    fn relative_constraint_serializes_without_value() {
        let mut element_scanner_pane_state = ElementScannerPaneState::default();
        element_scanner_pane_state.constraint_rows[0].scan_compare_type = ScanCompareType::Relative(ScanCompareTypeRelative::Changed);

        let anonymous_scan_constraints = element_scanner_pane_state.build_anonymous_scan_constraints();

        assert_eq!(anonymous_scan_constraints.len(), 1);
        assert!(
            anonymous_scan_constraints[0]
                .get_anonymous_value_string()
                .is_none()
        );
    }
}
