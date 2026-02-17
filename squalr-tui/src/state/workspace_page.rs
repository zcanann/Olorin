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

    pub fn focus_cycle_hint(self) -> &'static str {
        match self {
            Self::ProjectWorkspace => "Process Selector -> Project Explorer -> Output",
            Self::ScannerWorkspace => "Element Scanner -> Scan Results -> Output",
            Self::SettingsWorkspace => "Settings -> Output",
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

#[cfg(test)]
mod tests {
    use super::TuiWorkspacePage;
    use crate::state::pane::TuiPane;

    #[test]
    fn shortcut_digits_map_to_workspace_pages() {
        assert_eq!(TuiWorkspacePage::from_shortcut_digit('1'), Some(TuiWorkspacePage::ProjectWorkspace));
        assert_eq!(TuiWorkspacePage::from_shortcut_digit('2'), Some(TuiWorkspacePage::ScannerWorkspace));
        assert_eq!(TuiWorkspacePage::from_shortcut_digit('3'), Some(TuiWorkspacePage::SettingsWorkspace));
        assert_eq!(TuiWorkspacePage::from_shortcut_digit('4'), None);
    }

    #[test]
    fn visible_panes_are_defined_per_workspace_page() {
        assert_eq!(
            TuiWorkspacePage::ProjectWorkspace.visible_panes(),
            &[
                TuiPane::ProcessSelector,
                TuiPane::ProjectExplorer,
                TuiPane::Output
            ]
        );
        assert_eq!(
            TuiWorkspacePage::ScannerWorkspace.visible_panes(),
            &[TuiPane::ElementScanner, TuiPane::ScanResults, TuiPane::Output]
        );
        assert_eq!(TuiWorkspacePage::SettingsWorkspace.visible_panes(), &[TuiPane::Settings, TuiPane::Output]);
    }
}
