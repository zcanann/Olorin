use crate::memory_queryer::memory_protection_enum::MemoryProtectionEnum;
use crate::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use crate::memory_queryer::memory_type_enum::MemoryTypeEnum;
use crate::memory_queryer::region_bounds_handling::RegionBoundsHandling;
use crate::normalized_module::NormalizedModule;
use crate::normalized_region::NormalizedRegion;
use core::ffi::c_void;
use core::mem::size_of;
use squalr_engine_processes::process_info::Bitness;
use squalr_engine_processes::process_info::OpenedProcessInfo;

pub struct LinuxMemoryQueryer;

impl LinuxMemoryQueryer {
    pub fn new() -> Self {
        LinuxMemoryQueryer
    }

    fn get_protection_flags(
        &self,
        protection: &MemoryProtectionEnum,
    ) -> u32 {
        0
    }
}

impl IMemoryQueryer for LinuxMemoryQueryer {
    fn get_virtual_pages(
        &self,
        process_info: &OpenedProcessInfo,
        required_protection: MemoryProtectionEnum,
        excluded_protection: MemoryProtectionEnum,
        allowed_types: MemoryTypeEnum,
        start_address: u64,
        end_address: u64,
        region_bounds_handling: RegionBoundsHandling,
    ) -> Vec<NormalizedRegion> {
        vec![]
    }

    fn get_all_virtual_pages(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedRegion> {
        vec![]
    }

    fn is_address_writable(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
    ) -> bool {
        false
    }

    fn get_maximum_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64 {
        0
    }

    fn get_min_usermode_address(
        &self,
        _: &OpenedProcessInfo,
    ) -> u64 {
        0
    }

    fn get_max_usermode_address(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> u64 {
        0
    }

    fn get_modules(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Vec<NormalizedModule> {
        vec![]
    }

    fn address_to_module(
        &self,
        process_info: &OpenedProcessInfo,
        address: u64,
        module_name: &mut String,
    ) -> u64 {
        0
    }

    fn resolve_module(
        &self,
        process_info: &OpenedProcessInfo,
        identifier: &str,
    ) -> u64 {
        0
    }
}
