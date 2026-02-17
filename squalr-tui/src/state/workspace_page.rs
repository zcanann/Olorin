use crate::state::pane::TuiPane;

/// Represents the three full-screen workflow pages in the TUI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TuiWorkspacePage {
    ProjectWorkspace,
    ScannerWorkspace,
    SettingsWorkspace,
}

impl TuiWorkspacePage {
    pub fn from_shortcut_digit(shortcut_digit: char) -> Option<Self> {
        match shortcut_digit {
            '1' => Some(Self::ProjectWorkspace),
            '2' => Some(Self::ScannerWorkspace),
            '3' => Some(Self::SettingsWorkspace),
            _ => None,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::ProjectWorkspace => "Project Workspace",
            Self::ScannerWorkspace => "Scanner Workspace",
            Self::SettingsWorkspace => "Settings Workspace",
        }
    }

    pub fn visible_panes(self) -> &'static [TuiPane] {
        match self {
            Self::ProjectWorkspace => &[
                TuiPane::ProcessSelector,
                TuiPane::ProjectExplorer,
                TuiPane::Output,
            ],
            Self::ScannerWorkspace => &[TuiPane::ElementScanner, TuiPane::ScanResults, TuiPane::Output],
            Self::SettingsWorkspace => &[TuiPane::Settings, TuiPane::Output],
        }
    }
}

impl Default for TuiWorkspacePage {
    fn default() -> Self {
        Self::ProjectWorkspace
    }
}
