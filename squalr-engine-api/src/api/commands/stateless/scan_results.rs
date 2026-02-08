use crate::api::commands::stateless::scan::ScanSessionHandle;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::scan_results::scan_result::ScanResult;
use crate::structures::scan_results::scan_result_ref::ScanResultRef;
use serde::{Deserialize, Serialize};

/// Stateless scan results command request payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessScanResultsRequest {
    List(ScanResultsListRequest),
    Query(ScanResultsQueryRequest),
    Refresh(ScanResultsRefreshRequest),
    AddToProject(ScanResultsAddToProjectRequest),
    Freeze(ScanResultsFreezeRequest),
    SetProperty(ScanResultsSetPropertyRequest),
    Delete(ScanResultsDeleteRequest),
}

/// Stateless scan results command response payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessScanResultsResponse {
    List(ScanResultsListResponse),
    Query(ScanResultsQueryResponse),
    Refresh(ScanResultsRefreshResponse),
    AddToProject(ScanResultsAddToProjectResponse),
    Freeze(ScanResultsFreezeResponse),
    SetProperty(ScanResultsSetPropertyResponse),
    Delete(ScanResultsDeleteResponse),
}

/// Stateless scan results list request.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanResultsListRequest {
    pub session_handle: ScanSessionHandle,
    pub page_index: u64,
}

/// Stateless scan results list response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsListResponse {
    pub session_handle: ScanSessionHandle,
    pub scan_results: Vec<ScanResult>,
    pub page_index: u64,
    pub last_page_index: u64,
    pub page_size: u64,
    pub result_count: u64,
    pub total_size_in_bytes: u64,
}

/// Stateless scan results query request.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanResultsQueryRequest {
    pub session_handle: ScanSessionHandle,
    pub page_index: u64,
}

/// Stateless scan results query response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsQueryResponse {
    pub session_handle: ScanSessionHandle,
    pub scan_results: Vec<ScanResult>,
    pub page_index: u64,
    pub last_page_index: u64,
    pub page_size: u64,
    pub result_count: u64,
    pub total_size_in_bytes: u64,
}

/// Stateless scan results refresh request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsRefreshRequest {
    pub session_handle: ScanSessionHandle,
    pub scan_result_refs: Vec<ScanResultRef>,
}

/// Stateless scan results refresh response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsRefreshResponse {
    pub session_handle: ScanSessionHandle,
    pub scan_results: Vec<ScanResult>,
}

/// Stateless scan results add-to-project request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsAddToProjectRequest {
    pub session_handle: ScanSessionHandle,
    pub scan_result_refs: Vec<ScanResultRef>,
}

/// Stateless scan results add-to-project response.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanResultsAddToProjectResponse {
    pub session_handle: ScanSessionHandle,
}

/// Stateless scan results freeze request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsFreezeRequest {
    pub session_handle: ScanSessionHandle,
    pub scan_result_refs: Vec<ScanResultRef>,
    pub is_frozen: bool,
}

/// Stateless scan results freeze response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsFreezeResponse {
    pub session_handle: ScanSessionHandle,
    pub failed_freeze_toggle_scan_result_refs: Vec<ScanResultRef>,
}

/// Stateless scan results set-property request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsSetPropertyRequest {
    pub session_handle: ScanSessionHandle,
    pub scan_result_refs: Vec<ScanResultRef>,
    pub anonymous_value_string: AnonymousValueString,
    pub field_namespace: String,
}

/// Stateless scan results set-property response.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanResultsSetPropertyResponse {
    pub session_handle: ScanSessionHandle,
}

/// Stateless scan results delete request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsDeleteRequest {
    pub session_handle: ScanSessionHandle,
    pub scan_result_refs: Vec<ScanResultRef>,
}

/// Stateless scan results delete response.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanResultsDeleteResponse {
    pub session_handle: ScanSessionHandle,
}
