use crate::api::commands::stateless::process::ProcessSessionHandle;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::constraints::anonymous_scan_constraint::AnonymousScanConstraint;
use crate::structures::tasks::trackable_task_handle::TrackableTaskHandle;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Explicit caller-owned scan context used by stateless command flows.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ScanSessionHandle {
    pub session_id: Uuid,
}

/// Stateless scan command request payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessScanRequest {
    New(ScanNewRequest),
    Reset(ScanResetRequest),
    CollectValues(ScanCollectValuesRequest),
    ElementScan(ElementScanRequest),
    PointerScan(PointerScanRequest),
    StructScan(StructScanRequest),
}

/// Stateless scan command response payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessScanResponse {
    New(ScanNewResponse),
    Reset(ScanResetResponse),
    CollectValues(ScanCollectValuesResponse),
    ElementScan(ElementScanResponse),
    PointerScan(PointerScanResponse),
    StructScan(StructScanResponse),
}

/// Stateless scan new request.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanNewRequest {
    pub process_session_handle: ProcessSessionHandle,
}

/// Stateless scan new response carrying caller-owned context.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanNewResponse {
    pub session_handle: Option<ScanSessionHandle>,
}

/// Stateless scan reset request.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanResetRequest {
    pub session_handle: ScanSessionHandle,
}

/// Stateless scan reset response.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanResetResponse {
    pub session_handle: ScanSessionHandle,
    pub success: bool,
}

/// Stateless scan collect values request.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanCollectValuesRequest {
    pub session_handle: ScanSessionHandle,
}

/// Stateless scan collect values response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanCollectValuesResponse {
    pub session_handle: ScanSessionHandle,
    pub trackable_task_handle: Option<TrackableTaskHandle>,
}

/// Stateless element scan request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElementScanRequest {
    pub session_handle: ScanSessionHandle,
    pub scan_constraints: Vec<AnonymousScanConstraint>,
    pub data_type_refs: Vec<DataTypeRef>,
}

/// Stateless element scan response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElementScanResponse {
    pub session_handle: ScanSessionHandle,
    pub trackable_task_handle: Option<TrackableTaskHandle>,
}

/// Stateless pointer scan request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointerScanRequest {
    pub session_handle: ScanSessionHandle,
    pub target_address: AnonymousValueString,
    pub pointer_data_type_ref: DataTypeRef,
    pub max_depth: u64,
    pub offset_size: u64,
}

/// Stateless pointer scan response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointerScanResponse {
    pub session_handle: ScanSessionHandle,
    pub trackable_task_handle: Option<TrackableTaskHandle>,
}

/// Stateless struct scan request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructScanRequest {
    pub session_handle: ScanSessionHandle,
    pub scan_value: Option<AnonymousValueString>,
    pub data_type_ids: Vec<String>,
    pub compare_type: ScanCompareType,
}

/// Stateless struct scan response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructScanResponse {
    pub session_handle: ScanSessionHandle,
    pub trackable_task_handle: Option<TrackableTaskHandle>,
}
