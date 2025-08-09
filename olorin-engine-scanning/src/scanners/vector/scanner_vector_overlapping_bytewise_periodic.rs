use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use olorin_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use olorin_engine_api::structures::data_types::generics::vector_comparer::VectorComparer;
use olorin_engine_api::structures::data_types::generics::vector_generics::VectorGenerics;
use olorin_engine_api::structures::data_values::data_value::DataValue;
use olorin_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use olorin_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use olorin_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use olorin_engine_api::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;
use olorin_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::ptr;
use std::simd::cmp::SimdPartialEq;
use std::simd::{LaneCount, Simd, SupportedLaneCount};
use std::sync::{Arc, RwLock};

pub struct ScannerVectorOverlappingBytewisePeriodic<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>, {}

impl<const N: usize> ScannerVectorOverlappingBytewisePeriodic<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn encode_results(
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
    ) {
        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_result.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(N as u64);
        // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_result.simd_eq(false_mask).all() {
            run_length_encoder.finalize_current_encode_with_minimum_size_filtering(N as u64, data_type_size);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            Self::encode_remainder_results(&compare_result, run_length_encoder, data_type_size, N as u64);
        }
    }

    fn encode_remainder_results(
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size: u64,
        remainder_bytes: u64,
    ) {
        let start_byte_index = N.saturating_sub(remainder_bytes as usize);

        for byte_index in start_byte_index..N {
            if compare_result[byte_index] != 0 {
                run_length_encoder.encode_range(1);
            } else {
                run_length_encoder.finalize_current_encode_with_minimum_size_filtering(1, data_type_size);
            }
        }
    }
}

/// Implements a memory region scanner that is optmized/specialized for a repeated immediate value of the same byte.
/// For example, scanning for an i32 of value 00 00 00 00 can actually be greatly optimized by simply searching for the byte 0!
impl<const N: usize> Scanner for ScannerVectorOverlappingBytewisePeriodic<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn get_scanner_name(&self) -> &'static str {
        &"Vector Overlapping (Bytewise Periodic)"
    }

    fn scan_region(
        &self,
        data_type_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let symbol_registry_guard = match data_type_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return vec![];
            }
        };
        let current_values_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
        let base_address = snapshot_region_filter.get_base_address();
        let region_size = snapshot_region_filter.get_region_size();

        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type_ref = mapped_scan_parameters.get_data_type_ref();
        let data_type_size = symbol_registry_guard.get_unit_size_in_bytes(data_type_ref);

        let vector_size_in_bytes = N as u64;
        let iterations = region_size / vector_size_in_bytes as u64;
        let remainder_bytes = region_size % vector_size_in_bytes as u64;
        let remainder_ptr_offset = (iterations.saturating_sub(1) * vector_size_in_bytes) as usize;

        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Simd::<u8, N>::splat(0xFF);

        let scan_immedate = mapped_scan_parameters.get_data_value();
        let check_equal = match mapped_scan_parameters.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => match scan_compare_type_immediate {
                ScanCompareTypeImmediate::Equal => true,
                ScanCompareTypeImmediate::NotEqual => false,
                _ => {
                    log::error!("Invalid scan compare immediate type provided to bytewise periodic scan.");
                    return vec![];
                }
            },
            _ => {
                log::error!("Invalid scan compare type provided to bytewise periodic scan.");
                return vec![];
            }
        };

        let load_nth_byte_vec = |scan_immedate: &DataValue, byte_index: usize| -> Box<dyn Fn(*const u8) -> Simd<u8, N>> {
            let byte_vec = Simd::<u8, N>::splat(scan_immedate.get_value_bytes()[byte_index]);

            if check_equal {
                Box::new(move |current_values_ptr| {
                    let current_values = unsafe { Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [u8; N])) };
                    VectorGenerics::transmute_mask::<u8, N, N>(current_values.simd_eq(byte_vec))
                })
            } else {
                Box::new(move |current_values_ptr| {
                    let current_values = unsafe { Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [u8; N])) };
                    VectorGenerics::transmute_mask::<u8, N, N>(current_values.simd_ne(byte_vec))
                })
            }
        };

        let periodicity = mapped_scan_parameters.get_periodicity();

        match periodicity {
            1 => {
                let compare_func = load_nth_byte_vec(&scan_immedate, 0);

                // Compare as many full vectors as we can.
                for index in 0..iterations {
                    let current_values_pointer = unsafe { current_values_pointer.add((index * vector_size_in_bytes) as usize) };
                    let compare_result = compare_func(current_values_pointer);

                    Self::encode_results(&compare_result, &mut run_length_encoder, data_type_size, true_mask, false_mask);
                }

                // Handle remainder elements.
                if remainder_bytes > 0 {
                    let current_values_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                    let compare_result = compare_func(current_values_pointer);

                    Self::encode_remainder_results(&compare_result, &mut run_length_encoder, data_type_size, remainder_bytes);
                }
            }
            2 => {
                log::error!("Implementation incomplete!");
                let compare_func_byte_0 = load_nth_byte_vec(&scan_immedate, 0);
                let compare_func_byte_1 = load_nth_byte_vec(&scan_immedate, 1);

                // Compare as many full vectors as we can.
                for index in 0..iterations {
                    let current_values_pointer = unsafe { current_values_pointer.add((index * vector_size_in_bytes) as usize) };
                    let compare_results_0 = compare_func_byte_0(current_values_pointer);
                    let compare_results_1 = compare_func_byte_1(current_values_pointer);
                    let compare_result = compare_results_0 | compare_results_1;

                    Self::encode_results(&compare_result, &mut run_length_encoder, data_type_size, true_mask, false_mask);
                }

                // Handle remainder elements.
                if remainder_bytes > 0 {
                    let compare_results_0 = unsafe { compare_func_byte_0(current_values_pointer.add(remainder_ptr_offset)) };
                    let compare_results_1 = unsafe { compare_func_byte_1(current_values_pointer.add(remainder_ptr_offset)) };
                    let compare_result = compare_results_0 | compare_results_1;

                    Self::encode_remainder_results(&compare_result, &mut run_length_encoder, data_type_size, remainder_bytes);
                }
            }
            4 => {
                log::error!("Implementation incomplete!");
                let compare_func_byte_0 = load_nth_byte_vec(&scan_immedate, 0);
                let compare_func_byte_1 = load_nth_byte_vec(&scan_immedate, 1);
                let compare_func_byte_2 = load_nth_byte_vec(&scan_immedate, 2);
                let compare_func_byte_3 = load_nth_byte_vec(&scan_immedate, 3);

                // Compare as many full vectors as we can.
                for index in 0..iterations {
                    let current_values_pointer = unsafe { current_values_pointer.add((index * vector_size_in_bytes) as usize) };
                    let compare_results_0 = compare_func_byte_0(current_values_pointer);
                    let compare_results_1 = compare_func_byte_1(current_values_pointer);
                    let compare_results_2 = compare_func_byte_2(current_values_pointer);
                    let compare_results_3 = compare_func_byte_3(current_values_pointer);
                    let compare_result = compare_results_0 | compare_results_1 | compare_results_2 | compare_results_3;

                    Self::encode_results(&compare_result, &mut run_length_encoder, data_type_size, true_mask, false_mask);
                }

                // Handle remainder elements.
                if remainder_bytes > 0 {
                    let remainder_value_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                    let compare_results_0 = compare_func_byte_0(remainder_value_pointer);
                    let compare_results_1 = compare_func_byte_1(remainder_value_pointer);
                    let compare_results_2 = compare_func_byte_2(remainder_value_pointer);
                    let compare_results_3 = compare_func_byte_3(remainder_value_pointer);
                    let compare_result = compare_results_0 | compare_results_1 | compare_results_2 | compare_results_3;

                    Self::encode_remainder_results(&compare_result, &mut run_length_encoder, data_type_size, remainder_bytes);
                }
            }
            8 => {
                log::error!("Implementation incomplete!");
                let compare_func_byte_0 = load_nth_byte_vec(&scan_immedate, 0);
                let compare_func_byte_1 = load_nth_byte_vec(&scan_immedate, 1);
                let compare_func_byte_2 = load_nth_byte_vec(&scan_immedate, 2);
                let compare_func_byte_3 = load_nth_byte_vec(&scan_immedate, 3);
                let compare_func_byte_4 = load_nth_byte_vec(&scan_immedate, 4);
                let compare_func_byte_5 = load_nth_byte_vec(&scan_immedate, 5);
                let compare_func_byte_6 = load_nth_byte_vec(&scan_immedate, 6);
                let compare_func_byte_7 = load_nth_byte_vec(&scan_immedate, 7);

                // Compare as many full vectors as we can.
                for index in 0..iterations {
                    let current_values_pointer = unsafe { current_values_pointer.add((index * vector_size_in_bytes) as usize) };
                    let compare_results_0 = compare_func_byte_0(current_values_pointer);
                    let compare_results_1 = compare_func_byte_1(current_values_pointer);
                    let compare_results_2 = compare_func_byte_2(current_values_pointer);
                    let compare_results_3 = compare_func_byte_3(current_values_pointer);
                    let compare_results_4 = compare_func_byte_4(current_values_pointer);
                    let compare_results_5 = compare_func_byte_5(current_values_pointer);
                    let compare_results_6 = compare_func_byte_6(current_values_pointer);
                    let compare_results_7 = compare_func_byte_7(current_values_pointer);
                    let compare_result = compare_results_0
                        | compare_results_1
                        | compare_results_2
                        | compare_results_3
                        | compare_results_4
                        | compare_results_5
                        | compare_results_6
                        | compare_results_7;

                    Self::encode_results(&compare_result, &mut run_length_encoder, data_type_size, true_mask, false_mask);
                }

                // Handle remainder elements.
                if remainder_bytes > 0 {
                    let remainder_value_pointer = unsafe { current_values_pointer.add(remainder_ptr_offset) };
                    let compare_results_0 = compare_func_byte_0(remainder_value_pointer);
                    let compare_results_1 = compare_func_byte_1(remainder_value_pointer);
                    let compare_results_2 = compare_func_byte_2(remainder_value_pointer);
                    let compare_results_3 = compare_func_byte_3(remainder_value_pointer);
                    let compare_results_4 = compare_func_byte_4(remainder_value_pointer);
                    let compare_results_5 = compare_func_byte_5(remainder_value_pointer);
                    let compare_results_6 = compare_func_byte_6(remainder_value_pointer);
                    let compare_results_7 = compare_func_byte_7(remainder_value_pointer);
                    let compare_result = compare_results_0
                        | compare_results_1
                        | compare_results_2
                        | compare_results_3
                        | compare_results_4
                        | compare_results_5
                        | compare_results_6
                        | compare_results_7;

                    Self::encode_remainder_results(&compare_result, &mut run_length_encoder, data_type_size, remainder_bytes);
                }
            }
            _ => {
                log::error!("Unsupported data type size provided to bytewise periodic scan!");
                return vec![];
            }
        }

        run_length_encoder.finalize_current_encode_with_minimum_size_filtering(0, data_type_size);
        run_length_encoder.take_result_regions()
    }
}
