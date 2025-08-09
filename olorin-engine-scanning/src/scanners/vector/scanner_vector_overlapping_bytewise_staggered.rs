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

pub struct ScannerVectorOverlappingBytewiseStaggered<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>, {}

impl<const N: usize> ScannerVectorOverlappingBytewiseStaggered<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn encode_results(
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size_padding: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
        vector_compare_size: u64,
    ) {
        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_result.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(vector_compare_size);
            // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_result.simd_eq(false_mask).all() {
            run_length_encoder.finalize_current_encode_with_padding(vector_compare_size, data_type_size_padding);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            Self::encode_remainder_results(
                &compare_result,
                run_length_encoder,
                data_type_size_padding,
                vector_compare_size,
                vector_compare_size,
            );
        }
    }

    fn encode_remainder_results(
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size_padding: u64,
        remainder_bytes: u64,
        vector_compare_size: u64,
    ) {
        let start_byte_index = vector_compare_size.saturating_sub(remainder_bytes);

        for byte_index in start_byte_index..vector_compare_size {
            if compare_result[byte_index as usize] != 0 {
                run_length_encoder.encode_range(1);
            } else {
                run_length_encoder.finalize_current_encode_with_padding(1, data_type_size_padding);
            }
        }
    }
}

/// Implements a memory region scanner that is optmized for scanning for an overlapping sequence of N bytes.
/// For example, even scanning for something like `00 01 02 03`
impl<const N: usize> Scanner for ScannerVectorOverlappingBytewiseStaggered<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn get_scanner_name(&self) -> &'static str {
        &"Vector Overlapping (Bytewise Staggered)"
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
        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Simd::<u8, N>::splat(0xFF);

        let data_type_ref = mapped_scan_parameters.get_data_type_ref();
        let data_type_size = symbol_registry_guard.get_unit_size_in_bytes(data_type_ref);
        let data_type_size_padding = data_type_size.saturating_sub(mapped_scan_parameters.get_memory_alignment() as u64);
        let memory_alignment = mapped_scan_parameters.get_memory_alignment();
        let memory_alignment_size = memory_alignment as u64;

        let vector_size_in_bytes = N;
        let vector_underflow = data_type_size as usize;
        let vector_compare_size = vector_size_in_bytes.saturating_sub(vector_underflow) as u64;
        let element_count = snapshot_region_filter.get_element_count(data_type_registry, data_type_ref, memory_alignment);
        let vectorizable_iterations = region_size / vector_compare_size; // JIRA: Memory alignment!
        let remainder_bytes = region_size % vector_compare_size;
        let remainder_element_count: u64 = (remainder_bytes / memory_alignment_size).saturating_sub(data_type_size.saturating_sub(1));
        let vectorizable_element_count = element_count.saturating_sub(remainder_element_count);

        let scan_immedate = mapped_scan_parameters.get_data_value();
        let scan_compare_type_immediate = match mapped_scan_parameters.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => scan_compare_type_immediate,
            _ => {
                log::error!("Invalid scan compare type provided to bytewise staggered scan.");
                return vec![];
            }
        };
        let check_equal = match scan_compare_type_immediate {
            ScanCompareTypeImmediate::Equal => true,
            ScanCompareTypeImmediate::NotEqual => false,
            _ => {
                log::error!("Invalid scan compare immediate type provided to bytewise staggered scan.");
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

        // JIRA: Memory alignment!
        match data_type_size {
            2 => {
                let compare_func_byte_0 = load_nth_byte_vec(&scan_immedate, 0);
                let compare_func_byte_1 = load_nth_byte_vec(&scan_immedate, 1);

                // Compare as many full vectors as we can.
                for index in 0..vectorizable_iterations {
                    let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                    let compare_results_0 = compare_func_byte_0(current_values_pointer);
                    let compare_results_1 = VectorGenerics::rotate_left_with_discard::<N, 1>(compare_func_byte_1(current_values_pointer));
                    let compare_result = compare_results_0 & compare_results_1;

                    Self::encode_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size_padding,
                        true_mask,
                        false_mask,
                        vector_compare_size,
                    );
                }
            }
            4 => {
                let compare_func_byte_0 = load_nth_byte_vec(&scan_immedate, 0);
                let compare_func_byte_1 = load_nth_byte_vec(&scan_immedate, 1);
                let compare_func_byte_2 = load_nth_byte_vec(&scan_immedate, 2);
                let compare_func_byte_3 = load_nth_byte_vec(&scan_immedate, 3);

                // Compare as many full vectors as we can.
                for index in 0..vectorizable_iterations {
                    let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                    let compare_results_0 = compare_func_byte_0(current_values_pointer);
                    let compare_results_1 = VectorGenerics::rotate_left_with_discard::<N, 1>(compare_func_byte_1(current_values_pointer));
                    let compare_results_2 = VectorGenerics::rotate_left_with_discard::<N, 2>(compare_func_byte_2(current_values_pointer));
                    let compare_results_3 = VectorGenerics::rotate_left_with_discard::<N, 3>(compare_func_byte_3(current_values_pointer));
                    let compare_result = compare_results_0 & compare_results_1 & compare_results_2 & compare_results_3;

                    Self::encode_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size_padding,
                        true_mask,
                        false_mask,
                        vector_compare_size,
                    );
                }
            }
            8 => {
                let compare_func_byte_0 = load_nth_byte_vec(&scan_immedate, 0);
                let compare_func_byte_1 = load_nth_byte_vec(&scan_immedate, 1);
                let compare_func_byte_2 = load_nth_byte_vec(&scan_immedate, 2);
                let compare_func_byte_3 = load_nth_byte_vec(&scan_immedate, 3);
                let compare_func_byte_4 = load_nth_byte_vec(&scan_immedate, 4);
                let compare_func_byte_5 = load_nth_byte_vec(&scan_immedate, 5);
                let compare_func_byte_6 = load_nth_byte_vec(&scan_immedate, 6);
                let compare_func_byte_7 = load_nth_byte_vec(&scan_immedate, 7);

                // Compare as many full vectors as we can.
                for index in 0..vectorizable_iterations {
                    let current_values_pointer = unsafe { current_values_pointer.add((index * vector_compare_size) as usize) };
                    let compare_results_0 = compare_func_byte_0(current_values_pointer);
                    let compare_results_1 = VectorGenerics::rotate_left_with_discard::<N, 1>(compare_func_byte_1(current_values_pointer));
                    let compare_results_2 = VectorGenerics::rotate_left_with_discard::<N, 2>(compare_func_byte_2(current_values_pointer));
                    let compare_results_3 = VectorGenerics::rotate_left_with_discard::<N, 3>(compare_func_byte_3(current_values_pointer));
                    let compare_results_4 = VectorGenerics::rotate_left_with_discard::<N, 4>(compare_func_byte_4(current_values_pointer));
                    let compare_results_5 = VectorGenerics::rotate_left_with_discard::<N, 5>(compare_func_byte_5(current_values_pointer));
                    let compare_results_6 = VectorGenerics::rotate_left_with_discard::<N, 6>(compare_func_byte_6(current_values_pointer));
                    let compare_results_7 = VectorGenerics::rotate_left_with_discard::<N, 7>(compare_func_byte_7(current_values_pointer));
                    let compare_result = compare_results_0
                        & compare_results_1
                        & compare_results_2
                        & compare_results_3
                        & compare_results_4
                        & compare_results_5
                        & compare_results_6
                        & compare_results_7;

                    Self::encode_results(
                        &compare_result,
                        &mut run_length_encoder,
                        data_type_size_padding,
                        true_mask,
                        false_mask,
                        vector_compare_size,
                    );
                }
            }
            _ => {
                log::error!("Unsupported data type size provided to 2-periodic scan!");
                return vec![];
            }
        }

        // Handle remainder elements.
        if let Some(compare_func) =
            symbol_registry_guard.get_scalar_compare_func_immediate(data_type_ref, &scan_compare_type_immediate, mapped_scan_parameters)
        {
            for index in vectorizable_element_count..element_count {
                let current_value_pointer = unsafe { current_values_pointer.add(index as usize * memory_alignment_size as usize) };
                let compare_result = compare_func(current_value_pointer);

                if compare_result {
                    run_length_encoder.encode_range(memory_alignment_size);
                } else {
                    run_length_encoder.finalize_current_encode_with_padding(memory_alignment_size, data_type_size_padding);
                }
            }
        }

        run_length_encoder.finalize_current_encode_with_padding(0, data_type_size_padding);
        run_length_encoder.take_result_regions()
    }
}
