use crossbeam_channel::{Receiver, unbounded};
use squalr_engine_api::commands::memory::memory_command::MemoryCommand;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::commands::process::process_command::ProcessCommand;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::new::scan_new_response::ScanNewResponse;
use squalr_engine_api::commands::scan::scan_command::ScanCommand;
use squalr_engine_api::commands::settings::general::general_settings_command::GeneralSettingsCommand;
use squalr_engine_api::commands::settings::memory::memory_settings_command::MemorySettingsCommand;
use squalr_engine_api::commands::settings::scan::scan_settings_command::ScanSettingsCommand;
use squalr_engine_api::commands::settings::settings_command::SettingsCommand;
use squalr_engine_api::commands::trackable_tasks::trackable_tasks_command::TrackableTasksCommand;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_response::UnprivilegedCommandResponse;
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use squalr_engine_api::events::engine_event::EngineEvent;
use squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

struct MockEngineBindings {
    dispatched_commands: Arc<Mutex<Vec<PrivilegedCommand>>>,
    response_to_return: PrivilegedCommandResponse,
}

impl MockEngineBindings {
    fn new(response_to_return: PrivilegedCommandResponse) -> Self {
        Self {
            dispatched_commands: Arc::new(Mutex::new(Vec::new())),
            response_to_return,
        }
    }

    fn get_dispatched_commands(&self) -> Arc<Mutex<Vec<PrivilegedCommand>>> {
        self.dispatched_commands.clone()
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
        _engine_command: UnprivilegedCommand,
        _engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        _callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
    ) -> Result<(), String> {
        Err("Unprivileged command dispatch is not used by this test suite.".to_string())
    }

    fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, String> {
        let (_event_sender, event_receiver) = unbounded();

        Ok(event_receiver)
    }
}

#[test]
fn memory_write_request_dispatches_write_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(MemoryWriteResponse { success: true }.to_engine_response());
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
    let bindings = MockEngineBindings::new(MemoryWriteResponse { success: true }.to_engine_response());
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
