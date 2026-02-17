use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_api::structures::structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData};
use std::path::PathBuf;

/// Tracks where the struct currently being viewed originated from.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StructViewerSource {
    #[default]
    None,
    ScanResults,
    ProjectItems,
}

/// Stores state for viewing and editing selected structures.
#[derive(Clone, Debug)]
pub struct StructViewerPaneState {
    pub selected_struct_name: Option<String>,
    pub selected_field_name: Option<String>,
    pub has_uncommitted_edit: bool,
    pub source: StructViewerSource,
    pub focused_struct: Option<ValuedStruct>,
    pub selected_field_position: Option<usize>,
    pub pending_edit_text: String,
    pub selected_scan_result_refs: Vec<ScanResultRef>,
    pub selected_project_item_paths: Vec<PathBuf>,
    pub is_committing_edit: bool,
    pub status_message: String,
}

impl StructViewerPaneState {
    pub fn clear_focus(
        &mut self,
        status_message: &str,
    ) {
        self.selected_struct_name = None;
        self.selected_field_name = None;
        self.has_uncommitted_edit = false;
        self.source = StructViewerSource::None;
        self.focused_struct = None;
        self.selected_field_position = None;
        self.pending_edit_text.clear();
        self.selected_scan_result_refs.clear();
        self.selected_project_item_paths.clear();
        self.is_committing_edit = false;
        self.status_message = status_message.to_string();
    }

    pub fn focus_scan_results(
        &mut self,
        selected_scan_results: &[ScanResult],
        selected_scan_result_refs: Vec<ScanResultRef>,
    ) {
        if selected_scan_results.is_empty() || selected_scan_result_refs.is_empty() {
            self.clear_focus("No scan result selection is available for struct viewer.");
            return;
        }

        let selected_scan_result_structs = selected_scan_results
            .iter()
            .map(ScanResult::as_valued_struct)
            .collect::<Vec<_>>();
        let combined_struct = ValuedStruct::combine_exclusive(&selected_scan_result_structs);
        self.source = StructViewerSource::ScanResults;
        self.focused_struct = Some(combined_struct);
        self.selected_field_position = self
            .focused_struct
            .as_ref()
            .and_then(|focused_struct| (!focused_struct.get_fields().is_empty()).then_some(0));
        self.selected_struct_name = Some(format!("ScanResultSelection({})", selected_scan_result_refs.len()));
        self.selected_scan_result_refs = selected_scan_result_refs;
        self.selected_project_item_paths.clear();
        self.sync_selected_field_metadata();
        self.status_message = "Focused struct viewer on selected scan result entries.".to_string();
    }

    pub fn focus_project_items(
        &mut self,
        selected_project_items: Vec<(PathBuf, ProjectItem)>,
    ) {
        if selected_project_items.is_empty() {
            self.clear_focus("No project item selection is available for struct viewer.");
            return;
        }

        let selected_project_item_paths = selected_project_items
            .iter()
            .map(|(project_item_path, _)| project_item_path.clone())
            .collect::<Vec<_>>();
        let selected_project_item_structs = selected_project_items
            .iter()
            .map(|(_, project_item)| project_item.get_properties().clone())
            .collect::<Vec<_>>();
        let combined_struct = ValuedStruct::combine_exclusive(&selected_project_item_structs);
        self.source = StructViewerSource::ProjectItems;
        self.focused_struct = Some(combined_struct);
        self.selected_field_position = self
            .focused_struct
            .as_ref()
            .and_then(|focused_struct| (!focused_struct.get_fields().is_empty()).then_some(0));
        self.selected_struct_name = Some(format!("ProjectItemSelection({})", selected_project_item_paths.len()));
        self.selected_project_item_paths = selected_project_item_paths;
        self.selected_scan_result_refs.clear();
        self.sync_selected_field_metadata();
        self.status_message = "Focused struct viewer on selected project item entries.".to_string();
    }

    pub fn select_next_field(&mut self) {
        let Some(focused_struct) = self.focused_struct.as_ref() else {
            self.selected_field_position = None;
            return;
        };
        if focused_struct.get_fields().is_empty() {
            self.selected_field_position = None;
            return;
        }

        let selected_field_position = self.selected_field_position.unwrap_or(0);
        let next_field_position = (selected_field_position + 1) % focused_struct.get_fields().len();
        self.selected_field_position = Some(next_field_position);
        self.sync_selected_field_metadata();
    }

    pub fn select_previous_field(&mut self) {
        let Some(focused_struct) = self.focused_struct.as_ref() else {
            self.selected_field_position = None;
            return;
        };
        if focused_struct.get_fields().is_empty() {
            self.selected_field_position = None;
            return;
        }

        let selected_field_position = self.selected_field_position.unwrap_or(0);
        let previous_field_position = if selected_field_position == 0 {
            focused_struct.get_fields().len() - 1
        } else {
            selected_field_position - 1
        };
        self.selected_field_position = Some(previous_field_position);
        self.sync_selected_field_metadata();
    }

    pub fn append_pending_edit_character(
        &mut self,
        pending_character: char,
    ) {
        if pending_character.is_control() {
            return;
        }

        self.pending_edit_text.push(pending_character);
        self.has_uncommitted_edit = true;
    }

    pub fn backspace_pending_edit(&mut self) {
        self.pending_edit_text.pop();
        self.has_uncommitted_edit = true;
    }

    pub fn clear_pending_edit(&mut self) {
        self.pending_edit_text.clear();
        self.has_uncommitted_edit = true;
    }

    pub fn build_edited_field_from_pending_text(&self) -> Result<ValuedStructField, String> {
        let selected_field = self
            .selected_field()
            .ok_or_else(|| "No struct field is selected.".to_string())?;
        if selected_field.get_is_read_only() {
            return Err(format!("Field '{}' is read-only.", selected_field.get_name()));
        }

        let selected_field_data_value = selected_field
            .get_data_value()
            .ok_or_else(|| "Nested struct edits are not supported in the TUI yet.".to_string())?;
        let pending_edit_text = self.pending_edit_text.trim();
        if pending_edit_text.is_empty() {
            return Err("Edit value is empty.".to_string());
        }

        let symbol_registry = SymbolRegistry::get_instance();
        let selected_data_type_ref = selected_field_data_value.get_data_type_ref();
        let default_edit_format = symbol_registry.get_default_anonymous_value_string_format(selected_data_type_ref);
        let pending_edit_value = AnonymousValueString::new(pending_edit_text.to_string(), default_edit_format, ContainerType::None);
        let edited_data_value = symbol_registry
            .deanonymize_value_string(selected_data_type_ref, &pending_edit_value)
            .map_err(|error| format!("Failed to parse edited value: {}", error))?;

        Ok(ValuedStructField::new(
            selected_field.get_name().to_string(),
            ValuedStructFieldData::Value(edited_data_value),
            false,
        ))
    }

    pub fn apply_committed_field(
        &mut self,
        committed_field: &ValuedStructField,
    ) {
        if let Some(focused_struct) = self.focused_struct.as_mut() {
            focused_struct.set_field_data(
                committed_field.get_name(),
                committed_field.get_field_data().clone(),
                committed_field.get_is_read_only(),
            );
        }
        self.has_uncommitted_edit = false;
        self.sync_selected_field_metadata();
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let mut summary_lines = vec![
            "Actions: r refresh source, Up/Down or j/k select field, Enter commit field edit.".to_string(),
            "Edit mode: type, Backspace, Ctrl+u clear. Editable fields only.".to_string(),
            format!("source={:?}", self.source),
            format!("selected_struct={:?}", self.selected_struct_name),
            format!("field_count={}", self.focused_field_count()),
            format!("selected_field={:?}", self.selected_field_name),
            format!("pending_edit={}", self.pending_edit_text),
            format!("uncommitted_edit={}", self.has_uncommitted_edit),
            format!("selected_scan_results={}", self.selected_scan_result_refs.len()),
            format!("selected_project_items={}", self.selected_project_item_paths.len()),
            format!("committing={}", self.is_committing_edit),
            format!("status={}", self.status_message),
        ];

        let visible_field_count = self.focused_field_count().min(5);
        for field_position in 0..visible_field_count {
            if let Some(focused_field) = self
                .focused_struct
                .as_ref()
                .and_then(|focused_struct| focused_struct.get_fields().get(field_position))
            {
                let selected_marker = if self.selected_field_position == Some(field_position) { ">" } else { " " };
                let writable_marker = if focused_field.get_is_read_only() { "R" } else { "W" };
                summary_lines.push(format!("{} [{}] {}", selected_marker, writable_marker, focused_field.get_name()));
            }
        }

        summary_lines
    }

    fn focused_field_count(&self) -> usize {
        self.focused_struct
            .as_ref()
            .map(|focused_struct| focused_struct.get_fields().len())
            .unwrap_or(0)
    }

    fn selected_field(&self) -> Option<&ValuedStructField> {
        let selected_field_position = self.selected_field_position?;
        self.focused_struct
            .as_ref()
            .and_then(|focused_struct| focused_struct.get_fields().get(selected_field_position))
    }

    fn sync_selected_field_metadata(&mut self) {
        let Some(selected_field_position) = self.selected_field_position else {
            self.selected_field_name = None;
            self.pending_edit_text.clear();
            self.has_uncommitted_edit = false;
            return;
        };

        let Some(focused_struct) = self.focused_struct.as_ref() else {
            self.selected_field_name = None;
            self.pending_edit_text.clear();
            self.has_uncommitted_edit = false;
            return;
        };
        let Some(selected_field) = focused_struct.get_fields().get(selected_field_position) else {
            self.selected_field_name = None;
            self.pending_edit_text.clear();
            self.has_uncommitted_edit = false;
            return;
        };

        let selected_field_name = selected_field.get_name().to_string();
        self.selected_field_name = Some(selected_field_name);
        if self.has_uncommitted_edit {
            return;
        }

        let Some(selected_field_data_value) = selected_field.get_data_value() else {
            self.pending_edit_text.clear();
            return;
        };

        let symbol_registry = SymbolRegistry::get_instance();
        let selected_data_type_ref = selected_field_data_value.get_data_type_ref();
        let default_edit_format = symbol_registry.get_default_anonymous_value_string_format(selected_data_type_ref);
        let default_edit_value = symbol_registry
            .anonymize_value(selected_field_data_value, default_edit_format)
            .map(|anonymous_value_string| anonymous_value_string.get_anonymous_value_string().to_string())
            .unwrap_or_default();
        self.pending_edit_text = default_edit_value;
    }
}

impl Default for StructViewerPaneState {
    fn default() -> Self {
        Self {
            selected_struct_name: None,
            selected_field_name: None,
            has_uncommitted_edit: false,
            source: StructViewerSource::None,
            focused_struct: None,
            selected_field_position: None,
            pending_edit_text: String::new(),
            selected_scan_result_refs: Vec::new(),
            selected_project_item_paths: Vec::new(),
            is_committing_edit: false,
            status_message: "Ready.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::struct_viewer_pane_state::{StructViewerPaneState, StructViewerSource};
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
    use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
    use squalr_engine_api::structures::scan_results::scan_result_valued::ScanResultValued;

    fn create_scan_result(scan_result_global_index: u64) -> ScanResult {
        let scan_result_valued = ScanResultValued::new(
            0x1000 + scan_result_global_index,
            DataTypeRef::new("u8"),
            String::new(),
            None,
            Vec::new(),
            None,
            Vec::new(),
            ScanResultRef::new(scan_result_global_index),
        );

        ScanResult::new(scan_result_valued, String::new(), 0, None, Vec::new(), false)
    }

    #[test]
    fn focus_scan_results_sets_source_and_selection() {
        let mut struct_viewer_pane_state = StructViewerPaneState::default();
        let selected_scan_results = vec![create_scan_result(10)];
        let selected_scan_result_refs = vec![ScanResultRef::new(10)];

        struct_viewer_pane_state.focus_scan_results(&selected_scan_results, selected_scan_result_refs.clone());

        assert_eq!(struct_viewer_pane_state.source, StructViewerSource::ScanResults);
        assert_eq!(struct_viewer_pane_state.selected_scan_result_refs.len(), selected_scan_result_refs.len());
        assert!(struct_viewer_pane_state.selected_field_name.is_some());
    }

    #[test]
    fn select_next_field_wraps_to_first_field() {
        let mut struct_viewer_pane_state = StructViewerPaneState::default();
        let selected_scan_results = vec![create_scan_result(10)];
        let selected_scan_result_refs = vec![ScanResultRef::new(10)];
        struct_viewer_pane_state.focus_scan_results(&selected_scan_results, selected_scan_result_refs);

        let focused_field_count = struct_viewer_pane_state
            .focused_struct
            .as_ref()
            .map(|focused_struct| focused_struct.get_fields().len())
            .unwrap_or(0);
        if focused_field_count > 0 {
            struct_viewer_pane_state.selected_field_position = Some(focused_field_count - 1);
            struct_viewer_pane_state.select_next_field();
            assert_eq!(struct_viewer_pane_state.selected_field_position, Some(0));
        }
    }

    #[test]
    fn append_pending_edit_character_sets_uncommitted_flag() {
        let mut struct_viewer_pane_state = StructViewerPaneState::default();

        struct_viewer_pane_state.append_pending_edit_character('5');

        assert!(struct_viewer_pane_state.has_uncommitted_edit);
        assert_eq!(struct_viewer_pane_state.pending_edit_text, "5");
    }
}
