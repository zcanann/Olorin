use crate::state::pane::TuiPane;

const FOCUSED_PANE_WEIGHT_BOOST: u16 = 2;

pub fn pane_layout_weights(
    panes: &[TuiPane],
    focused_pane: TuiPane,
) -> Vec<u16> {
    panes
        .iter()
        .map(|pane| {
            let mut pane_weight = pane.base_layout_weight();
            if *pane == focused_pane {
                pane_weight = pane_weight.saturating_add(FOCUSED_PANE_WEIGHT_BOOST);
            }

            pane_weight
        })
        .collect()
}

trait PaneLayoutWeight {
    fn base_layout_weight(self) -> u16;
}

impl PaneLayoutWeight for TuiPane {
    fn base_layout_weight(self) -> u16 {
        match self {
            TuiPane::ProcessSelector => 3,
            TuiPane::ProjectExplorer => 5,
            TuiPane::Settings => 2,
            TuiPane::ElementScanner => 4,
            TuiPane::ScanResults => 6,
            TuiPane::StructViewer => 5,
            TuiPane::Output => 3,
        }
    }
}
