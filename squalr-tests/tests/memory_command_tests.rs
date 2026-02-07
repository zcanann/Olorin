use crossbeam_channel::{Receiver, unbounded};
use squalr_engine_api::commands::memory::memory_command::MemoryCommand;
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use squalr_engine_api::events::engine_event::EngineEvent;
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
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
fn memory_write_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        MemoryReadResponse {
            valued_struct: Default::default(),
            address: 0x2000,
            success: true,
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let memory_write_request = MemoryWriteRequest {
        address: 0x88,
        module_name: "game.exe".to_string(),
        value: vec![9, 8, 7, 6],
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    memory_write_request.send_unprivileged(&bindings, move |_memory_write_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Memory(MemoryCommand::Write {
            memory_write_request: captured_memory_write_request,
        }) => {
            assert_eq!(captured_memory_write_request.address, 0x88);
            assert_eq!(captured_memory_write_request.module_name, "game.exe".to_string());
            assert_eq!(captured_memory_write_request.value, vec![9, 8, 7, 6]);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn memory_read_request_dispatches_read_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryReadResponse {
            valued_struct: Default::default(),
            address: 0x1234,
            success: true,
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let memory_read_request = MemoryReadRequest {
        address: 0x1234,
        module_name: "kernel32.dll".to_string(),
        symbolic_struct_definition: SymbolicStructDefinition::new(String::new(), vec![]),
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    memory_read_request.send_unprivileged(&bindings, move |memory_read_response| {
        callback_invoked_clone.store(memory_read_response.success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Memory(MemoryCommand::Read {
            memory_read_request: captured_memory_read_request,
        }) => {
            assert_eq!(captured_memory_read_request.address, 0x1234);
            assert_eq!(captured_memory_read_request.module_name, "kernel32.dll".to_string());
            assert_eq!(
                captured_memory_read_request
                    .symbolic_struct_definition
                    .get_symbol_namespace(),
                ""
            );
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn memory_read_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let memory_read_request = MemoryReadRequest {
        address: 0x5678,
        module_name: "game.exe".to_string(),
        symbolic_struct_definition: SymbolicStructDefinition::new(String::new(), vec![]),
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    memory_read_request.send_unprivileged(&bindings, move |_memory_read_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Memory(MemoryCommand::Read {
            memory_read_request: captured_memory_read_request,
        }) => {
            assert_eq!(captured_memory_read_request.address, 0x5678);
            assert_eq!(captured_memory_read_request.module_name, "game.exe".to_string());
            assert_eq!(
                captured_memory_read_request
                    .symbolic_struct_definition
                    .get_symbol_namespace(),
                ""
            );
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
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
