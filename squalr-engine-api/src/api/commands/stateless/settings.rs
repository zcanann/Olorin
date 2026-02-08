use crate::commands::settings::settings_error::SettingsError;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::memory_read_mode::MemoryReadMode;
use crate::structures::settings::general_settings::GeneralSettings;
use crate::structures::settings::memory_settings::MemorySettings;
use crate::structures::settings::scan_settings::ScanSettings;
use serde::{Deserialize, Serialize};

/// Stateless settings command request payloads.
///
/// Settings are engine-global configuration and do not require an additional session handle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessSettingsRequest {
    General(GeneralSettingsRequest),
    Memory(MemorySettingsRequest),
    Scan(ScanSettingsRequest),
}

/// Stateless settings command response payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatelessSettingsResponse {
    General(GeneralSettingsResponse),
    Memory(MemorySettingsResponse),
    Scan(ScanSettingsResponse),
}

/// Stateless general-settings request payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GeneralSettingsRequest {
    List(GeneralSettingsListRequest),
    Set(GeneralSettingsSetRequest),
}

/// Stateless general-settings response payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GeneralSettingsResponse {
    List(GeneralSettingsListResponse),
    Set(GeneralSettingsSetResponse),
}

/// Stateless memory-settings request payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemorySettingsRequest {
    List(MemorySettingsListRequest),
    Set(MemorySettingsSetRequest),
}

/// Stateless memory-settings response payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MemorySettingsResponse {
    List(MemorySettingsListResponse),
    Set(MemorySettingsSetResponse),
}

/// Stateless scan-settings request payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanSettingsRequest {
    List(ScanSettingsListRequest),
    Set(ScanSettingsSetRequest),
}

/// Stateless scan-settings response payloads.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanSettingsResponse {
    List(ScanSettingsListResponse),
    Set(ScanSettingsSetResponse),
}

/// Stateless general-settings list request.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneralSettingsListRequest {}

/// Stateless general-settings list response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralSettingsListResponse {
    pub general_settings: Result<GeneralSettings, SettingsError>,
}

/// Stateless general-settings set request.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneralSettingsSetRequest {
    pub engine_request_delay: Option<u64>,
}

/// Stateless general-settings set response.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneralSettingsSetResponse {}

/// Stateless memory-settings list request.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemorySettingsListRequest {}

/// Stateless memory-settings list response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemorySettingsListResponse {
    pub memory_settings: Result<MemorySettings, SettingsError>,
}

/// Stateless memory-settings set request.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemorySettingsSetRequest {
    pub memory_type_none: Option<bool>,
    pub memory_type_private: Option<bool>,
    pub memory_type_image: Option<bool>,
    pub memory_type_mapped: Option<bool>,
    pub required_write: Option<bool>,
    pub required_execute: Option<bool>,
    pub required_copy_on_write: Option<bool>,
    pub excluded_write: Option<bool>,
    pub excluded_execute: Option<bool>,
    pub excluded_copy_on_write: Option<bool>,
    pub start_address: Option<u64>,
    pub end_address: Option<u64>,
    pub only_query_usermode: Option<bool>,
}

/// Stateless memory-settings set response.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemorySettingsSetResponse {}

/// Stateless scan-settings list request.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanSettingsListRequest {}

/// Stateless scan-settings list response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanSettingsListResponse {
    pub scan_settings: Result<ScanSettings, SettingsError>,
}

/// Stateless scan-settings set request.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ScanSettingsSetRequest {
    pub results_page_size: Option<u32>,
    pub results_read_interval_ms: Option<u64>,
    pub project_read_interval_ms: Option<u64>,
    pub freeze_interval_ms: Option<u64>,
    pub memory_alignment: Option<MemoryAlignment>,
    pub memory_read_mode: Option<MemoryReadMode>,
    pub floating_point_tolerance: Option<FloatingPointTolerance>,
    pub is_single_threaded_scan: Option<bool>,
    pub debug_perform_validation_scan: Option<bool>,
}

/// Stateless scan-settings set response.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanSettingsSetResponse {}
