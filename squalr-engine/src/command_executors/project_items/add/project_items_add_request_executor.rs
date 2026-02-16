use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project_items::add::project_items_add_request::ProjectItemsAddRequest;
use squalr_engine_api::commands::project_items::add::project_items_add_response::ProjectItemsAddResponse;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_response::ScanResultsRefreshResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

impl UnprivilegedCommandRequestExecutor for ProjectItemsAddRequest {
    type ResponseType = ProjectItemsAddResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.scan_result_refs.is_empty() {
            return ProjectItemsAddResponse {
                success: true,
                added_project_item_count: 0,
            };
        }

        let scan_results = match request_scan_results(engine_unprivileged_state, self) {
            Some(scan_results) => scan_results,
            None => {
                return ProjectItemsAddResponse {
                    success: false,
                    added_project_item_count: 0,
                };
            }
        };

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project = match opened_project.write() {
            Ok(opened_project) => opened_project,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for add command: {}", error);

                return ProjectItemsAddResponse {
                    success: false,
                    added_project_item_count: 0,
                };
            }
        };
        let opened_project = match opened_project.as_mut() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Cannot add scan results to project items without an opened project.");

                return ProjectItemsAddResponse {
                    success: false,
                    added_project_item_count: 0,
                };
            }
        };
        let project_directory_path = match opened_project.get_project_info().get_project_directory() {
            Some(project_directory_path) => project_directory_path,
            None => {
                log::error!("Failed to resolve opened project directory for project item add operation.");

                return ProjectItemsAddResponse {
                    success: false,
                    added_project_item_count: 0,
                };
            }
        };

        let added_file_paths = add_scan_results_to_project(opened_project, &project_directory_path, &scan_results);

        if added_file_paths.is_empty() {
            return ProjectItemsAddResponse {
                success: true,
                added_project_item_count: 0,
            };
        }

        if let Err(error) = create_placeholder_files(&added_file_paths) {
            log::error!("Failed creating project item placeholder files before save: {}", error);

            return ProjectItemsAddResponse {
                success: false,
                added_project_item_count: 0,
            };
        }

        if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
            log::error!("Failed to save project after add operation: {}", error);

            return ProjectItemsAddResponse {
                success: false,
                added_project_item_count: 0,
            };
        }

        project_manager.notify_project_items_changed();

        ProjectItemsAddResponse {
            success: true,
            added_project_item_count: added_file_paths.len() as u64,
        }
    }
}

fn request_scan_results(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    project_items_add_request: &ProjectItemsAddRequest,
) -> Option<Vec<ScanResult>> {
    let scan_results_refresh_request = ScanResultsRefreshRequest {
        scan_result_refs: project_items_add_request.scan_result_refs.clone(),
    };
    let scan_results_refresh_command = scan_results_refresh_request.to_engine_command();
    let (scan_results_sender, scan_results_receiver): (
        Sender<Result<ScanResultsRefreshResponse, String>>,
        Receiver<Result<ScanResultsRefreshResponse, String>>,
    ) = channel();

    let dispatch_result = match engine_unprivileged_state.get_bindings().read() {
        Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
            scan_results_refresh_command,
            Box::new(move |engine_response| {
                let conversion_result = match ScanResultsRefreshResponse::from_engine_response(engine_response) {
                    Ok(scan_results_refresh_response) => Ok(scan_results_refresh_response),
                    Err(unexpected_response) => Err(format!("Unexpected response variant for project-items add: {:?}", unexpected_response)),
                };

                if let Err(error) = scan_results_sender.send(conversion_result) {
                    log::error!("Failed to deliver refreshed scan results to project-items add command: {}", error);
                }
            }),
        ),
        Err(error) => {
            log::error!("Failed to acquire engine bindings lock for project-items add command: {}", error);
            return None;
        }
    };

    if let Err(error) = dispatch_result {
        log::error!("Failed to dispatch refresh request for project-items add command: {}", error);
        return None;
    }

    match scan_results_receiver.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(scan_results_refresh_response)) => Some(scan_results_refresh_response.scan_results),
        Ok(Err(error)) => {
            log::error!("Failed to convert refresh response for project-items add command: {}", error);
            None
        }
        Err(error) => {
            log::error!("Timed out waiting for refreshed scan results during project-items add command: {}", error);
            None
        }
    }
}

fn add_scan_results_to_project(
    opened_project: &mut Project,
    project_directory_path: &PathBuf,
    scan_results: &[ScanResult],
) -> Vec<PathBuf> {
    let symbol_registry = SymbolRegistry::get_instance();
    let project_items = opened_project.get_project_items_mut();
    let mut added_file_paths = Vec::new();
    let directory_relative_path = PathBuf::from("Address");
    let directory_absolute_path = project_directory_path.join(&directory_relative_path);
    let directory_project_item_ref = ProjectItemRef::new(directory_absolute_path);

    if !project_items.contains_key(&directory_project_item_ref) {
        let directory_project_item = ProjectItemTypeDirectory::new_project_item(&directory_project_item_ref);
        project_items.insert(directory_project_item_ref, directory_project_item);
    }

    for scan_result in scan_results {
        let data_type_ref = scan_result.get_data_type_ref();
        let default_data_value = match symbol_registry.get_default_value(data_type_ref) {
            Some(default_data_value) => default_data_value,
            None => {
                log::warn!("Skipping scan result add for unsupported data type: {}", data_type_ref.get_data_type_id());
                continue;
            }
        };
        let scan_result_global_index = scan_result
            .get_base_result()
            .get_scan_result_ref()
            .get_scan_result_global_index();
        let project_item_file_name = format!("scan_result_{}.json", scan_result_global_index);
        let project_item_relative_path = directory_relative_path.join(project_item_file_name);
        let project_item_absolute_path = project_directory_path.join(&project_item_relative_path);
        let project_item_ref = ProjectItemRef::new(project_item_absolute_path.clone());

        if project_items.contains_key(&project_item_ref) {
            continue;
        }

        let project_item_name = build_project_item_name(scan_result);
        let project_item = ProjectItemTypeAddress::new_project_item(
            &project_item_name,
            scan_result.get_module_offset(),
            scan_result.get_module(),
            "",
            default_data_value,
        );

        project_items.insert(project_item_ref, project_item);
        added_file_paths.push(project_item_absolute_path);
    }

    added_file_paths
}

fn create_placeholder_files(file_paths: &[PathBuf]) -> Result<(), String> {
    for file_path in file_paths {
        if let Some(parent_path) = file_path.parent() {
            if let Err(error) = fs::create_dir_all(parent_path) {
                return Err(format!("Failed creating project item parent directory {:?}: {}", parent_path, error));
            }
        }

        if !file_path.exists() {
            if let Err(error) = File::create(file_path) {
                return Err(format!("Failed creating project item file {:?}: {}", file_path, error));
            }
        }
    }

    Ok(())
}

fn build_project_item_name(scan_result: &ScanResult) -> String {
    if scan_result.is_module() {
        format!("{}+0x{:X}", scan_result.get_module(), scan_result.get_module_offset())
    } else {
        format!("0x{:X}", scan_result.get_address())
    }
}
