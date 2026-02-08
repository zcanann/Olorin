use crate::installer_runtime::start_installer;
use crate::theme::InstallerTheme;
use crate::ui_assets::{InstallerIconLibrary, load_installer_icon_library};
use crate::ui_state::InstallerUiState;
use crate::views::main_window::installer_main_window_view::InstallerMainWindowView;
use eframe::egui;
use eframe::egui::Visuals;
use epaint::Rgba;
use std::sync::{Arc, Mutex};

pub(crate) struct InstallerApp {
    ui_state: Arc<Mutex<InstallerUiState>>,
    installer_theme: InstallerTheme,
    installer_icon_library: Option<InstallerIconLibrary>,
}

impl InstallerApp {
    pub(crate) fn new(
        context: &egui::Context,
        ui_state: Arc<Mutex<InstallerUiState>>,
    ) -> Self {
        let installer_theme = InstallerTheme::default();
        installer_theme.apply(context);
        installer_theme.install_fonts(context);

        let installer_icon_library = load_installer_icon_library(context);
        start_installer(ui_state.clone(), context.clone());

        Self {
            ui_state,
            installer_theme,
            installer_icon_library,
        }
    }
}

impl eframe::App for InstallerApp {
    fn clear_color(
        &self,
        _visuals: &Visuals,
    ) -> [f32; 4] {
        Rgba::TRANSPARENT.to_array()
    }

    fn update(
        &mut self,
        context: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        let state_snapshot = match self.ui_state.lock() {
            Ok(ui_state) => ui_state.clone(),
            Err(_) => InstallerUiState::new(),
        };

        egui::CentralPanel::default().show(context, |ui| {
            let installer_main_window_view = InstallerMainWindowView::new(self.installer_theme.clone(), self.installer_icon_library.clone());
            installer_main_window_view.show(ui, &state_snapshot);
        });
    }
}
