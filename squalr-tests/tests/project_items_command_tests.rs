use crossbeam_channel::{Receiver, unbounded};
use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::project::export::project_export_response::ProjectExportResponse;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_response::ProjectItemsListResponse;
use squalr_engine_api::commands::project_items::project_items_command::ProjectItemsCommand;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use squalr_engine_api::events::engine_event::EngineEvent;
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
fn project_items_list_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectItemsListResponse::default().to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = EngineUnprivilegedState::new(Arc::new(RwLock::new(MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    ))));

    let project_items_list_request = ProjectItemsListRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_items_list_request.send_unprivileged(&bindings, &execution_context, move |project_items_list_response| {
        let response_has_empty_project = project_items_list_response.opened_project_info.is_none() && project_items_list_response.opened_project_root.is_none();
        callback_invoked_clone.store(response_has_empty_project, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::List { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_items_list_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = EngineUnprivilegedState::new(Arc::new(RwLock::new(MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    ))));

    let project_items_list_request = ProjectItemsListRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_items_list_request.send_unprivileged(&bindings, &execution_context, move |_project_items_list_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::List { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
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
