use squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;
use squalr_engine_api::structures::settings::scan_settings::ScanSettings;
use serde_json::to_string_pretty;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::sync::{Arc, RwLock};

pub struct ScanSettingsConfig {
    config: Arc<RwLock<ScanSettings>>,
    config_file: PathBuf,
}

impl ScanSettingsConfig {
    fn new() -> Self {
        let config_file = Self::default_config_path();
        let config = if config_file.exists() {
            match fs::read_to_string(&config_file) {
                Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
                Err(_) => ScanSettings::default(),
            }
        } else {
            ScanSettings::default()
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            config_file,
        }
    }

    fn get_instance() -> &'static ScanSettingsConfig {
        static mut INSTANCE: Option<ScanSettingsConfig> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let instance = ScanSettingsConfig::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    fn default_config_path() -> PathBuf {
        std::env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or(&Path::new(""))
            .join("scan_settings.json")
    }

    fn save_config() {
        if let Ok(config) = Self::get_instance().config.read() {
            if let Ok(json) = to_string_pretty(&*config) {
                let _ = fs::write(&Self::get_instance().config_file, json);
            }
        }
    }

    pub fn get_full_config() -> &'static Arc<RwLock<ScanSettings>> {
        &Self::get_instance().config
    }

    pub fn get_results_page_size() -> u32 {
        if let Ok(config) = Self::get_instance().config.read() {
            config.results_page_size
        } else {
            ScanSettings::default().results_page_size
        }
    }

    pub fn set_results_page_size(value: u32) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.results_page_size = value;
        }

        Self::save_config();
    }

    pub fn get_results_read_interval() -> u64 {
        if let Ok(config) = Self::get_instance().config.read() {
            config.results_read_interval_ms
        } else {
            ScanSettings::default().results_read_interval_ms
        }
    }

    pub fn set_results_read_interval(value: u64) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.results_read_interval_ms = value;
        }

        Self::save_config();
    }

    pub fn get_project_read_interval() -> u64 {
        if let Ok(config) = Self::get_instance().config.read() {
            config.project_read_interval_ms
        } else {
            ScanSettings::default().project_read_interval_ms
        }
    }

    pub fn set_project_read_interval(value: u64) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.project_read_interval_ms = value;
        }

        Self::save_config();
    }

    pub fn get_freeze_interval() -> u64 {
        if let Ok(config) = Self::get_instance().config.read() {
            config.freeze_interval_ms
        } else {
            ScanSettings::default().freeze_interval_ms
        }
    }

    pub fn set_freeze_interval(value: u64) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.freeze_interval_ms = value;
        }

        Self::save_config();
    }

    pub fn get_memory_alignment() -> Option<MemoryAlignment> {
        if let Ok(config) = Self::get_instance().config.read() {
            config.memory_alignment
        } else {
            ScanSettings::default().memory_alignment
        }
    }

    pub fn set_memory_alignment(value: Option<MemoryAlignment>) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.memory_alignment = value;
        }

        Self::save_config();
    }

    pub fn get_memory_read_mode() -> MemoryReadMode {
        if let Ok(config) = Self::get_instance().config.read() {
            config.memory_read_mode
        } else {
            ScanSettings::default().memory_read_mode
        }
    }

    pub fn set_memory_read_mode(value: MemoryReadMode) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.memory_read_mode = value;
        }

        Self::save_config();
    }

    pub fn get_floating_point_tolerance() -> FloatingPointTolerance {
        if let Ok(config) = Self::get_instance().config.read() {
            config.floating_point_tolerance
        } else {
            ScanSettings::default().floating_point_tolerance
        }
    }

    pub fn set_floating_point_tolerance(value: FloatingPointTolerance) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.floating_point_tolerance = value;
        }

        Self::save_config();
    }

    pub fn get_is_single_threaded_scan() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.is_single_threaded_scan
        } else {
            ScanSettings::default().is_single_threaded_scan
        }
    }

    pub fn get_debug_perform_validation_scan() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.debug_perform_validation_scan
        } else {
            ScanSettings::default().debug_perform_validation_scan
        }
    }
}
