use crate::scanners::snapshot_scanner::Scanner;
use olorin_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use olorin_engine_api::structures::scanning::comparisons::scan_function_scalar::ScanFunctionScalar;
use olorin_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use olorin_engine_api::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;
use olorin_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::sync::{Arc, RwLock};

pub struct ScannerScalarSingleElement {}

/// Implements a scalar (ie CPU bound, non-SIMD) scanner which only scans a single element of memory (ie only containing 1 data type).
impl Scanner for ScannerScalarSingleElement {
    fn get_scanner_name(&self) -> &'static str {
        &"Single Element"
    }

    fn scan_region(
        &self,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let mut compare_result = false;

        if let Some(scalar_compare_func) = mapped_scan_parameters.get_scan_function_scalar(symbol_registry) {
            match scalar_compare_func {
                ScanFunctionScalar::Immediate(compare_func) => {
                    let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);

                    compare_result = compare_func(current_value_pointer);
                }
                ScanFunctionScalar::RelativeOrDelta(compare_func) => {
                    let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
                    let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);

                    compare_result = compare_func(current_value_pointer, previous_value_pointer);
                }
            }
        }

        if compare_result {
            vec![SnapshotRegionFilter::new(
                snapshot_region_filter.get_base_address(),
                snapshot_region_filter.get_region_size(),
            )]
        } else {
            vec![]
        }
    }
}
