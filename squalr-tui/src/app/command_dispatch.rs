use super::app_shell::AppShell;
use crate::views::project_explorer_pane_state::ProjectSelectorInputMode;
use crate::views::settings_pane_state::SettingsCategory;
use crate::views::struct_viewer_pane_state::StructViewerSource;
use anyhow::Result;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::process::list::process_list_request::ProcessListRequest;
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::commands::project::close::project_close_request::ProjectCloseRequest;
use squalr_engine_api::commands::project::create::project_create_request::ProjectCreateRequest;
use squalr_engine_api::commands::project::delete::project_delete_request::ProjectDeleteRequest;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::add::project_items_add_request::ProjectItemsAddRequest;
use squalr_engine_api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use squalr_engine_api::commands::project_items::delete::project_items_delete_request::ProjectItemsDeleteRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::move_item::project_items_move_request::ProjectItemsMoveRequest;
use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::element_scan::element_scan_request::ElementScanRequest;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::reset::scan_reset_request::ScanResetRequest;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use squalr_engine_api::commands::settings::general::list::general_settings_list_request::GeneralSettingsListRequest;
use squalr_engine_api::commands::settings::general::set::general_settings_set_request::GeneralSettingsSetRequest;
use squalr_engine_api::commands::settings::memory::list::memory_settings_list_request::MemorySettingsListRequest;
use squalr_engine_api::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use squalr_engine_api::commands::settings::scan::set::scan_settings_set_request::ScanSettingsSetRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_engine_api::structures::structs::valued_struct_field::ValuedStructField;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

impl AppShell {
    pub(super) fn refresh_output_log_history(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        self.refresh_output_log_history_with_feedback(squalr_engine, false);
    }

    pub(super) fn refresh_output_log_history_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.output_pane_state.status_message = "No unprivileged engine state is available for output logs.".to_string();
                return;
            }
        };

        let log_history_guard = match engine_unprivileged_state.get_logger().get_log_history().read() {
            Ok(log_history_guard) => log_history_guard,
            Err(lock_error) => {
                self.app_state.output_pane_state.status_message = format!("Failed to lock output log history: {}", lock_error);
                return;
            }
        };
        let log_history_snapshot = log_history_guard.iter().cloned().collect();
        self.app_state
            .output_pane_state
            .apply_log_history_with_feedback(log_history_snapshot, should_update_status_message);
    }

    pub(super) fn refresh_all_settings_categories_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        if self.app_state.settings_pane_state.is_refreshing_settings {
            self.app_state.settings_pane_state.status_message = "Settings refresh is already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.settings_pane_state.status_message = "No unprivileged engine state is available for settings refresh.".to_string();
                return;
            }
        };

        self.app_state.settings_pane_state.is_refreshing_settings = true;
        let mut did_read_general_settings = false;
        let mut did_read_memory_settings = false;
        let mut did_read_scan_settings = false;

        let general_settings_list_request = GeneralSettingsListRequest {};
        let (general_response_sender, general_response_receiver) = mpsc::sync_channel(1);
        general_settings_list_request.send(engine_unprivileged_state, move |general_settings_list_response| {
            let _ = general_response_sender.send(general_settings_list_response);
        });

        match general_response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(general_settings_list_response) => {
                if let Ok(general_settings) = general_settings_list_response.general_settings {
                    self.app_state
                        .settings_pane_state
                        .apply_general_settings(general_settings);
                    did_read_general_settings = true;
                } else {
                    self.app_state.settings_pane_state.status_message = "Failed to read general settings.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.settings_pane_state.status_message = format!("Timed out waiting for general settings response: {}", receive_error);
            }
        }

        let memory_settings_list_request = MemorySettingsListRequest {};
        let (memory_response_sender, memory_response_receiver) = mpsc::sync_channel(1);
        memory_settings_list_request.send(engine_unprivileged_state, move |memory_settings_list_response| {
            let _ = memory_response_sender.send(memory_settings_list_response);
        });

        match memory_response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(memory_settings_list_response) => {
                if let Ok(memory_settings) = memory_settings_list_response.memory_settings {
                    self.app_state
                        .settings_pane_state
                        .apply_memory_settings(memory_settings);
                    did_read_memory_settings = true;
                } else {
                    self.app_state.settings_pane_state.status_message = "Failed to read memory settings.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.settings_pane_state.status_message = format!("Timed out waiting for memory settings response: {}", receive_error);
            }
        }

        let scan_settings_list_request = ScanSettingsListRequest {};
        let (scan_response_sender, scan_response_receiver) = mpsc::sync_channel(1);
        scan_settings_list_request.send(engine_unprivileged_state, move |scan_settings_list_response| {
            let _ = scan_response_sender.send(scan_settings_list_response);
        });

        match scan_response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_settings_list_response) => {
                if let Ok(scan_settings) = scan_settings_list_response.scan_settings {
                    self.app_state
                        .settings_pane_state
                        .apply_scan_settings(scan_settings);
                    did_read_scan_settings = true;
                    if should_update_status_message {
                        self.app_state.settings_pane_state.status_message = "Settings refreshed.".to_string();
                    }
                } else {
                    self.app_state.settings_pane_state.status_message = "Failed to read scan settings.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.settings_pane_state.status_message = format!("Timed out waiting for scan settings response: {}", receive_error);
            }
        }

        if did_read_general_settings && did_read_memory_settings && did_read_scan_settings {
            self.app_state.settings_pane_state.has_loaded_settings_once = true;
        }

        self.app_state.settings_pane_state.is_refreshing_settings = false;
    }

    pub(super) fn apply_selected_settings_category(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.settings_pane_state.is_applying_settings {
            self.app_state.settings_pane_state.status_message = "Settings update is already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.settings_pane_state.status_message = "No unprivileged engine state is available for settings update.".to_string();
                return;
            }
        };

        self.app_state.settings_pane_state.is_applying_settings = true;
        let selected_settings_category = self.app_state.settings_pane_state.selected_category;

        match selected_settings_category {
            SettingsCategory::General => {
                let general_settings_set_request = GeneralSettingsSetRequest {
                    engine_request_delay: Some(
                        self.app_state
                            .settings_pane_state
                            .general_settings
                            .engine_request_delay_ms,
                    ),
                };
                let (response_sender, response_receiver) = mpsc::sync_channel(1);
                general_settings_set_request.send(engine_unprivileged_state, move |general_settings_set_response| {
                    let _ = response_sender.send(general_settings_set_response);
                });

                match response_receiver.recv_timeout(Duration::from_secs(3)) {
                    Ok(_general_settings_set_response) => {
                        self.app_state.settings_pane_state.has_pending_changes = false;
                        self.app_state.settings_pane_state.status_message = "Applied general settings.".to_string();
                    }
                    Err(receive_error) => {
                        self.app_state.settings_pane_state.status_message = format!("Timed out waiting for general settings set response: {}", receive_error);
                    }
                }
            }
            SettingsCategory::Memory => {
                let memory_settings = self.app_state.settings_pane_state.memory_settings;
                let memory_settings_set_request = MemorySettingsSetRequest {
                    memory_type_none: Some(memory_settings.memory_type_none),
                    memory_type_private: Some(memory_settings.memory_type_private),
                    memory_type_image: Some(memory_settings.memory_type_image),
                    memory_type_mapped: Some(memory_settings.memory_type_mapped),
                    required_write: Some(memory_settings.required_write),
                    required_execute: Some(memory_settings.required_execute),
                    required_copy_on_write: Some(memory_settings.required_copy_on_write),
                    excluded_write: Some(memory_settings.excluded_write),
                    excluded_execute: Some(memory_settings.excluded_execute),
                    excluded_copy_on_write: Some(memory_settings.excluded_copy_on_write),
                    start_address: Some(memory_settings.start_address),
                    end_address: Some(memory_settings.end_address),
                    only_query_usermode: Some(memory_settings.only_query_usermode),
                };
                let (response_sender, response_receiver) = mpsc::sync_channel(1);
                memory_settings_set_request.send(engine_unprivileged_state, move |memory_settings_set_response| {
                    let _ = response_sender.send(memory_settings_set_response);
                });

                match response_receiver.recv_timeout(Duration::from_secs(3)) {
                    Ok(_memory_settings_set_response) => {
                        self.app_state.settings_pane_state.has_pending_changes = false;
                        self.app_state.settings_pane_state.status_message = "Applied memory settings.".to_string();
                    }
                    Err(receive_error) => {
                        self.app_state.settings_pane_state.status_message = format!("Timed out waiting for memory settings set response: {}", receive_error);
                    }
                }
            }
            SettingsCategory::Scan => {
                let scan_settings = self.app_state.settings_pane_state.scan_settings;
                let scan_settings_set_request = ScanSettingsSetRequest {
                    results_page_size: Some(scan_settings.results_page_size),
                    results_read_interval_ms: Some(scan_settings.results_read_interval_ms),
                    project_read_interval_ms: Some(scan_settings.project_read_interval_ms),
                    freeze_interval_ms: Some(scan_settings.freeze_interval_ms),
                    memory_alignment: scan_settings.memory_alignment,
                    memory_read_mode: Some(scan_settings.memory_read_mode),
                    floating_point_tolerance: Some(scan_settings.floating_point_tolerance),
                    is_single_threaded_scan: Some(scan_settings.is_single_threaded_scan),
                    debug_perform_validation_scan: Some(scan_settings.debug_perform_validation_scan),
                };
                let (response_sender, response_receiver) = mpsc::sync_channel(1);
                scan_settings_set_request.send(engine_unprivileged_state, move |scan_settings_set_response| {
                    let _ = response_sender.send(scan_settings_set_response);
                });

                match response_receiver.recv_timeout(Duration::from_secs(3)) {
                    Ok(_scan_settings_set_response) => {
                        self.app_state.settings_pane_state.has_pending_changes = false;
                        self.app_state.settings_pane_state.status_message = "Applied scan settings.".to_string();
                    }
                    Err(receive_error) => {
                        self.app_state.settings_pane_state.status_message = format!("Timed out waiting for scan settings set response: {}", receive_error);
                    }
                }
            }
        }

        self.app_state.settings_pane_state.is_applying_settings = false;
    }

    pub(super) fn refresh_struct_viewer_focus_from_source(&mut self) {
        match self.app_state.struct_viewer_pane_state.source {
            StructViewerSource::None => {
                self.app_state.struct_viewer_pane_state.status_message = "No struct viewer source is selected.".to_string();
            }
            StructViewerSource::ScanResults => self.sync_struct_viewer_focus_from_scan_results(),
            StructViewerSource::ProjectItems => self.sync_struct_viewer_focus_from_project_items(),
        }
    }

    pub(super) fn sync_struct_viewer_focus_from_scan_results(&mut self) {
        let selected_scan_results = self.app_state.scan_results_pane_state.selected_scan_results();
        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        self.app_state
            .struct_viewer_pane_state
            .focus_scan_results(&selected_scan_results, selected_scan_result_refs);
    }

    pub(super) fn sync_struct_viewer_focus_from_project_items(&mut self) {
        let selected_project_items = self
            .app_state
            .project_explorer_pane_state
            .selected_project_items_for_struct_viewer();
        self.app_state
            .struct_viewer_pane_state
            .focus_project_items(selected_project_items);
    }

    pub(super) fn commit_struct_viewer_field_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.struct_viewer_pane_state.is_committing_edit {
            self.app_state.struct_viewer_pane_state.status_message = "Struct field edit is already in progress.".to_string();
            return;
        }

        let edited_field = match self
            .app_state
            .struct_viewer_pane_state
            .build_edited_field_from_pending_text()
        {
            Ok(edited_field) => edited_field,
            Err(error) => {
                self.app_state.struct_viewer_pane_state.status_message = error;
                return;
            }
        };

        self.app_state.struct_viewer_pane_state.is_committing_edit = true;
        self.app_state.struct_viewer_pane_state.status_message = format!("Committing field '{}'.", edited_field.get_name());

        match self.app_state.struct_viewer_pane_state.source {
            StructViewerSource::None => {
                self.app_state.struct_viewer_pane_state.status_message = "No struct viewer source is selected for commit.".to_string();
            }
            StructViewerSource::ScanResults => self.commit_scan_result_struct_field_edit(squalr_engine, edited_field),
            StructViewerSource::ProjectItems => self.commit_project_item_struct_field_edit(squalr_engine, edited_field),
        }

        self.app_state.struct_viewer_pane_state.is_committing_edit = false;
    }

    pub(super) fn commit_scan_result_struct_field_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        edited_field: ValuedStructField,
    ) {
        let selected_scan_result_refs = self
            .app_state
            .struct_viewer_pane_state
            .selected_scan_result_refs
            .clone();
        if selected_scan_result_refs.is_empty() {
            self.app_state.struct_viewer_pane_state.status_message = "No scan results are selected for struct edit commit.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.struct_viewer_pane_state.status_message = "No unprivileged engine state is available for scan result struct edits.".to_string();
                return;
            }
        };

        if edited_field.get_name() == ScanResult::PROPERTY_NAME_IS_FROZEN {
            let target_frozen_state = edited_field
                .get_data_value()
                .map(|edited_data_value| {
                    edited_data_value
                        .get_value_bytes()
                        .iter()
                        .any(|edited_value_byte| *edited_value_byte != 0)
                })
                .unwrap_or(false);

            let scan_results_freeze_request = ScanResultsFreezeRequest {
                scan_result_refs: selected_scan_result_refs,
                is_frozen: target_frozen_state,
            };
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            let request_dispatched = scan_results_freeze_request.send(engine_unprivileged_state, move |scan_results_freeze_response| {
                let _ = response_sender.send(scan_results_freeze_response);
            });

            if !request_dispatched {
                self.app_state.struct_viewer_pane_state.status_message = "Failed to dispatch scan result freeze request from struct viewer.".to_string();
                return;
            }

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(scan_results_freeze_response) => {
                    if scan_results_freeze_response
                        .failed_freeze_toggle_scan_result_refs
                        .is_empty()
                    {
                        self.app_state.struct_viewer_pane_state.status_message = if target_frozen_state {
                            "Committed frozen state from struct viewer.".to_string()
                        } else {
                            "Committed unfrozen state from struct viewer.".to_string()
                        };
                    } else {
                        self.app_state.struct_viewer_pane_state.status_message = format!(
                            "Freeze commit partially failed for {} scan results.",
                            scan_results_freeze_response
                                .failed_freeze_toggle_scan_result_refs
                                .len()
                        );
                    }
                    self.refresh_scan_results_page(squalr_engine);
                    self.sync_struct_viewer_focus_from_scan_results();
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for scan result freeze response: {}", receive_error);
                }
            }
            return;
        }

        let scan_results_set_property_request = match Self::build_scan_results_set_property_request_for_struct_edit(selected_scan_result_refs, &edited_field) {
            Ok(scan_results_set_property_request) => scan_results_set_property_request,
            Err(error) => {
                self.app_state.struct_viewer_pane_state.status_message = error;
                return;
            }
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_set_property_request.send(engine_unprivileged_state, move |scan_results_set_property_response| {
            let _ = response_sender.send(scan_results_set_property_response);
        });

        if !request_dispatched {
            self.app_state.struct_viewer_pane_state.status_message = "Failed to dispatch scan result property request from struct viewer.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_scan_results_set_property_response) => {
                self.app_state.struct_viewer_pane_state.status_message =
                    format!("Committed scan result field '{}' from struct viewer.", edited_field.get_name());
                self.refresh_scan_results_page(squalr_engine);
                self.sync_struct_viewer_focus_from_scan_results();
            }
            Err(receive_error) => {
                self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for scan result property response: {}", receive_error);
            }
        }
    }

    pub(super) fn commit_project_item_struct_field_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        edited_field: ValuedStructField,
    ) {
        let selected_project_item_paths = self
            .app_state
            .struct_viewer_pane_state
            .selected_project_item_paths
            .clone();
        if selected_project_item_paths.is_empty() {
            self.app_state.struct_viewer_pane_state.status_message = "No project items are selected for struct edit commit.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.struct_viewer_pane_state.status_message = "No unprivileged engine state is available for project item struct edits.".to_string();
                return;
            }
        };

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let edited_field_name = edited_field.get_name().to_string();
        let edited_name = if edited_field_name == ProjectItem::PROPERTY_NAME {
            Self::extract_string_value_from_edited_field(&edited_field)
        } else {
            None
        };

        let mut pending_memory_write_requests = Vec::new();
        let mut pending_rename_requests = Vec::new();
        let mut has_persisted_property_edit = false;
        let mut opened_project_write_guard = match opened_project_lock.write() {
            Ok(opened_project_write_guard) => opened_project_write_guard,
            Err(error) => {
                self.app_state.struct_viewer_pane_state.status_message = format!("Failed to acquire opened project lock for struct edit: {}", error);
                return;
            }
        };
        let opened_project = match opened_project_write_guard.as_mut() {
            Some(opened_project) => opened_project,
            None => {
                self.app_state.struct_viewer_pane_state.status_message = "Cannot apply struct edit because no project is currently open.".to_string();
                return;
            }
        };
        let root_project_item_path = opened_project
            .get_project_root_ref()
            .get_project_item_path()
            .clone();

        for selected_project_item_path in &selected_project_item_paths {
            if edited_field_name == ProjectItem::PROPERTY_NAME && selected_project_item_path == &root_project_item_path {
                continue;
            }

            let project_item_ref = ProjectItemRef::new(selected_project_item_path.clone());
            let selected_project_item = match opened_project.get_project_item_mut(&project_item_ref) {
                Some(selected_project_item) => selected_project_item,
                None => continue,
            };
            let project_item_type_id = selected_project_item
                .get_item_type()
                .get_project_item_type_id()
                .to_string();
            let should_apply_edited_field = Self::should_apply_struct_field_edit_to_project_item(&project_item_type_id, &edited_field_name);

            if should_apply_edited_field {
                selected_project_item.get_properties_mut().set_field_data(
                    edited_field.get_name(),
                    edited_field.get_field_data().clone(),
                    edited_field.get_is_read_only(),
                );
                selected_project_item.set_has_unsaved_changes(true);
                has_persisted_property_edit = true;
            }

            if let Some(edited_name) = &edited_name {
                if let Some(project_items_rename_request) =
                    Self::build_project_item_rename_request(selected_project_item_path, &project_item_type_id, edited_name)
                {
                    pending_rename_requests.push(project_items_rename_request);
                }
            }

            if let Some(memory_write_request) = Self::build_memory_write_request_for_project_item_edit(selected_project_item, &edited_field) {
                pending_memory_write_requests.push(memory_write_request);
            }
        }

        if !has_persisted_property_edit && pending_rename_requests.is_empty() && pending_memory_write_requests.is_empty() {
            self.app_state.struct_viewer_pane_state.status_message = "Selected project item field cannot be committed through TUI struct routing.".to_string();
            return;
        }

        drop(opened_project_write_guard);

        if has_persisted_property_edit {
            if let Ok(mut opened_project_write_guard) = opened_project_lock.write() {
                if let Some(opened_project) = opened_project_write_guard.as_mut() {
                    opened_project
                        .get_project_info_mut()
                        .set_has_unsaved_changes(true);
                }
            }

            let project_save_request = ProjectSaveRequest {};
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            project_save_request.send(engine_unprivileged_state, move |project_save_response| {
                let _ = response_sender.send(project_save_response);
            });

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(project_save_response) => {
                    if !project_save_response.success {
                        self.app_state.struct_viewer_pane_state.status_message = "Project save failed while committing project item struct field.".to_string();
                        return;
                    }
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for project save response: {}", receive_error);
                    return;
                }
            }

            project_manager.notify_project_items_changed();
        }

        for pending_rename_request in pending_rename_requests {
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            pending_rename_request.send(engine_unprivileged_state, move |project_items_rename_response| {
                let _ = response_sender.send(project_items_rename_response);
            });

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(project_items_rename_response) => {
                    if !project_items_rename_response.success {
                        self.app_state.struct_viewer_pane_state.status_message = "Project item rename failed during struct edit commit.".to_string();
                        return;
                    }
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for project item rename response: {}", receive_error);
                    return;
                }
            }
        }

        for pending_memory_write_request in pending_memory_write_requests {
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            let request_dispatched = pending_memory_write_request.send(engine_unprivileged_state, move |memory_write_response| {
                let _ = response_sender.send(memory_write_response);
            });
            if !request_dispatched {
                self.app_state.struct_viewer_pane_state.status_message = "Failed to dispatch memory write request during struct edit commit.".to_string();
                return;
            }

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(memory_write_response) => {
                    if !memory_write_response.success {
                        self.app_state.struct_viewer_pane_state.status_message = "Memory write failed during project item struct edit commit.".to_string();
                        return;
                    }
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for memory write response: {}", receive_error);
                    return;
                }
            }
        }

        self.app_state
            .struct_viewer_pane_state
            .apply_committed_field(&edited_field);
        self.app_state.struct_viewer_pane_state.status_message = format!("Committed project item field '{}' from struct viewer.", edited_field.get_name());
        self.refresh_project_items_list(squalr_engine);
        self.sync_struct_viewer_focus_from_project_items();
    }

    pub(super) fn commit_project_selector_input(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        match self.app_state.project_explorer_pane_state.input_mode {
            ProjectSelectorInputMode::CreatingProject => self.create_project_from_pending_name(squalr_engine),
            ProjectSelectorInputMode::RenamingProject => self.rename_selected_project_from_pending_name(squalr_engine),
            ProjectSelectorInputMode::CreatingProjectDirectory => self.create_project_directory_from_pending_name(squalr_engine),
            ProjectSelectorInputMode::None => {}
        }
    }

    pub(super) fn reset_scan_state(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .element_scanner_pane_state
            .has_pending_scan_request
        {
            self.app_state.element_scanner_pane_state.status_message = "Scan request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.element_scanner_pane_state.status_message = "No unprivileged engine state is available for scan reset.".to_string();
                return;
            }
        };

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = true;
        self.app_state.element_scanner_pane_state.status_message = "Resetting active scan.".to_string();

        let scan_reset_request = ScanResetRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_reset_request.send(engine_unprivileged_state, move |scan_reset_response| {
            let _ = response_sender.send(scan_reset_response);
        });

        if !request_dispatched {
            self.app_state
                .element_scanner_pane_state
                .has_pending_scan_request = false;
            self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch scan reset request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_reset_response) => {
                if scan_reset_response.success {
                    self.app_state.element_scanner_pane_state.has_scan_results = false;
                    self.app_state.element_scanner_pane_state.last_result_count = 0;
                    self.app_state
                        .element_scanner_pane_state
                        .last_total_size_in_bytes = 0;
                    self.app_state.scan_results_pane_state.clear_results();
                    self.app_state.element_scanner_pane_state.status_message = "Scan state reset.".to_string();
                } else {
                    self.app_state.element_scanner_pane_state.status_message = "Scan reset request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for scan reset response: {}", receive_error);
            }
        }

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = false;
    }

    pub(super) fn collect_scan_values(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .element_scanner_pane_state
            .has_pending_scan_request
        {
            self.app_state.element_scanner_pane_state.status_message = "Scan request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.element_scanner_pane_state.status_message = "No unprivileged engine state is available for value collection.".to_string();
                return;
            }
        };

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = true;
        self.app_state.element_scanner_pane_state.status_message = "Collecting scan values.".to_string();

        let scan_collect_values_request = ScanCollectValuesRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_collect_values_request.send(engine_unprivileged_state, move |scan_collect_values_response| {
            let _ = response_sender.send(scan_collect_values_response);
        });

        if !request_dispatched {
            self.app_state
                .element_scanner_pane_state
                .has_pending_scan_request = false;
            self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch scan collect values request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_collect_values_response) => {
                self.app_state.element_scanner_pane_state.last_result_count = scan_collect_values_response.scan_results_metadata.result_count;
                self.app_state
                    .element_scanner_pane_state
                    .last_total_size_in_bytes = scan_collect_values_response
                    .scan_results_metadata
                    .total_size_in_bytes;
                self.app_state.element_scanner_pane_state.status_message = format!(
                    "Collected values for {} results.",
                    scan_collect_values_response.scan_results_metadata.result_count
                );
            }
            Err(receive_error) => {
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for collect values response: {}", receive_error);
            }
        }

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = false;
    }

    pub(super) fn start_element_scan(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .element_scanner_pane_state
            .has_pending_scan_request
        {
            self.app_state.element_scanner_pane_state.status_message = "Scan request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.element_scanner_pane_state.status_message = "No unprivileged engine state is available for element scanning.".to_string();
                return;
            }
        };

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = true;
        self.app_state.element_scanner_pane_state.status_message = "Starting scan.".to_string();

        if !self.app_state.element_scanner_pane_state.has_scan_results {
            let scan_new_request = ScanNewRequest {};
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            let request_dispatched = scan_new_request.send(engine_unprivileged_state, move |scan_new_response| {
                let _ = response_sender.send(scan_new_response);
            });

            if !request_dispatched {
                self.app_state
                    .element_scanner_pane_state
                    .has_pending_scan_request = false;
                self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch new scan request.".to_string();
                return;
            }

            if let Err(receive_error) = response_receiver.recv_timeout(Duration::from_secs(3)) {
                self.app_state
                    .element_scanner_pane_state
                    .has_pending_scan_request = false;
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for new scan response: {}", receive_error);
                return;
            }
        }

        let element_scan_request = ElementScanRequest {
            scan_constraints: self
                .app_state
                .element_scanner_pane_state
                .build_anonymous_scan_constraints(),
            data_type_refs: vec![
                self.app_state
                    .element_scanner_pane_state
                    .selected_data_type_ref(),
            ],
        };

        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = element_scan_request.send(engine_unprivileged_state, move |element_scan_response| {
            let _ = response_sender.send(element_scan_response);
        });

        if !request_dispatched {
            self.app_state
                .element_scanner_pane_state
                .has_pending_scan_request = false;
            self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch element scan request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(element_scan_response) => {
                self.app_state.element_scanner_pane_state.has_scan_results = true;
                self.app_state.element_scanner_pane_state.last_result_count = element_scan_response.scan_results_metadata.result_count;
                self.app_state
                    .element_scanner_pane_state
                    .last_total_size_in_bytes = element_scan_response.scan_results_metadata.total_size_in_bytes;
                self.app_state.element_scanner_pane_state.status_message =
                    format!("Scan complete with {} results.", element_scan_response.scan_results_metadata.result_count);
                self.query_scan_results_current_page(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for element scan response: {}", receive_error);
            }
        }

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = false;
    }

    pub(super) fn query_scan_results_current_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let _ = self.query_scan_results_current_page_with_feedback(squalr_engine, true);
    }

    pub(super) fn query_scan_results_current_page_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) -> bool {
        if self.app_state.scan_results_pane_state.is_querying_scan_results {
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "Scan results query already in progress.".to_string();
            }
            return false;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for scan results query.".to_string();
                }
                return false;
            }
        };

        self.app_state.scan_results_pane_state.is_querying_scan_results = true;
        if should_update_status_message {
            self.app_state.scan_results_pane_state.status_message =
                format!("Querying scan results page {}.", self.app_state.scan_results_pane_state.current_page_index);
        }

        let page_index = self.app_state.scan_results_pane_state.current_page_index;
        let scan_results_query_request = ScanResultsQueryRequest { page_index };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_query_request.send(engine_unprivileged_state, move |scan_results_query_response| {
            let _ = response_sender.send(scan_results_query_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_querying_scan_results = false;
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results query request.".to_string();
            }
            return false;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_results_query_response) => {
                self.apply_scan_results_query_response(scan_results_query_response, should_update_status_message);
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results query response: {}", receive_error);
                }
            }
        }

        self.app_state.scan_results_pane_state.is_querying_scan_results = false;
        true
    }

    pub(super) fn query_next_scan_results_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let current_page_index = self.app_state.scan_results_pane_state.current_page_index;
        let target_page_index = current_page_index.saturating_add(1);

        if self
            .app_state
            .scan_results_pane_state
            .set_current_page_index(target_page_index)
        {
            self.query_scan_results_current_page(squalr_engine);
        }
    }

    pub(super) fn query_previous_scan_results_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let current_page_index = self.app_state.scan_results_pane_state.current_page_index;
        let target_page_index = current_page_index.saturating_sub(1);

        if self
            .app_state
            .scan_results_pane_state
            .set_current_page_index(target_page_index)
        {
            self.query_scan_results_current_page(squalr_engine);
        }
    }

    pub(super) fn refresh_scan_results_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let _ = self.refresh_scan_results_page_with_feedback(squalr_engine, true);
    }

    pub(super) fn refresh_scan_results_page_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) -> bool {
        if self
            .app_state
            .scan_results_pane_state
            .is_refreshing_scan_results
        {
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "Scan results refresh already in progress.".to_string();
            }
            return false;
        }

        let scan_result_refs_for_current_page = self
            .app_state
            .scan_results_pane_state
            .scan_results
            .iter()
            .map(|scan_result| scan_result.get_base_result().get_scan_result_ref().clone())
            .collect::<Vec<_>>();
        if scan_result_refs_for_current_page.is_empty() {
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "No scan results are available to refresh.".to_string();
            }
            return false;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for scan results refresh.".to_string();
                }
                return false;
            }
        };

        self.app_state
            .scan_results_pane_state
            .is_refreshing_scan_results = true;
        if should_update_status_message {
            self.app_state.scan_results_pane_state.status_message =
                format!("Refreshing {} scan results on the current page.", scan_result_refs_for_current_page.len());
        }

        let scan_results_refresh_request = ScanResultsRefreshRequest {
            scan_result_refs: scan_result_refs_for_current_page,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_refresh_request.send(engine_unprivileged_state, move |scan_results_refresh_response| {
            let _ = response_sender.send(scan_results_refresh_response);
        });

        if !request_dispatched {
            self.app_state
                .scan_results_pane_state
                .is_refreshing_scan_results = false;
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results refresh request.".to_string();
            }
            return false;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_results_refresh_response) => {
                let refreshed_result_count = scan_results_refresh_response.scan_results.len();
                self.app_state
                    .scan_results_pane_state
                    .apply_refreshed_results(scan_results_refresh_response.scan_results);
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = format!("Refreshed {} scan results.", refreshed_result_count);
                }
                self.last_scan_results_periodic_refresh_time = Some(Instant::now());
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results refresh response: {}", receive_error);
                }
            }
        }

        self.app_state
            .scan_results_pane_state
            .is_refreshing_scan_results = false;
        true
    }

    pub(super) fn toggle_selected_scan_results_frozen_state(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.scan_results_pane_state.is_freezing_scan_results {
            self.app_state.scan_results_pane_state.status_message = "Scan results freeze request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to freeze/unfreeze.".to_string();
            return;
        }

        let target_frozen_state = !self
            .app_state
            .scan_results_pane_state
            .any_selected_result_frozen();
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for freeze toggles.".to_string();
                return;
            }
        };

        self.app_state.scan_results_pane_state.is_freezing_scan_results = true;
        self.app_state.scan_results_pane_state.status_message = if target_frozen_state {
            "Freezing selected scan results.".to_string()
        } else {
            "Unfreezing selected scan results.".to_string()
        };

        let scan_results_freeze_request = ScanResultsFreezeRequest {
            scan_result_refs: selected_scan_result_refs,
            is_frozen: target_frozen_state,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_freeze_request.send(engine_unprivileged_state, move |scan_results_freeze_response| {
            let _ = response_sender.send(scan_results_freeze_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_freezing_scan_results = false;
            self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results freeze request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_results_freeze_response) => {
                let failed_toggle_count = scan_results_freeze_response
                    .failed_freeze_toggle_scan_result_refs
                    .len();
                self.app_state.scan_results_pane_state.status_message = if failed_toggle_count == 0 {
                    if target_frozen_state {
                        "Selected scan results frozen.".to_string()
                    } else {
                        "Selected scan results unfrozen.".to_string()
                    }
                } else {
                    format!("Freeze toggle partially failed for {} entries.", failed_toggle_count)
                };
                self.refresh_scan_results_page(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results freeze response: {}", receive_error);
            }
        }

        self.app_state.scan_results_pane_state.is_freezing_scan_results = false;
    }

    pub(super) fn add_selected_scan_results_to_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .scan_results_pane_state
            .is_adding_scan_results_to_project
        {
            self.app_state.scan_results_pane_state.status_message = "Add to project request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to add to project.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for project item creation.".to_string();
                return;
            }
        };

        self.app_state
            .scan_results_pane_state
            .is_adding_scan_results_to_project = true;
        self.app_state.scan_results_pane_state.status_message = format!("Adding {} scan results to project.", selected_scan_result_refs.len());

        let project_items_add_request = ProjectItemsAddRequest {
            scan_result_refs: selected_scan_result_refs,
            target_directory_path: None,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_add_request.send(engine_unprivileged_state, move |project_items_add_response| {
            let _ = response_sender.send(project_items_add_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_add_response) => {
                self.app_state.scan_results_pane_state.status_message = if project_items_add_response.success {
                    format!(
                        "Added {} project items from selected scan results.",
                        project_items_add_response.added_project_item_count
                    )
                } else {
                    "Add-to-project request failed.".to_string()
                };
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for add-to-project response: {}", receive_error);
            }
        }

        self.app_state
            .scan_results_pane_state
            .is_adding_scan_results_to_project = false;
    }

    pub(super) fn delete_selected_scan_results(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.scan_results_pane_state.is_deleting_scan_results {
            self.app_state.scan_results_pane_state.status_message = "Delete request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to delete.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for deletion.".to_string();
                return;
            }
        };

        self.app_state.scan_results_pane_state.is_deleting_scan_results = true;
        self.app_state.scan_results_pane_state.status_message = format!("Deleting {} selected scan results.", selected_scan_result_refs.len());

        let scan_results_delete_request = ScanResultsDeleteRequest {
            scan_result_refs: selected_scan_result_refs,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_delete_request.send(engine_unprivileged_state, move |scan_results_delete_response| {
            let _ = response_sender.send(scan_results_delete_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_deleting_scan_results = false;
            self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results delete request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_scan_results_delete_response) => {
                self.app_state.scan_results_pane_state.status_message = "Deleted selected scan results.".to_string();
                self.query_scan_results_current_page(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results delete response: {}", receive_error);
            }
        }

        self.app_state.scan_results_pane_state.is_deleting_scan_results = false;
    }

    pub(super) fn commit_selected_scan_results_value_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.scan_results_pane_state.is_committing_value_edit {
            self.app_state.scan_results_pane_state.status_message = "Value commit request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to commit value edits.".to_string();
            return;
        }

        let pending_value_edit_text = self
            .app_state
            .scan_results_pane_state
            .pending_value_edit_text
            .trim()
            .to_string();
        if pending_value_edit_text.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "Edit value is empty.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for value commits.".to_string();
                return;
            }
        };

        self.app_state.scan_results_pane_state.is_committing_value_edit = true;
        self.app_state.scan_results_pane_state.status_message = format!(
            "Committing value edit '{}' for {} selected results.",
            pending_value_edit_text,
            selected_scan_result_refs.len()
        );

        let scan_results_set_property_request = ScanResultsSetPropertyRequest {
            scan_result_refs: selected_scan_result_refs,
            anonymous_value_string: AnonymousValueString::new(pending_value_edit_text, AnonymousValueStringFormat::Decimal, ContainerType::None),
            field_namespace: ScanResult::PROPERTY_NAME_VALUE.to_string(),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_set_property_request.send(engine_unprivileged_state, move |scan_results_set_property_response| {
            let _ = response_sender.send(scan_results_set_property_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_committing_value_edit = false;
            self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results set property request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_scan_results_set_property_response) => {
                self.app_state.scan_results_pane_state.status_message = "Committed selected scan result values.".to_string();
                self.refresh_scan_results_page(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results set property response: {}", receive_error);
            }
        }

        self.app_state.scan_results_pane_state.is_committing_value_edit = false;
    }

    pub(super) fn apply_scan_results_query_response(
        &mut self,
        scan_results_query_response: squalr_engine_api::commands::scan_results::query::scan_results_query_response::ScanResultsQueryResponse,
        should_update_status_message: bool,
    ) {
        let result_count = scan_results_query_response.result_count;
        let page_index = scan_results_query_response.page_index;
        self.app_state
            .scan_results_pane_state
            .apply_query_response(scan_results_query_response);
        if should_update_status_message {
            self.app_state.scan_results_pane_state.status_message = format!("Loaded page {} ({} total results).", page_index, result_count);
        }
        self.sync_struct_viewer_focus_from_scan_results();
    }

    pub(super) fn refresh_process_list(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        self.refresh_process_list_with_feedback(squalr_engine, true);
    }

    pub(super) fn refresh_process_list_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        if self
            .app_state
            .process_selector_pane_state
            .is_awaiting_process_list_response
        {
            if should_update_status_message {
                self.app_state.process_selector_pane_state.status_message = "Process list request already in progress.".to_string();
            }
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.process_selector_pane_state.status_message = "No unprivileged engine state is available for process queries.".to_string();
                }
                return;
            }
        };

        self.app_state
            .process_selector_pane_state
            .is_awaiting_process_list_response = true;
        if should_update_status_message {
            self.app_state.process_selector_pane_state.status_message = "Refreshing process list.".to_string();
        }

        let process_list_request = ProcessListRequest {
            require_windowed: self
                .app_state
                .process_selector_pane_state
                .show_windowed_processes_only,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: false,
        };

        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = process_list_request.send(engine_unprivileged_state, move |process_list_response| {
            let _ = response_sender.send(process_list_response);
        });

        if !request_dispatched {
            self.app_state
                .process_selector_pane_state
                .is_awaiting_process_list_response = false;
            if should_update_status_message {
                self.app_state.process_selector_pane_state.status_message = "Failed to dispatch process list request.".to_string();
            }
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(process_list_response) => {
                let process_count = process_list_response.processes.len();
                self.app_state
                    .process_selector_pane_state
                    .apply_process_list(process_list_response.processes);
                self.app_state
                    .process_selector_pane_state
                    .has_loaded_process_list_once = true;
                if should_update_status_message {
                    self.app_state.process_selector_pane_state.status_message = format!("Loaded {} processes.", process_count);
                }
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.process_selector_pane_state.status_message = format!("Timed out waiting for process list response: {}", receive_error);
                }
            }
        }

        self.app_state
            .process_selector_pane_state
            .is_awaiting_process_list_response = false;
    }

    pub(super) fn open_selected_process(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.process_selector_pane_state.is_opening_process {
            self.app_state.process_selector_pane_state.status_message = "Process open request already in progress.".to_string();
            return;
        }

        let selected_process_identifier = match self.app_state.process_selector_pane_state.selected_process_id() {
            Some(selected_process_identifier) => selected_process_identifier,
            None => {
                self.app_state.process_selector_pane_state.status_message = "No process is selected.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.process_selector_pane_state.status_message = "No unprivileged engine state is available for process opening.".to_string();
                return;
            }
        };

        self.app_state.process_selector_pane_state.is_opening_process = true;
        self.app_state.process_selector_pane_state.status_message = format!("Opening process {}.", selected_process_identifier);

        let process_open_request = ProcessOpenRequest {
            process_id: Some(selected_process_identifier),
            search_name: None,
            match_case: false,
        };

        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = process_open_request.send(engine_unprivileged_state, move |process_open_response| {
            let _ = response_sender.send(process_open_response);
        });

        if !request_dispatched {
            self.app_state.process_selector_pane_state.is_opening_process = false;
            self.app_state.process_selector_pane_state.status_message = "Failed to dispatch process open request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(process_open_response) => {
                let opened_process = process_open_response.opened_process_info;
                self.app_state
                    .process_selector_pane_state
                    .set_opened_process(opened_process.clone());
                self.app_state.process_selector_pane_state.status_message = if let Some(opened_process_info) = opened_process {
                    format!(
                        "Opened process {} ({}).",
                        opened_process_info.get_name(),
                        opened_process_info.get_process_id_raw()
                    )
                } else {
                    "Open process request completed with no process.".to_string()
                };
            }
            Err(receive_error) => {
                self.app_state.process_selector_pane_state.status_message = format!("Timed out waiting for process open response: {}", receive_error);
            }
        }

        self.app_state.process_selector_pane_state.is_opening_process = false;
    }

    pub(super) fn refresh_project_list(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        self.refresh_project_list_with_feedback(squalr_engine, true);
    }

    pub(super) fn refresh_project_list_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_awaiting_project_list_response
        {
            if should_update_status_message {
                self.app_state.project_explorer_pane_state.status_message = "Project list request already in progress.".to_string();
            }
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project queries.".to_string();
                }
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_list_response = true;
        if should_update_status_message {
            self.app_state.project_explorer_pane_state.status_message = "Refreshing project list.".to_string();
        }

        let project_list_request = ProjectListRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_list_request.send(engine_unprivileged_state, move |project_list_response| {
            let _ = response_sender.send(project_list_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_list_response) => {
                let project_count = project_list_response.projects_info.len();
                self.app_state
                    .project_explorer_pane_state
                    .apply_project_list(project_list_response.projects_info);
                self.app_state
                    .project_explorer_pane_state
                    .has_loaded_project_list_once = true;
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message = format!("Loaded {} projects.", project_count);
                }
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project list response: {}", receive_error);
                }
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_list_response = false;
    }

    pub(super) fn refresh_project_items_list(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        self.refresh_project_items_list_with_feedback(squalr_engine, true);
    }

    pub(super) fn refresh_project_items_list_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_awaiting_project_item_list_response
        {
            if should_update_status_message {
                self.app_state.project_explorer_pane_state.status_message = "Project item list request already in progress.".to_string();
            }
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message =
                        "No unprivileged engine state is available for project item listing.".to_string();
                }
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_item_list_response = true;
        if should_update_status_message {
            self.app_state.project_explorer_pane_state.status_message = "Refreshing project item hierarchy.".to_string();
        }

        let project_items_list_request = ProjectItemsListRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_list_request.send(engine_unprivileged_state, move |project_items_list_response| {
            let _ = response_sender.send(project_items_list_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_list_response) => {
                let project_item_count = project_items_list_response.opened_project_items.len();
                self.app_state
                    .project_explorer_pane_state
                    .apply_project_items_list(project_items_list_response.opened_project_items);
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message = format!("Loaded {} project items.", project_item_count);
                }
                self.sync_struct_viewer_focus_from_project_items();
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item list response: {}", receive_error);
                }
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_item_list_response = false;
    }

    pub(super) fn create_project_from_pending_name(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_creating_project {
            self.app_state.project_explorer_pane_state.status_message = "Project create request already in progress.".to_string();
            return;
        }

        let new_project_name = match self
            .app_state
            .project_explorer_pane_state
            .pending_project_name_trimmed()
        {
            Some(new_project_name) => new_project_name,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Project name is empty.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project creation.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_creating_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Creating project '{}'.", new_project_name);

        let project_create_request = ProjectCreateRequest {
            project_directory_path: None,
            project_name: Some(new_project_name.clone()),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_create_request.send(engine_unprivileged_state, move |project_create_response| {
            let _ = response_sender.send(project_create_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_create_response) => {
                if project_create_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .cancel_project_name_input();
                    self.app_state.project_explorer_pane_state.status_message = format!(
                        "Created project '{}' at {}.",
                        new_project_name,
                        project_create_response.new_project_path.display()
                    );
                    self.refresh_project_list(squalr_engine);
                    let _ = self
                        .app_state
                        .project_explorer_pane_state
                        .select_project_by_directory_path(&project_create_response.new_project_path);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project create request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project create response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_creating_project = false;
    }

    pub(super) fn create_project_directory_from_pending_name(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_creating_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item create request already in progress.".to_string();
            return;
        }

        let parent_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_directory_target_path()
        {
            Some(parent_directory_path) => parent_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No directory target is selected for project item create.".to_string();
                return;
            }
        };

        let project_item_name = match self
            .app_state
            .project_explorer_pane_state
            .pending_project_name_trimmed()
        {
            Some(project_item_name) => project_item_name,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Project item name is empty.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project item create.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_creating_project_item = true;
        self.app_state.project_explorer_pane_state.status_message =
            format!("Creating directory '{}' under {}.", project_item_name, parent_directory_path.display());

        let project_items_create_request = ProjectItemsCreateRequest {
            parent_directory_path,
            project_item_name: project_item_name.clone(),
            project_item_type: "directory".to_string(),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_create_request.send(engine_unprivileged_state, move |project_items_create_response| {
            let _ = response_sender.send(project_items_create_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_create_response) => {
                if project_items_create_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .cancel_project_name_input();
                    self.app_state.project_explorer_pane_state.status_message = format!("Created project directory '{}'.", project_item_name);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item create request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item create response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_creating_project_item = false;
    }

    pub(super) fn toggle_selected_project_item_activation(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_toggling_project_item_activation
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item activation request already in progress.".to_string();
            return;
        }

        let selected_project_item_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_path()
        {
            Some(selected_project_item_path) => selected_project_item_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project item is selected for activation.".to_string();
                return;
            }
        };
        let is_target_activated = !self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_is_activated();

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message =
                    "No unprivileged engine state is available for project item activation.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_toggling_project_item_activation = true;
        self.app_state.project_explorer_pane_state.status_message =
            format!("Setting activation={} for {}.", is_target_activated, selected_project_item_path.display());

        let project_items_activate_request = ProjectItemsActivateRequest {
            project_item_paths: vec![selected_project_item_path.display().to_string()],
            is_activated: is_target_activated,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_activate_request.send(engine_unprivileged_state, move |project_items_activate_response| {
            let _ = response_sender.send(project_items_activate_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_) => {
                self.app_state.project_explorer_pane_state.status_message = "Updated selected project item activation.".to_string();
                self.refresh_project_items_list(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message =
                    format!("Timed out waiting for project item activation response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_toggling_project_item_activation = false;
    }

    pub(super) fn move_staged_project_items_to_selected_directory(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_moving_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item move request already in progress.".to_string();
            return;
        }

        if !self
            .app_state
            .project_explorer_pane_state
            .has_pending_move_source_paths()
        {
            self.app_state.project_explorer_pane_state.status_message = "No staged project items to move.".to_string();
            return;
        }

        let target_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_directory_target_path()
        {
            Some(target_directory_path) => target_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No target directory is selected for move.".to_string();
                return;
            }
        };

        let project_item_paths = self
            .app_state
            .project_explorer_pane_state
            .pending_move_source_paths();
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for move.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_moving_project_item = true;
        self.app_state.project_explorer_pane_state.status_message =
            format!("Moving {} project items to {}.", project_item_paths.len(), target_directory_path.display());

        let project_items_move_request = ProjectItemsMoveRequest {
            project_item_paths,
            target_directory_path,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_move_request.send(engine_unprivileged_state, move |project_items_move_response| {
            let _ = response_sender.send(project_items_move_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_move_response) => {
                if project_items_move_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .clear_pending_move_source_paths();
                    self.app_state.project_explorer_pane_state.status_message =
                        format!("Moved {} project items.", project_items_move_response.moved_project_item_count);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item move request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item move response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_moving_project_item = false;
    }

    pub(super) fn reorder_selected_project_item(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        move_toward_previous_position: bool,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_reordering_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item reorder request already in progress.".to_string();
            return;
        }

        let project_item_paths = match self
            .app_state
            .project_explorer_pane_state
            .build_reorder_request_paths_for_selected_project_item(move_toward_previous_position)
        {
            Some(project_item_paths) => project_item_paths,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Selected project item cannot be reordered in that direction.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for reorder.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_reordering_project_item = true;
        self.app_state.project_explorer_pane_state.status_message = "Reordering project items.".to_string();

        let project_items_reorder_request = ProjectItemsReorderRequest { project_item_paths };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_reorder_request.send(engine_unprivileged_state, move |project_items_reorder_response| {
            let _ = response_sender.send(project_items_reorder_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_reorder_response) => {
                if project_items_reorder_response.success {
                    self.app_state.project_explorer_pane_state.status_message =
                        format!("Reordered {} project items.", project_items_reorder_response.reordered_project_item_count);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item reorder request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item reorder response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_reordering_project_item = false;
    }

    pub(super) fn delete_selected_project_item_with_confirmation(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_deleting_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item delete request already in progress.".to_string();
            return;
        }

        if !self
            .app_state
            .project_explorer_pane_state
            .has_pending_delete_confirmation_for_selected_project_item()
        {
            if self
                .app_state
                .project_explorer_pane_state
                .arm_delete_confirmation_for_selected_project_item()
            {
                self.app_state.project_explorer_pane_state.status_message = "Press x again to confirm deleting selected project item.".to_string();
            } else {
                self.app_state.project_explorer_pane_state.status_message = "No project item is selected for delete.".to_string();
            }
            return;
        }

        let project_item_paths = self
            .app_state
            .project_explorer_pane_state
            .take_pending_delete_confirmation_paths();
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for delete.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_deleting_project_item = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Deleting {} project items.", project_item_paths.len());

        let project_items_delete_request = ProjectItemsDeleteRequest { project_item_paths };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_delete_request.send(engine_unprivileged_state, move |project_items_delete_response| {
            let _ = response_sender.send(project_items_delete_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_delete_response) => {
                if project_items_delete_response.success {
                    self.app_state.project_explorer_pane_state.status_message =
                        format!("Deleted {} project items.", project_items_delete_response.deleted_project_item_count);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item delete request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item delete response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_deleting_project_item = false;
    }

    pub(super) fn open_selected_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_opening_project {
            self.app_state.project_explorer_pane_state.status_message = "Project open request already in progress.".to_string();
            return;
        }

        let selected_project_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_directory_path()
        {
            Some(selected_project_directory_path) => selected_project_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project is selected.".to_string();
                return;
            }
        };

        let selected_project_name = self
            .app_state
            .project_explorer_pane_state
            .selected_project_name()
            .unwrap_or_else(|| "<unknown>".to_string());

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project opening.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_opening_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Opening project '{}'.", selected_project_name);

        let project_open_request = ProjectOpenRequest {
            open_file_browser: false,
            project_directory_path: Some(selected_project_directory_path.clone()),
            project_name: None,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_open_request.send(engine_unprivileged_state, move |project_open_response| {
            let _ = response_sender.send(project_open_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_open_response) => {
                if project_open_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .set_active_project(Some(selected_project_name.clone()), Some(selected_project_directory_path.clone()));
                    self.app_state.project_explorer_pane_state.clear_project_items();
                    self.app_state
                        .struct_viewer_pane_state
                        .clear_focus("Cleared struct viewer after project open.");
                    self.app_state.project_explorer_pane_state.status_message = format!("Opened project '{}'.", selected_project_name);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project open request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project open response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_opening_project = false;
    }

    pub(super) fn rename_selected_project_from_pending_name(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_renaming_project {
            self.app_state.project_explorer_pane_state.status_message = "Project rename request already in progress.".to_string();
            return;
        }

        let selected_project_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_directory_path()
        {
            Some(selected_project_directory_path) => selected_project_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project is selected for rename.".to_string();
                return;
            }
        };
        let selected_project_directory_path_for_active_check = selected_project_directory_path.clone();

        let new_project_name = match self
            .app_state
            .project_explorer_pane_state
            .pending_project_name_trimmed()
        {
            Some(new_project_name) => new_project_name,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Project name is empty.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project renaming.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_renaming_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Renaming project to '{}'.", new_project_name);

        let project_rename_request = ProjectRenameRequest {
            project_directory_path: selected_project_directory_path,
            new_project_name: new_project_name.clone(),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_rename_request.send(engine_unprivileged_state, move |project_rename_response| {
            let _ = response_sender.send(project_rename_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_rename_response) => {
                if project_rename_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .cancel_project_name_input();
                    self.app_state.project_explorer_pane_state.status_message = format!("Renamed project to '{}'.", new_project_name);
                    self.refresh_project_list(squalr_engine);
                    let _ = self
                        .app_state
                        .project_explorer_pane_state
                        .select_project_by_directory_path(&project_rename_response.new_project_path);
                    if self
                        .app_state
                        .project_explorer_pane_state
                        .active_project_directory_path
                        .as_ref()
                        .is_some_and(|active_project_directory_path| *active_project_directory_path == selected_project_directory_path_for_active_check)
                    {
                        self.app_state
                            .project_explorer_pane_state
                            .set_active_project(Some(new_project_name), Some(project_rename_response.new_project_path));
                        self.sync_struct_viewer_focus_from_project_items();
                    }
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project rename request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project rename response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_renaming_project = false;
    }

    pub(super) fn delete_selected_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_deleting_project {
            self.app_state.project_explorer_pane_state.status_message = "Project delete request already in progress.".to_string();
            return;
        }

        let selected_project_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_directory_path()
        {
            Some(selected_project_directory_path) => selected_project_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project is selected for delete.".to_string();
                return;
            }
        };

        let selected_project_name = self
            .app_state
            .project_explorer_pane_state
            .selected_project_name()
            .unwrap_or_else(|| "<unknown>".to_string());

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project deletion.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_deleting_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Deleting project '{}'.", selected_project_name);

        let project_delete_request = ProjectDeleteRequest {
            project_directory_path: Some(selected_project_directory_path.clone()),
            project_name: None,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_delete_request.send(engine_unprivileged_state, move |project_delete_response| {
            let _ = response_sender.send(project_delete_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_delete_response) => {
                if project_delete_response.success {
                    if self
                        .app_state
                        .project_explorer_pane_state
                        .active_project_directory_path
                        .as_ref()
                        .is_some_and(|active_project_directory_path| *active_project_directory_path == selected_project_directory_path)
                    {
                        self.app_state
                            .project_explorer_pane_state
                            .set_active_project(None, None);
                        self.app_state.project_explorer_pane_state.clear_project_items();
                        self.app_state
                            .struct_viewer_pane_state
                            .clear_focus("Cleared struct viewer after project delete.");
                    }
                    self.app_state.project_explorer_pane_state.status_message = format!("Deleted project '{}'.", selected_project_name);
                    self.refresh_project_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project delete request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project delete response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_deleting_project = false;
    }

    pub(super) fn close_active_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_closing_project {
            self.app_state.project_explorer_pane_state.status_message = "Project close request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project close.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_closing_project = true;
        self.app_state.project_explorer_pane_state.status_message = "Closing active project.".to_string();

        let project_close_request = ProjectCloseRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_close_request.send(engine_unprivileged_state, move |project_close_response| {
            let _ = response_sender.send(project_close_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_close_response) => {
                if project_close_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .set_active_project(None, None);
                    self.app_state.project_explorer_pane_state.clear_project_items();
                    self.app_state
                        .struct_viewer_pane_state
                        .clear_focus("Cleared struct viewer after project close.");
                    self.app_state.project_explorer_pane_state.status_message = "Closed active project.".to_string();
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project close request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project close response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_closing_project = false;
    }

    pub(super) fn extract_string_value_from_edited_field(edited_field: &ValuedStructField) -> Option<String> {
        let edited_data_value = edited_field.get_data_value()?;
        let edited_name = String::from_utf8(edited_data_value.get_value_bytes().clone()).ok()?;
        let edited_name = edited_name.trim();

        if edited_name.is_empty() { None } else { Some(edited_name.to_string()) }
    }

    pub(super) fn build_project_item_rename_request(
        project_item_path: &Path,
        project_item_type_id: &str,
        edited_name: &str,
    ) -> Option<ProjectItemsRenameRequest> {
        let sanitized_file_name = Path::new(edited_name)
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .map(str::trim)
            .filter(|file_name| !file_name.is_empty())?
            .to_string();
        let is_directory_project_item = project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID;
        let renamed_project_item_name = if is_directory_project_item {
            sanitized_file_name
        } else {
            let mut file_name_with_extension = sanitized_file_name.clone();
            let expected_extension = Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.');
            let has_expected_extension = Path::new(&sanitized_file_name)
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension.eq_ignore_ascii_case(expected_extension))
                .unwrap_or(false);
            if !has_expected_extension {
                file_name_with_extension.push('.');
                file_name_with_extension.push_str(expected_extension);
            }

            file_name_with_extension
        };
        let current_file_name = project_item_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or_default();
        if current_file_name == renamed_project_item_name {
            return None;
        }

        Some(ProjectItemsRenameRequest {
            project_item_path: project_item_path.to_path_buf(),
            project_item_name: renamed_project_item_name,
        })
    }

    pub(super) fn build_memory_write_request_for_project_item_edit(
        project_item: &mut ProjectItem,
        edited_field: &ValuedStructField,
    ) -> Option<MemoryWriteRequest> {
        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            return None;
        }
        if edited_field.get_name() != ProjectItemTypeAddress::PROPERTY_ADDRESS {
            return None;
        }

        let edited_data_value = edited_field.get_data_value()?;
        let address = ProjectItemTypeAddress::get_field_address(project_item);
        let module_name = ProjectItemTypeAddress::get_field_module(project_item);

        Some(MemoryWriteRequest {
            address,
            module_name,
            value: edited_data_value.get_value_bytes().clone(),
        })
    }

    pub(super) fn build_scan_results_set_property_request_for_struct_edit(
        scan_result_refs: Vec<ScanResultRef>,
        edited_field: &ValuedStructField,
    ) -> Result<ScanResultsSetPropertyRequest, String> {
        let edited_data_value = edited_field
            .get_data_value()
            .ok_or_else(|| "Nested struct scan result edits are not supported in the TUI yet.".to_string())?;
        let symbol_registry = SymbolRegistry::get_instance();
        let default_edit_format = symbol_registry.get_default_anonymous_value_string_format(edited_data_value.get_data_type_ref());
        let edited_anonymous_value = symbol_registry
            .anonymize_value(edited_data_value, default_edit_format)
            .map_err(|error| format!("Failed to format edited scan result value: {}", error))?;

        Ok(ScanResultsSetPropertyRequest {
            scan_result_refs,
            field_namespace: edited_field.get_name().to_string(),
            anonymous_value_string: edited_anonymous_value,
        })
    }

    pub(super) fn should_apply_struct_field_edit_to_project_item(
        project_item_type_id: &str,
        edited_field_name: &str,
    ) -> bool {
        !(edited_field_name == ProjectItem::PROPERTY_NAME && project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID)
    }
}
