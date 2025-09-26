use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::data_types::generics::vector_comparer::VectorComparer;
use squalr_engine_api::structures::data_types::generics::vector_generics::VectorGenerics;
use squalr_engine_api::structures::scanning::comparisons::scan_function_scalar::ScanFunctionScalar;
use squalr_engine_api::structures::scanning::comparisons::scan_function_vector::ScanFunctionVector;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::simd::cmp::SimdPartialEq;
use std::simd::{LaneCount, Simd, SupportedLaneCount};
use std::sync::{Arc, RwLock};

pub struct ScannerVectorOverlapping<const N: usize>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>, {}

impl<const N: usize> ScannerVectorOverlapping<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    /// Produces a mask chunked up into `data_type_size` chunks, with the first N bytes of each chunk set to 0xFF up to the `memory_alignment_size`.
    /// 4-byte align 1 -> 0xFF 0x00 0x00 0x00..
    /// 4-byte align 2 -> 0xFF 0xFF 0x00 0x00..
    /// 2-byte align 1 -> 0xFF 0x00..
    pub fn get_element_wise_mask(
        data_type_size: u64,
        memory_alignment_size: u64,
    ) -> Simd<u8, N> {
        let mut mask = [0u8; N];

        for start_index in (0..N).step_by(data_type_size as usize) {
            for align_index in 0..memory_alignment_size {
                mask[start_index + align_index as usize] = 0xFF;
            }
        }

        Simd::from_array(mask)
    }

    fn encode_results(
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size_padding: u64,
        true_mask: Simd<u8, N>,
        false_mask: Simd<u8, N>,
    ) {
        // Optimization: Check if all scan results are true. This helps substantially when scanning for common values like 0.
        if compare_result.simd_eq(true_mask).all() {
            run_length_encoder.encode_range(N as u64);
        // Optimization: Check if all scan results are false. This is also a very common result, and speeds up scans.
        } else if compare_result.simd_eq(false_mask).all() {
            run_length_encoder.finalize_current_encode_with_padding(N as u64, data_type_size_padding);
        // Otherwise, there is a mix of true/false results that need to be processed manually.
        } else {
            Self::encode_remainder_results(&compare_result, run_length_encoder, data_type_size_padding, 0);
        }
    }

    fn encode_remainder_results(
        compare_result: &Simd<u8, N>,
        run_length_encoder: &mut SnapshotRegionFilterRunLengthEncoder,
        data_type_size_padding: u64,
        start_byte_index: u64,
    ) {
        for byte_index in start_byte_index..(N as u64) {
            if compare_result[byte_index as usize] != 0 {
                run_length_encoder.encode_range(1);
            } else {
                run_length_encoder.finalize_current_encode_with_padding(1, data_type_size_padding);
            }
        }
    }
}

/// Implements a CPU-bound SIMD memory region scanner that is optmized for scanning for an overlapping sequence of N bytes.
/// In other words, this scan efficiently handles searching for values where the data type size is larger than the memory alignment.
impl<const N: usize> Scanner for ScannerVectorOverlapping<N>
where
    LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
{
    fn get_scanner_name(&self) -> &'static str {
        &"Vector Overlapping"
    }

    /// Performs a sequential iteration over a region of memory, performing the scan comparison.
    /// A run-length encoding algorithm is used to generate new sub-regions as the scan progresses.
    fn scan_region(
        &self,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return vec![];
            }
        };
        let current_values_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
        let previous_values_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);
        let base_address = snapshot_region_filter.get_base_address();

        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let data_type_ref = mapped_scan_parameters.get_data_type_ref();
        let data_type_size = symbol_registry_guard.get_unit_size_in_bytes(data_type_ref);
        let memory_alignment = mapped_scan_parameters.get_memory_alignment();
        let memory_alignment_size = memory_alignment as u64;
        let data_type_size_padding = data_type_size.saturating_sub(memory_alignment_size);

        let vector_size_in_bytes = N as u64;
        let element_count = snapshot_region_filter.get_element_count(symbol_registry, data_type_ref, memory_alignment);
        let elements_per_vector = vector_size_in_bytes / memory_alignment_size;
        let vectorizable_iterations = element_count / elements_per_vector;
        let vector_element_count = vectorizable_iterations * elements_per_vector;

        let false_mask = Simd::<u8, N>::splat(0x00);
        let true_mask = Simd::<u8, N>::splat(0xFF);
        let element_wise_mask = Self::get_element_wise_mask(data_type_size, memory_alignment_size);

        debug_assert!(vectorizable_iterations > 0);
        debug_assert!(data_type_size > memory_alignment_size);
        debug_assert!(memory_alignment_size == 1 || memory_alignment_size == 2 || memory_alignment_size == 4);

        if let Some(vector_compare_func) = mapped_scan_parameters.get_scan_function_vector(symbol_registry) {
            match vector_compare_func {
                ScanFunctionVector::Immediate(compare_func) => {
                    // Compare as many full vectors as we can.
                    for index in 0..vectorizable_iterations {
                        let current_values_pointer = unsafe { current_values_pointer.add((index * vector_size_in_bytes) as usize) };
                        let mut compare_result = compare_func(current_values_pointer) & element_wise_mask;

                        for overlap_index in (memory_alignment_size..data_type_size).step_by(memory_alignment_size as usize) {
                            let current_values_pointer = unsafe { current_values_pointer.add(overlap_index as usize) };
                            compare_result |=
                                VectorGenerics::rotate_right_with_discard_max_8::<N>(compare_func(current_values_pointer) & element_wise_mask, overlap_index);
                        }

                        Self::encode_results(&compare_result, &mut run_length_encoder, data_type_size_padding, true_mask, false_mask);
                    }
                }
                ScanFunctionVector::RelativeOrDelta(compare_func) => {
                    // Compare as many full vectors as we can.
                    for index in 0..vectorizable_iterations {
                        let current_values_pointer = unsafe { current_values_pointer.add((index * vector_size_in_bytes) as usize) };
                        let previous_values_pointer = unsafe { previous_values_pointer.add((index * vector_size_in_bytes) as usize) };
                        let mut compare_result = compare_func(current_values_pointer, previous_values_pointer) & element_wise_mask;

                        for overlap_index in (memory_alignment_size..data_type_size).step_by(memory_alignment_size as usize) {
                            let current_values_pointer = unsafe { current_values_pointer.add(overlap_index as usize) };
                            let previous_values_pointer = unsafe { previous_values_pointer.add(overlap_index as usize) };
                            compare_result |= VectorGenerics::rotate_right_with_discard_max_8::<N>(
                                compare_func(current_values_pointer, previous_values_pointer) & element_wise_mask,
                                overlap_index,
                            );
                        }

                        Self::encode_results(&compare_result, &mut run_length_encoder, data_type_size_padding, true_mask, false_mask);
                    }
                }
            }
        }

        if let Some(scalar_compare_func) = mapped_scan_parameters.get_scan_function_scalar(symbol_registry) {
            match scalar_compare_func {
                ScanFunctionScalar::Immediate(compare_func) => {
                    // Handle remainder elements (reverting to scalar comparisons.)
                    for index in vector_element_count..element_count {
                        let current_value_pointer = unsafe { current_values_pointer.add((index * memory_alignment_size) as usize) };
                        let compare_result = compare_func(current_value_pointer);

                        if compare_result {
                            run_length_encoder.encode_range(memory_alignment_size);
                        } else {
                            run_length_encoder.finalize_current_encode_with_padding(memory_alignment_size, data_type_size_padding);
                        }
                    }
                }
                ScanFunctionScalar::RelativeOrDelta(compare_func) => {
                    // Handle remainder elements (reverting to scalar comparisons.)
                    for index in vector_element_count..element_count {
                        let current_value_pointer = unsafe { current_values_pointer.add((index * memory_alignment_size) as usize) };
                        let previous_value_pointer = unsafe { previous_values_pointer.add((index * memory_alignment_size) as usize) };
                        let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                        if compare_result {
                            run_length_encoder.encode_range(memory_alignment_size);
                        } else {
                            run_length_encoder.finalize_current_encode_with_padding(memory_alignment_size, data_type_size_padding);
                        }
                    }
                }
            }
        }

        run_length_encoder.finalize_current_encode_with_padding(0, data_type_size_padding);
        run_length_encoder.take_result_regions()
    }
}
