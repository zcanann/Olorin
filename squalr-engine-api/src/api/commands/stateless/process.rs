use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::processes::process_info::ProcessInfo;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Explicit caller-owned process context used by stateless command flows.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProcessSessionHandle {
    pub session_id: Uuid,
}

/// Stateless process command request payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessProcessRequest {
    List(ProcessListRequest),
    Open(ProcessOpenRequest),
    Close(ProcessCloseRequest),
}

/// Stateless process command response payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessProcessResponse {
    List(ProcessListResponse),
    Open(ProcessOpenResponse),
    Close(ProcessCloseResponse),
}

/// Stateless process listing request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessListRequest {
    pub require_windowed: bool,
    pub search_name: Option<String>,
    pub match_case: bool,
    pub limit: Option<u64>,
    pub fetch_icons: bool,
}

/// Stateless process listing response.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProcessListResponse {
    pub processes: Vec<ProcessInfo>,
}

/// Stateless process open request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessOpenRequest {
    pub process_id: Option<u32>,
    pub search_name: Option<String>,
    pub match_case: bool,
}

/// Stateless process open response carrying caller-owned context.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessOpenResponse {
    pub session_handle: Option<ProcessSessionHandle>,
    pub opened_process_info: Option<OpenedProcessInfo>,
}

/// Stateless process close request.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessCloseRequest {
    pub session_handle: ProcessSessionHandle,
}

/// Stateless process close response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessCloseResponse {
    pub session_handle: ProcessSessionHandle,
    pub closed_process_info: Option<OpenedProcessInfo>,
}
