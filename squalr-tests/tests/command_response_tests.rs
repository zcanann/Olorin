use crossbeam_channel::{Receiver, unbounded};
use squalr_engine_api::commands::memory::memory_command::MemoryCommand;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::commands::process::process_command::ProcessCommand;
use squalr_engine_api::commands::project::create::project_create_request::ProjectCreateRequest;
use squalr_engine_api::commands::project::create::project_create_response::ProjectCreateResponse;
use squalr_engine_api::commands::project::delete::project_delete_request::ProjectDeleteRequest;
use squalr_engine_api::commands::project::delete::project_delete_response::ProjectDeleteResponse;
use squalr_engine_api::commands::project::export::project_export_response::ProjectExportResponse;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest as UnprivilegedProjectOpenRequest;
use squalr_engine_api::commands::project::open::project_open_response::ProjectOpenResponse;
use squalr_engine_api::commands::project::project_command::ProjectCommand;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project::rename::project_rename_response::ProjectRenameResponse;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse;
use squalr_engine_api::commands::project_items::project_items_command::ProjectItemsCommand;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::new::scan_new_response::ScanNewResponse;
use squalr_engine_api::commands::scan::scan_command::ScanCommand;
use squalr_engine_api::commands::scan_results::scan_results_command::ScanResultsCommand;
use squalr_engine_api::commands::settings::general::general_settings_command::GeneralSettingsCommand;
use squalr_engine_api::commands::settings::memory::memory_settings_command::MemorySettingsCommand;
use squalr_engine_api::commands::settings::scan::scan_settings_command::ScanSettingsCommand;
use squalr_engine_api::commands::settings::settings_command::SettingsCommand;
use squalr_engine_api::commands::trackable_tasks::trackable_tasks_command::TrackableTasksCommand;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use squalr_engine_api::events::engine_event::EngineEvent;
use squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use structopt::StructOpt;

struct MockEngineBindings {
    dispatched_commands: Arc<Mutex<Vec<PrivilegedCommand>>>,
    dispatched_unprivileged_commands: Arc<Mutex<Vec<UnprivilegedCommand>>>,
    response_to_return: PrivilegedCommandResponse,
    unprivileged_response_to_return: UnprivilegedCommandResponse,
}

impl MockEngineBindings {
    fn new(
        response_to_return: PrivilegedCommandResponse,
        unprivileged_response_to_return: UnprivilegedCommandResponse,
    ) -> Self {
        Self {
            dispatched_commands: Arc::new(Mutex::new(Vec::new())),
            dispatched_unprivileged_commands: Arc::new(Mutex::new(Vec::new())),
            response_to_return,
            unprivileged_response_to_return,
        }
    }

    fn get_dispatched_commands(&self) -> Arc<Mutex<Vec<PrivilegedCommand>>> {
        self.dispatched_commands.clone()
    }

    fn get_dispatched_unprivileged_commands(&self) -> Arc<Mutex<Vec<UnprivilegedCommand>>> {
        self.dispatched_unprivileged_commands.clone()
    }
}

impl EngineApiUnprivilegedBindings for MockEngineBindings {
    fn dispatch_privileged_command(
        &self,
        engine_command: PrivilegedCommand,
        callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), String> {
        match self.dispatched_commands.lock() {
            Ok(mut dispatched_commands) => {
                dispatched_commands.push(engine_command);
            }
            Err(error) => {
                return Err(format!("Failed to capture dispatched command: {}", error));
            }
        }

        callback(self.response_to_return.clone());

        Ok(())
    }

    fn dispatch_unprivileged_command(
        &self,
        engine_command: UnprivilegedCommand,
        _engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), String> {
        match self.dispatched_unprivileged_commands.lock() {
            Ok(mut dispatched_unprivileged_commands) => {
                dispatched_unprivileged_commands.push(engine_command);
            }
            Err(error) => {
                return Err(format!("Failed to capture dispatched unprivileged command: {}", error));
            }
        }

        callback(self.unprivileged_response_to_return.clone());

        Ok(())
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        let (_event_sender, event_receiver) = unbounded();

        Ok(event_receiver)
    }
}

#[test]
fn memory_write_request_dispatches_write_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let memory_write_request = MemoryWriteRequest {
        address: 0x40,
        module_name: String::new(),
        value: vec![1, 2, 3],
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    memory_write_request.send_unprivileged(&bindings, move |memory_write_response| {
        callback_invoked_clone.store(memory_write_response.success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Memory(MemoryCommand::Write {
            memory_write_request: captured_memory_write_request,
        }) => {
            assert_eq!(captured_memory_write_request.address, 0x40);
            assert_eq!(captured_memory_write_request.value, vec![1, 2, 3]);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn process_open_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let process_open_request = ProcessOpenRequest {
        process_id: Some(1234),
        search_name: None,
        match_case: false,
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    process_open_request.send_unprivileged(&bindings, move |_process_open_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Process(ProcessCommand::Open {
            process_open_request: captured_process_open_request,
        }) => {
            assert_eq!(captured_process_open_request.process_id, Some(1234));
            assert_eq!(captured_process_open_request.search_name, None);
            assert!(!captured_process_open_request.match_case);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_list_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = EngineUnprivilegedState::new(Arc::new(RwLock::new(MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    ))));

    let project_list_request = ProjectListRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_list_request.send_unprivileged(&bindings, &execution_context, move |project_list_response| {
        callback_invoked_clone.store(project_list_response.projects_info.is_empty(), Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::List { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_open_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectOpenResponse { success: true }.to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = EngineUnprivilegedState::new(Arc::new(RwLock::new(MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    ))));

    let project_open_request = UnprivilegedProjectOpenRequest {
        open_file_browser: true,
        project_directory_path: Some(PathBuf::from("C:\\Projects\\ContractProject")),
        project_name: Some("ContractProject".to_string()),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_open_request.send_unprivileged(&bindings, &execution_context, move |project_open_response| {
        callback_invoked_clone.store(project_open_response.success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Open {
            project_open_request: captured_project_open_request,
        }) => {
            assert!(captured_project_open_request.open_file_browser);
            assert_eq!(
                captured_project_open_request
                    .project_directory_path
                    .as_ref()
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects\\ContractProject".to_string())
            );
            assert_eq!(captured_project_open_request.project_name, Some("ContractProject".to_string()));
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_create_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectCreateResponse {
            success: true,
            new_project_path: PathBuf::from("C:\\Projects\\ContractCreateProject"),
        }
        .to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = EngineUnprivilegedState::new(Arc::new(RwLock::new(MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    ))));

    let project_create_request = ProjectCreateRequest {
        project_directory_path: Some(PathBuf::from("C:\\Projects")),
        project_name: Some("ContractCreateProject".to_string()),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_create_request.send_unprivileged(&bindings, &execution_context, move |project_create_response| {
        let callback_should_mark_success =
            project_create_response.success && project_create_response.new_project_path == PathBuf::from("C:\\Projects\\ContractCreateProject");
        callback_invoked_clone.store(callback_should_mark_success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Create {
            project_create_request: captured_project_create_request,
        }) => {
            assert_eq!(
                captured_project_create_request
                    .project_directory_path
                    .as_ref()
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects".to_string())
            );
            assert_eq!(captured_project_create_request.project_name, Some("ContractCreateProject".to_string()));
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_delete_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectDeleteResponse { success: true }.to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = EngineUnprivilegedState::new(Arc::new(RwLock::new(MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    ))));

    let project_delete_request = ProjectDeleteRequest {
        project_directory_path: Some(PathBuf::from("C:\\Projects\\ContractDeleteProject")),
        project_name: Some("ContractDeleteProject".to_string()),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_delete_request.send_unprivileged(&bindings, &execution_context, move |project_delete_response| {
        callback_invoked_clone.store(project_delete_response.success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Delete {
            project_delete_request: captured_project_delete_request,
        }) => {
            assert_eq!(
                captured_project_delete_request
                    .project_directory_path
                    .as_ref()
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects\\ContractDeleteProject".to_string())
            );
            assert_eq!(captured_project_delete_request.project_name, Some("ContractDeleteProject".to_string()));
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_rename_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectRenameResponse {
            success: true,
            new_project_path: PathBuf::from("C:\\Projects\\RenamedProject"),
        }
        .to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = EngineUnprivilegedState::new(Arc::new(RwLock::new(MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    ))));

    let project_rename_request = ProjectRenameRequest {
        project_directory_path: PathBuf::from("C:\\Projects\\OriginalProject"),
        new_project_name: "RenamedProject".to_string(),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_rename_request.send_unprivileged(&bindings, &execution_context, move |project_rename_response| {
        let callback_should_mark_success =
            project_rename_response.success && project_rename_response.new_project_path == PathBuf::from("C:\\Projects\\RenamedProject");
        callback_invoked_clone.store(callback_should_mark_success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Rename {
            project_rename_request: captured_project_rename_request,
        }) => {
            assert_eq!(
                captured_project_rename_request
                    .project_directory_path
                    .display()
                    .to_string(),
                "C:\\Projects\\OriginalProject".to_string()
            );
            assert_eq!(captured_project_rename_request.new_project_name, "RenamedProject".to_string());
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_items_activate_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectItemsActivateResponse {}.to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = EngineUnprivilegedState::new(Arc::new(RwLock::new(MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectExportResponse::default().to_engine_response(),
    ))));

    let project_items_activate_request = ProjectItemsActivateRequest {
        project_item_paths: vec![
            "Addresses.Player.Health".to_string(),
            "Addresses.Player.Ammo".to_string(),
        ],
        is_activated: true,
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_items_activate_request.send_unprivileged(&bindings, &execution_context, move |_project_items_activate_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Activate {
            project_items_activate_request: captured_project_items_activate_request,
        }) => {
            assert_eq!(
                captured_project_items_activate_request.project_item_paths,
                vec![
                    "Addresses.Player.Health".to_string(),
                    "Addresses.Player.Ammo".to_string(),
                ]
            );
            assert!(captured_project_items_activate_request.is_activated);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_new_request_maps_to_scan_new_privileged_command() {
    match (ScanNewRequest {}).to_engine_command() {
        PrivilegedCommand::Scan(ScanCommand::New {
            scan_new_request: ScanNewRequest {},
        }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn typed_response_round_trip_for_scan_new_response() {
    let scan_new_response = ScanNewResponse {};

    let engine_response = scan_new_response.to_engine_response();
    let typed_response_result = ScanNewResponse::from_engine_response(engine_response);

    assert!(typed_response_result.is_ok());
}

#[test]
fn privileged_command_parser_accepts_trackable_tasks_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "tasks", "list"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::TrackableTasks(TrackableTasksCommand::List { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_memory_settings_set_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "settings",
            "memory",
            "set",
            "--start-address",
            "4096",
            "--end-address",
            "8192",
            "--only-query-usermode",
            "true",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::Memory {
            memory_settings_command: MemorySettingsCommand::Set { memory_settings_set_request },
        }) => {
            assert_eq!(memory_settings_set_request.start_address, Some(4096));
            assert_eq!(memory_settings_set_request.end_address, Some(8192));
            assert_eq!(memory_settings_set_request.only_query_usermode, Some(true));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_pointer_scan_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "scan",
            "pointer-scan",
            "--target-address",
            "4096;address;",
            "--pointer-data-type-ref",
            "u64",
            "--max-depth",
            "5",
            "--offset-size",
            "8",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Scan(ScanCommand::PointerScan { pointer_scan_request }) => {
            assert_eq!(pointer_scan_request.target_address.get_anonymous_value_string(), "4096");
            assert_eq!(pointer_scan_request.pointer_data_type_ref.get_data_type_id(), "u64");
            assert_eq!(pointer_scan_request.max_depth, 5);
            assert_eq!(pointer_scan_request.offset_size, 8);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_process_list_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "process",
            "list",
            "--require-windowed",
            "--search-name",
            "calc",
            "--match-case",
            "--limit",
            "10",
            "--fetch-icons",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Process(ProcessCommand::List { process_list_request }) => {
            assert!(process_list_request.require_windowed);
            assert_eq!(process_list_request.search_name, Some("calc".to_string()));
            assert!(process_list_request.match_case);
            assert_eq!(process_list_request.limit, Some(10));
            assert!(process_list_request.fetch_icons);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_process_open_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "process",
            "open",
            "--process-id",
            "1337",
            "--search-name",
            "calc",
            "--match-case",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Process(ProcessCommand::Open { process_open_request }) => {
            assert_eq!(process_open_request.process_id, Some(1337));
            assert_eq!(process_open_request.search_name, Some("calc".to_string()));
            assert!(process_open_request.match_case);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_settings_set_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "settings",
            "scan",
            "set",
            "--results-page-size",
            "512",
            "--memory-alignment",
            "8",
            "--memory-read-mode",
            "i",
            "--floating-point-tolerance",
            "epsilon",
            "--is-single-threaded-scan",
            "true",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::Scan {
            scan_settings_command: ScanSettingsCommand::Set { scan_settings_set_request },
        }) => {
            assert_eq!(scan_settings_set_request.results_page_size, Some(512));
            assert_eq!(scan_settings_set_request.memory_alignment, Some(MemoryAlignment::Alignment8));
            assert_eq!(scan_settings_set_request.memory_read_mode, Some(MemoryReadMode::ReadInterleavedWithScan));
            assert_eq!(
                scan_settings_set_request.floating_point_tolerance,
                Some(FloatingPointTolerance::ToleranceEpsilon)
            );
            assert_eq!(scan_settings_set_request.is_single_threaded_scan, Some(true));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_general_settings_set_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "settings",
            "general",
            "set",
            "--engine-request-delay",
            "250",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::General {
            general_settings_command: GeneralSettingsCommand::Set { general_settings_set_request },
        }) => {
            assert_eq!(general_settings_set_request.engine_request_delay, Some(250));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_element_scan_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "scan",
            "element-scan",
            "--scan-constraints",
            ">=5;dec;",
            "--scan-constraints",
            "==",
            "--data-type-refs",
            "i32",
            "--data-type-refs",
            "f32",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Scan(ScanCommand::ElementScan { element_scan_request }) => {
            assert_eq!(element_scan_request.scan_constraints.len(), 2);
            assert_eq!(element_scan_request.data_type_refs.len(), 2);

            let first_constraint = &element_scan_request.scan_constraints[0];
            assert_eq!(
                first_constraint.get_scan_compare_type(),
                ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThanOrEqual)
            );
            assert_eq!(
                first_constraint
                    .get_anonymous_value_string()
                    .as_ref()
                    .map(|anonymous_value_string| anonymous_value_string.get_anonymous_value_string()),
                Some("5")
            );

            let second_constraint = &element_scan_request.scan_constraints[1];
            assert_eq!(
                second_constraint.get_scan_compare_type(),
                ScanCompareType::Relative(ScanCompareTypeRelative::Unchanged)
            );
            assert_eq!(second_constraint.get_anonymous_value_string(), &None);

            assert_eq!(element_scan_request.data_type_refs[0].get_data_type_id(), "i32");
            assert_eq!(element_scan_request.data_type_refs[1].get_data_type_id(), "f32");
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_list_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "results", "list", "--page-index", "2"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::List { results_list_request }) => {
            assert_eq!(results_list_request.page_index, 2);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_query_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "results", "query", "--page-index", "5"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::Query { results_query_request }) => {
            assert_eq!(results_query_request.page_index, 5);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_refresh_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "results",
            "refresh",
            "--scan-result-refs",
            "13",
            "--scan-result-refs",
            "21",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::Refresh { results_refresh_request }) => {
            assert_eq!(results_refresh_request.scan_result_refs.len(), 2);
            assert_eq!(results_refresh_request.scan_result_refs[0].get_scan_result_global_index(), 13);
            assert_eq!(results_refresh_request.scan_result_refs[1].get_scan_result_global_index(), 21);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_add_to_project_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "results",
            "add-to-project",
            "--scan-result-refs",
            "8",
            "--scan-result-refs",
            "34",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::AddToProject {
            results_add_to_project_request,
        }) => {
            assert_eq!(results_add_to_project_request.scan_result_refs.len(), 2);
            assert_eq!(results_add_to_project_request.scan_result_refs[0].get_scan_result_global_index(), 8);
            assert_eq!(results_add_to_project_request.scan_result_refs[1].get_scan_result_global_index(), 34);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_set_property_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "results",
            "set-property",
            "--scan-result-refs",
            "7",
            "--scan-result-refs",
            "11",
            "--anonymous-value-string",
            "255;dec;",
            "--field-namespace",
            "value",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::SetProperty { results_set_property_request }) => {
            assert_eq!(results_set_property_request.scan_result_refs.len(), 2);
            assert_eq!(results_set_property_request.scan_result_refs[0].get_scan_result_global_index(), 7);
            assert_eq!(results_set_property_request.scan_result_refs[1].get_scan_result_global_index(), 11);
            assert_eq!(
                results_set_property_request
                    .anonymous_value_string
                    .get_anonymous_value_string(),
                "255"
            );
            assert_eq!(results_set_property_request.field_namespace, "value".to_string());
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_delete_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "results",
            "delete",
            "--scan-result-refs",
            "17",
            "--scan-result-refs",
            "29",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::Delete { results_delete_request }) => {
            assert_eq!(results_delete_request.scan_result_refs.len(), 2);
            assert_eq!(results_delete_request.scan_result_refs[0].get_scan_result_global_index(), 17);
            assert_eq!(results_delete_request.scan_result_refs[1].get_scan_result_global_index(), 29);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_results_freeze_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "results",
            "freeze",
            "--scan-result-refs",
            "3",
            "--scan-result-refs",
            "9",
            "--is-frozen",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Results(ScanResultsCommand::Freeze { results_freeze_request }) => {
            assert_eq!(results_freeze_request.scan_result_refs.len(), 2);
            assert_eq!(results_freeze_request.scan_result_refs[0].get_scan_result_global_index(), 3);
            assert_eq!(results_freeze_request.scan_result_refs[1].get_scan_result_global_index(), 9);
            assert!(results_freeze_request.is_frozen);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_memory_read_with_short_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "memory",
            "read",
            "--address",
            "4096",
            "-m",
            "kernel32.dll",
            "-v",
            "u32",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Memory(MemoryCommand::Read { memory_read_request }) => {
            assert_eq!(memory_read_request.address, 4096);
            assert_eq!(memory_read_request.module_name, "kernel32.dll".to_string());
            assert_eq!(
                memory_read_request
                    .symbolic_struct_definition
                    .get_symbol_namespace(),
                ""
            );
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_memory_write_with_short_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "memory",
            "write",
            "--address",
            "8192",
            "-m",
            "game.exe",
            "-v",
            "255",
            "17",
            "42",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Memory(MemoryCommand::Write { memory_write_request }) => {
            assert_eq!(memory_write_request.address, 8192);
            assert_eq!(memory_write_request.module_name, "game.exe".to_string());
            assert_eq!(memory_write_request.value, vec![255, 17, 42]);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_reset_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "scan", "reset"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Scan(ScanCommand::Reset { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_collect_values_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "scan", "collect-values"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Scan(ScanCommand::CollectValues { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_struct_scan_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "scan",
            "struct-scan",
            "--scan-value",
            "12;dec;",
            "--data-type-ids",
            "u32",
            "--data-type-ids",
            "f32",
            "--compare-type",
            "==",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Scan(ScanCommand::StructScan { struct_scan_request }) => {
            assert_eq!(
                struct_scan_request
                    .scan_value
                    .as_ref()
                    .map(|scan_value| scan_value.get_anonymous_value_string()),
                Some("12")
            );
            assert_eq!(struct_scan_request.data_type_ids, vec!["u32".to_string(), "f32".to_string()]);
            assert_eq!(struct_scan_request.compare_type, ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_trackable_tasks_cancel_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "tasks", "cancel", "--task-id", "scan-123"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::TrackableTasks(TrackableTasksCommand::Cancel {
            trackable_tasks_cancel_request,
        }) => {
            assert_eq!(trackable_tasks_cancel_request.task_id, "scan-123".to_string());
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_process_close_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "process", "close"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Process(ProcessCommand::Close { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_create_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project",
            "create",
            "--project-directory-path",
            "C:\\Projects",
            "--project-name",
            "UnitTestProject",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Create { project_create_request }) => {
            assert_eq!(
                project_create_request
                    .project_directory_path
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects".to_string())
            );
            assert_eq!(project_create_request.project_name, Some("UnitTestProject".to_string()));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_rename_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project",
            "rename",
            "--project-directory-path",
            "C:\\Projects\\OldProject",
            "--new-project-name",
            "RenamedProject",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Rename { project_rename_request }) => {
            assert_eq!(
                project_rename_request
                    .project_directory_path
                    .display()
                    .to_string(),
                "C:\\Projects\\OldProject".to_string()
            );
            assert_eq!(project_rename_request.new_project_name, "RenamedProject".to_string());
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_items_activate_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project-items",
            "activate",
            "--project-item-paths",
            "Addresses.Player.Health",
            "--project-item-paths",
            "Addresses.Player.Ammo",
            "--is-activated",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Activate {
            project_items_activate_request,
        }) => {
            assert_eq!(project_items_activate_request.project_item_paths.len(), 2);
            assert_eq!(project_items_activate_request.project_item_paths[0], "Addresses.Player.Health".to_string());
            assert_eq!(project_items_activate_request.project_item_paths[1], "Addresses.Player.Ammo".to_string());
            assert!(project_items_activate_request.is_activated);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_open_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project",
            "open",
            "--open-file-browser",
            "--project-directory-path",
            "C:\\Projects\\OpenProject",
            "--project-name",
            "OpenProject",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Open { project_open_request }) => {
            assert!(project_open_request.open_file_browser);
            assert_eq!(
                project_open_request
                    .project_directory_path
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects\\OpenProject".to_string())
            );
            assert_eq!(project_open_request.project_name, Some("OpenProject".to_string()));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_delete_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project",
            "delete",
            "--project-directory-path",
            "C:\\Projects\\DeleteProject",
            "--project-name",
            "DeleteProject",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Delete { project_delete_request }) => {
            assert_eq!(
                project_delete_request
                    .project_directory_path
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects\\DeleteProject".to_string())
            );
            assert_eq!(project_delete_request.project_name, Some("DeleteProject".to_string()));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_export_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project",
            "export",
            "--project-directory-path",
            "C:\\Projects\\ExportProject",
            "--project-name",
            "ExportProject",
            "--open-export-folder",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Export { project_export_request }) => {
            assert_eq!(
                project_export_request
                    .project_directory_path
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects\\ExportProject".to_string())
            );
            assert_eq!(project_export_request.project_name, Some("ExportProject".to_string()));
            assert!(project_export_request.open_export_folder);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_list_subcommand() {
    let parse_result = std::panic::catch_unwind(|| UnprivilegedCommand::from_iter_safe(["squalr-cli", "project", "list"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::List { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_close_subcommand() {
    let parse_result = std::panic::catch_unwind(|| UnprivilegedCommand::from_iter_safe(["squalr-cli", "project", "close"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Close { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_save_subcommand() {
    let parse_result = std::panic::catch_unwind(|| UnprivilegedCommand::from_iter_safe(["squalr-cli", "project", "save"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Save { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_general_settings_list_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "settings", "general", "list"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::General {
            general_settings_command: GeneralSettingsCommand::List { .. },
        }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_memory_settings_list_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "settings", "memory", "list"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::Memory {
            memory_settings_command: MemorySettingsCommand::List { .. },
        }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_settings_list_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "settings", "scan", "list"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::Scan {
            scan_settings_command: ScanSettingsCommand::List { .. },
        }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_items_list_subcommand() {
    let parse_result = std::panic::catch_unwind(|| UnprivilegedCommand::from_iter_safe(["squalr-cli", "project-items", "list"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::List { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}
