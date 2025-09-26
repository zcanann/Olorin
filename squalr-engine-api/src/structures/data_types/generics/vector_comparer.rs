use crate::structures::data_types::data_type::DataType;
use crate::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use crate::structures::scanning::comparisons::scan_function_vector::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;
use std::simd::{LaneCount, Simd, SupportedLaneCount};
use std::sync::Arc;

/// A wrapper function to re-genericize vector functions on `DataType` structs for use by scanners.
/// This is necessary because all `DataType` instances need to be implemented by the traits that define them.
/// Due to Rust limitations, these traits cannot have generics, so explicit 64/32/16 byte vector functions are implemented.
/// However, our scanners are generic, so we need to "get back to" generics, and this is how we do it.
pub trait VectorComparer<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    fn get_vector_compare_func_immediate(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>;

    fn get_vector_compare_func_relative(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>;

    fn get_vector_compare_func_delta(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>;
}

impl VectorComparer<64> for LaneCount<64> {
    fn get_vector_compare_func_immediate(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64> {
        VectorCompareWrapper64::get_vector_compare_func_immediate(data_type, scan_compare_type_immediate, mapped_scan_parameters)
    }

    fn get_vector_compare_func_relative(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative64> {
        VectorCompareWrapper64::get_vector_compare_func_relative(data_type, scan_compare_type_relative, mapped_scan_parameters)
    }

    fn get_vector_compare_func_delta(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        VectorCompareWrapper64::get_vector_compare_func_delta(data_type, scan_compare_type_delta, mapped_scan_parameters)
    }
}

impl VectorComparer<32> for LaneCount<32> {
    fn get_vector_compare_func_immediate(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32> {
        VectorCompareWrapper32::get_vector_compare_func_immediate(data_type, scan_compare_type_immediate, mapped_scan_parameters)
    }

    fn get_vector_compare_func_relative(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative32> {
        VectorCompareWrapper32::get_vector_compare_func_relative(data_type, scan_compare_type_relative, mapped_scan_parameters)
    }

    fn get_vector_compare_func_delta(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        VectorCompareWrapper32::get_vector_compare_func_delta(data_type, scan_compare_type_delta, mapped_scan_parameters)
    }
}

impl VectorComparer<16> for LaneCount<16> {
    fn get_vector_compare_func_immediate(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16> {
        VectorCompareWrapper16::get_vector_compare_func_immediate(data_type, scan_compare_type_immediate, mapped_scan_parameters)
    }

    fn get_vector_compare_func_relative(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative16> {
        VectorCompareWrapper16::get_vector_compare_func_relative(data_type, scan_compare_type_relative, mapped_scan_parameters)
    }

    fn get_vector_compare_func_delta(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        VectorCompareWrapper16::get_vector_compare_func_delta(data_type, scan_compare_type_delta, mapped_scan_parameters)
    }
}

trait VectorCompareWrapper<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    fn get_vector_compare_func_immediate(
        data_type: &Arc<dyn DataType>,
        compare_type_immediate: &ScanCompareTypeImmediate,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>;

    fn get_vector_compare_func_relative(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>;

    fn get_vector_compare_func_delta(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>;
}
struct VectorCompareWrapper64 {}

impl VectorCompareWrapper<64> for VectorCompareWrapper64 {
    fn get_vector_compare_func_immediate(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate64> {
        data_type.get_vector_compare_func_immediate_64(scan_compare_type_immediate, mapped_scan_parameters)
    }

    fn get_vector_compare_func_relative(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative64> {
        data_type.get_vector_compare_func_relative_64(scan_compare_type_relative, mapped_scan_parameters)
    }

    fn get_vector_compare_func_delta(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta64> {
        data_type.get_vector_compare_func_delta_64(scan_compare_type_delta, mapped_scan_parameters)
    }
}

struct VectorCompareWrapper32 {}

impl VectorCompareWrapper<32> for VectorCompareWrapper32 {
    fn get_vector_compare_func_immediate(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate32> {
        data_type.get_vector_compare_func_immediate_32(scan_compare_type_immediate, mapped_scan_parameters)
    }

    fn get_vector_compare_func_relative(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative32> {
        data_type.get_vector_compare_func_relative_32(scan_compare_type_relative, mapped_scan_parameters)
    }

    fn get_vector_compare_func_delta(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta32> {
        data_type.get_vector_compare_func_delta_32(scan_compare_type_delta, mapped_scan_parameters)
    }
}

struct VectorCompareWrapper16 {}

impl VectorCompareWrapper<16> for VectorCompareWrapper16 {
    fn get_vector_compare_func_immediate(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnImmediate16> {
        data_type.get_vector_compare_func_immediate_16(scan_compare_type_immediate, mapped_scan_parameters)
    }

    fn get_vector_compare_func_relative(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnRelative16> {
        data_type.get_vector_compare_func_relative_16(scan_compare_type_relative, mapped_scan_parameters)
    }

    fn get_vector_compare_func_delta(
        data_type: &Arc<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Option<VectorCompareFnDelta16> {
        data_type.get_vector_compare_func_delta_16(scan_compare_type_delta, mapped_scan_parameters)
    }
}
