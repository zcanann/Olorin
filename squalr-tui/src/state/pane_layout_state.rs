use crate::state::pane::TuiPane;

/// Stores focus and visibility for top-level panes.
#[derive(Clone, Debug)]
pub struct PaneLayoutState {
    pub focused_pane: TuiPane,
    pub pane_visibility: [bool; 7],
}

impl PaneLayoutState {
    pub fn is_pane_visible(
        &self,
        pane: TuiPane,
    ) -> bool {
        self.pane_visibility[pane.to_index()]
    }

    pub fn visible_panes_in_order(&self) -> Vec<TuiPane> {
        TuiPane::all_panes()
            .into_iter()
            .filter(|pane| self.is_pane_visible(*pane))
            .collect()
    }

    pub fn visible_pane_count(&self) -> usize {
        self.visible_panes_in_order().len()
    }

    pub fn show_all_panes(&mut self) {
        self.pane_visibility = [true; 7];
    }

    pub fn set_pane_visibility(
        &mut self,
        pane: TuiPane,
        is_visible: bool,
    ) -> bool {
        if !is_visible && self.visible_pane_count() == 1 && self.is_pane_visible(pane) {
            return false;
        }

        self.pane_visibility[pane.to_index()] = is_visible;

        if !self.is_pane_visible(self.focused_pane) {
            if let Some(first_visible_pane) = self.visible_panes_in_order().first().copied() {
                self.focused_pane = first_visible_pane;
            }
        }

        true
    }

    pub fn toggle_pane_visibility(
        &mut self,
        pane: TuiPane,
    ) -> bool {
        let next_visibility = !self.is_pane_visible(pane);
        self.set_pane_visibility(pane, next_visibility)
    }
}

impl Default for PaneLayoutState {
    fn default() -> Self {
        Self {
            focused_pane: TuiPane::default(),
            pane_visibility: [true; 7],
        }
    }
}
