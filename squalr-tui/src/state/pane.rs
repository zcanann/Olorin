/// Enumerates all top-level panes in keyboard focus order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TuiPane {
    ProcessSelector,
    ElementScanner,
    ScanResults,
    ProjectExplorer,
    StructViewer,
    Output,
    Settings,
}

impl TuiPane {
    pub fn all_panes() -> [TuiPane; 7] {
        [
            TuiPane::ProcessSelector,
            TuiPane::ElementScanner,
            TuiPane::ScanResults,
            TuiPane::ProjectExplorer,
            TuiPane::StructViewer,
            TuiPane::Output,
            TuiPane::Settings,
        ]
    }

    pub fn title(self) -> &'static str {
        match self {
            TuiPane::ProcessSelector => "Process Selector",
            TuiPane::ElementScanner => "Element Scanner",
            TuiPane::ScanResults => "Scan Results",
            TuiPane::ProjectExplorer => "Project Explorer",
            TuiPane::StructViewer => "Struct Viewer",
            TuiPane::Output => "Output",
            TuiPane::Settings => "Settings",
        }
    }

    pub fn shortcut_digit(self) -> char {
        match self {
            TuiPane::ProcessSelector => '1',
            TuiPane::ElementScanner => '2',
            TuiPane::ScanResults => '3',
            TuiPane::ProjectExplorer => '4',
            TuiPane::StructViewer => '5',
            TuiPane::Output => '6',
            TuiPane::Settings => '7',
        }
    }

    pub fn from_shortcut_digit(shortcut_digit: char) -> Option<TuiPane> {
        match shortcut_digit {
            '1' => Some(TuiPane::ProcessSelector),
            '2' => Some(TuiPane::ElementScanner),
            '3' => Some(TuiPane::ScanResults),
            '4' => Some(TuiPane::ProjectExplorer),
            '5' => Some(TuiPane::StructViewer),
            '6' => Some(TuiPane::Output),
            '7' => Some(TuiPane::Settings),
            _ => None,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            TuiPane::ProcessSelector => 0,
            TuiPane::ElementScanner => 1,
            TuiPane::ScanResults => 2,
            TuiPane::ProjectExplorer => 3,
            TuiPane::StructViewer => 4,
            TuiPane::Output => 5,
            TuiPane::Settings => 6,
        }
    }
}

impl Default for TuiPane {
    fn default() -> Self {
        TuiPane::ProcessSelector
    }
}
