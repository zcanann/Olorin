use squalr_engine::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use squalr_engine::engine_mode::EngineMode;
use squalr_engine::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::process::close::process_close_request::ProcessCloseRequest;
use squalr_engine_api::commands::process::list::process_list_request::ProcessListRequest;
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use squalr_tests::mocks::mock_os::MockEngineOs;

fn create_test_state() -> (MockEngineOs, std::sync::Arc<EnginePrivilegedState>) {
    let mock_engine_os = MockEngineOs::new();
    let engine_os_providers = mock_engine_os.create_providers();
    let engine_privileged_state = EnginePrivilegedState::new_with_os_providers(EngineMode::Standalone, engine_os_providers);

    (mock_engine_os, engine_privileged_state)
}

fn create_opened_process_info() -> OpenedProcessInfo {
    OpenedProcessInfo::new(std::process::id(), "test-process.exe".to_string(), 0xABC0, Bitness::Bit64, None)
}

#[test]
fn memory_write_executor_uses_injected_module_resolution_and_writer() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("game.exe", 0x1000, 0x2000)]);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());

    let memory_write_request = MemoryWriteRequest {
        address: 0x20,
        module_name: "game.exe".to_string(),
        value: vec![1, 2, 3, 4],
    };

    let memory_write_response = memory_write_request.execute(&engine_privileged_state);
    assert!(memory_write_response.success);

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_write_requests.len(), 1);
    assert_eq!(state_guard.memory_write_requests[0].0, 0x1020);
    assert_eq!(state_guard.memory_write_requests[0].1, vec![1, 2, 3, 4]);
}

#[test]
fn memory_read_executor_uses_injected_module_resolution_and_reader() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_modules(vec![NormalizedModule::new("game.exe", 0x7000, 0x1000)]);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());

    let memory_read_request = MemoryReadRequest {
        address: 0x10,
        module_name: "game.exe".to_string(),
        symbolic_struct_definition: SymbolicStructDefinition::new(String::new(), vec![]),
    };

    let memory_read_response = memory_read_request.execute(&engine_privileged_state);
    assert!(memory_read_response.success);

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.memory_struct_read_addresses.len(), 1);
    assert_eq!(state_guard.memory_struct_read_addresses[0], 0x7010);
}

#[test]
fn process_executors_use_injected_process_provider() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    let process_identifier = std::process::id();
    let process_info = ProcessInfo::new(process_identifier, "calc.exe".to_string(), true, None);

    mock_engine_os.set_processes(vec![process_info.clone()]);
    mock_engine_os.set_opened_process_result(Some(OpenedProcessInfo::new(
        process_identifier,
        "calc.exe".to_string(),
        0xBEEF,
        Bitness::Bit64,
        None,
    )));

    let process_list_request = ProcessListRequest {
        require_windowed: true,
        search_name: Some("calc".to_string()),
        match_case: true,
        limit: Some(5),
        fetch_icons: false,
    };
    let process_list_response = process_list_request.execute(&engine_privileged_state);
    assert_eq!(process_list_response.processes.len(), 1);
    assert_eq!(process_list_response.processes[0].get_name(), "calc.exe");

    let process_open_request = ProcessOpenRequest {
        process_id: Some(process_identifier),
        search_name: Some("calc".to_string()),
        match_case: true,
    };
    let process_open_response = process_open_request.execute(&engine_privileged_state);
    assert!(process_open_response.opened_process_info.is_some());

    let process_close_response = ProcessCloseRequest {}.execute(&engine_privileged_state);
    assert!(process_close_response.process_info.is_some());

    let mock_os_state = mock_engine_os.get_state();
    let state_guard = match mock_os_state.lock() {
        Ok(state_guard) => state_guard,
        Err(error) => panic!("failed to lock mock state: {}", error),
    };
    assert_eq!(state_guard.process_query_requests.len(), 2);
    assert_eq!(state_guard.process_query_requests[0].search_name, Some("calc".to_string()));
    assert!(state_guard.process_query_requests[0].require_windowed);
    assert!(state_guard.process_query_requests[0].match_case);
    assert_eq!(state_guard.process_query_requests[0].limit, Some(5));
    assert!(!state_guard.process_query_requests[0].fetch_icons);
    assert_eq!(state_guard.process_query_requests[1].required_process_id, Some(process_identifier));
    assert_eq!(state_guard.open_process_requests, vec![process_identifier]);
    assert_eq!(state_guard.close_process_handles, vec![0xBEEF]);
}

#[test]
fn scan_new_executor_uses_injected_memory_page_bounds() {
    let (mock_engine_os, engine_privileged_state) = create_test_state();
    mock_engine_os.set_memory_pages(vec![
        NormalizedRegion::new(0x1000, 0x1000),
        NormalizedRegion::new(0x2000, 0x1000),
        NormalizedRegion::new(0x5000, 0x1000),
    ]);
    engine_privileged_state
        .get_process_manager()
        .set_opened_process(create_opened_process_info());

    let _scan_new_response = ScanNewRequest {}.execute(&engine_privileged_state);

    let snapshot_ref = engine_privileged_state.get_snapshot();
    let snapshot_guard = match snapshot_ref.read() {
        Ok(snapshot_guard) => snapshot_guard,
        Err(error) => panic!("failed to lock snapshot for read: {}", error),
    };
    let snapshot_regions = snapshot_guard.get_snapshot_regions();

    assert_eq!(snapshot_regions.len(), 2);
    assert_eq!(snapshot_regions[0].get_base_address(), 0x1000);
    assert_eq!(snapshot_regions[0].get_region_size(), 0x2000);
    assert_eq!(snapshot_regions[0].page_boundaries, vec![0x2000]);
    assert_eq!(snapshot_regions[1].get_base_address(), 0x5000);
    assert_eq!(snapshot_regions[1].get_region_size(), 0x1000);
}
