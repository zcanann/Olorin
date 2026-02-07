use crossbeam_channel::{Receiver, unbounded};
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::trackable_tasks::cancel::trackable_tasks_cancel_request::TrackableTasksCancelRequest;
use squalr_engine_api::commands::trackable_tasks::cancel::trackable_tasks_cancel_response::TrackableTasksCancelResponse;
use squalr_engine_api::commands::trackable_tasks::list::trackable_tasks_list_request::TrackableTasksListRequest;
use squalr_engine_api::commands::trackable_tasks::list::trackable_tasks_list_response::TrackableTasksListResponse;
use squalr_engine_api::commands::trackable_tasks::trackable_tasks_command::TrackableTasksCommand;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse};
use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
use squalr_engine_api::engine::engine_unprivileged_state::EngineUnprivilegedState;
use squalr_engine_api::events::engine_event::EngineEvent;
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
fn trackable_tasks_list_request_dispatches_list_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        TrackableTasksListResponse {}.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let trackable_tasks_list_request = TrackableTasksListRequest {};

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    trackable_tasks_list_request.send_unprivileged(&bindings, move |_trackable_tasks_list_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::TrackableTasks(TrackableTasksCommand::List { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn trackable_tasks_list_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        TrackableTasksCancelResponse {}.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let trackable_tasks_list_request = TrackableTasksListRequest {};

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    trackable_tasks_list_request.send_unprivileged(&bindings, move |_trackable_tasks_list_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::TrackableTasks(TrackableTasksCommand::List { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn trackable_tasks_cancel_request_dispatches_cancel_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        TrackableTasksCancelResponse {}.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let trackable_tasks_cancel_request = TrackableTasksCancelRequest {
        task_id: "task-42".to_string(),
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    trackable_tasks_cancel_request.send_unprivileged(&bindings, move |_trackable_tasks_cancel_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::TrackableTasks(TrackableTasksCommand::Cancel {
            trackable_tasks_cancel_request: captured_trackable_tasks_cancel_request,
        }) => {
            assert_eq!(captured_trackable_tasks_cancel_request.task_id, "task-42".to_string());
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn trackable_tasks_cancel_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        TrackableTasksListResponse {}.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let trackable_tasks_cancel_request = TrackableTasksCancelRequest {
        task_id: "task-13".to_string(),
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    trackable_tasks_cancel_request.send_unprivileged(&bindings, move |_trackable_tasks_cancel_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::TrackableTasks(TrackableTasksCommand::Cancel {
            trackable_tasks_cancel_request: captured_trackable_tasks_cancel_request,
        }) => {
            assert_eq!(captured_trackable_tasks_cancel_request.task_id, "task-13".to_string());
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
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
