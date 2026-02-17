/// Category selection for settings-pane routing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SettingsCategory {
    #[default]
    General,
    Memory,
    Scan,
}

impl SettingsCategory {
    pub fn all_categories() -> [SettingsCategory; 3] {
        [
            SettingsCategory::General,
            SettingsCategory::Memory,
            SettingsCategory::Scan,
        ]
    }
}

/// Stores state for settings pages and staged changes.
#[derive(Clone, Debug, Default)]
pub struct SettingsPaneState {
    pub selected_category: SettingsCategory,
    pub has_pending_changes: bool,
}
