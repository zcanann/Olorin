use crate::commands::engine_command_request::EngineCommandRequest;
use crate::commands::scan::pointer_scan::pointer_scan_response::PointerScanResponse;
use crate::commands::scan::scan_command::ScanCommand;
use crate::commands::scan::scan_response::ScanResponse;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::{commands::engine_command::EngineCommand, structures::data_types::data_type_ref::DataTypeRef};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct PointerScanRequest {
    #[structopt(short = "a", long)]
    pub target_address: AnonymousValue,
    #[structopt(short = "d", long)]
    pub pointer_data_type_ref: DataTypeRef,
    #[structopt(short = "d", long)]
    pub max_depth: u64,
    #[structopt(short = "o", long)]
    pub offset_size: u64,
}

impl EngineCommandRequest for PointerScanRequest {
    type ResponseType = PointerScanResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::PointerScan {
            pointer_scan_request: self.clone(),
        })
    }
}

impl From<PointerScanResponse> for ScanResponse {
    fn from(pointer_scan_response: PointerScanResponse) -> Self {
        ScanResponse::PointerScan { pointer_scan_response }
    }
}
