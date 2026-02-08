#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::NativeOptions;
use eframe::egui;
use eframe::egui::{Align, Color32, Frame, Layout, Margin, RichText, ScrollArea, Stroke, ViewportBuilder};
use eframe::epaint::CornerRadius;
use log::{Level, Log, Metadata, Record, SetLoggerError};
use squalr_engine::app_provisioner::app_provisioner_config::AppProvisionerConfig;
use squalr_engine::app_provisioner::installer::app_installer::AppInstaller;
use squalr_engine::app_provisioner::installer::install_phase::InstallPhase;
use squalr_engine::app_provisioner::installer::install_progress::InstallProgress;
use squalr_engine::app_provisioner::operations::launch::update_operation_launch::UpdateOperationLaunch;
use squalr_engine::app_provisioner::progress_tracker::ProgressTracker;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const APP_NAME: &str = "Squalr Installer";
const MAX_LOG_BUFFER_BYTES: usize = 256 * 1024;

#[derive(Clone)]
struct InstallerUiState {
    installer_phase: InstallPhase,
    installer_progress: f32,
    installer_progress_string: String,
    install_complete: bool,
    installer_logs: String,
}

impl InstallerUiState {
    fn new() -> Self {
        Self {
            installer_phase: InstallPhase::Download,
            installer_progress: 0.0,
            installer_progress_string: "0%".to_string(),
            install_complete: false,
            installer_logs: String::new(),
        }
    }

    fn set_progress(
        &mut self,
        install_progress: InstallProgress,
    ) {
        self.installer_phase = install_progress.phase;
        self.installer_progress = install_progress.progress_percent.clamp(0.0, 1.0);
        self.installer_progress_string = format!("{:.0}%", self.installer_progress * 100.0);
        self.install_complete = install_progress.phase == InstallPhase::Complete;
    }

    fn append_log(
        &mut self,
        log_message: &str,
    ) {
        self.installer_logs.push_str(log_message);
        trim_log_buffer(&mut self.installer_logs, MAX_LOG_BUFFER_BYTES);
    }
}

struct InstallerLogger {
    ui_state: Arc<Mutex<InstallerUiState>>,
}

impl InstallerLogger {
    fn new(ui_state: Arc<Mutex<InstallerUiState>>) -> Self {
        Self { ui_state }
    }
}

impl Log for InstallerLogger {
    fn enabled(
        &self,
        metadata: &Metadata,
    ) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(
        &self,
        record: &Record,
    ) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let log_message = format!("[{}] {}\n", record.level(), record.args());

        if let Ok(mut ui_state) = self.ui_state.lock() {
            ui_state.append_log(&log_message);
        }
    }

    fn flush(&self) {}
}

struct InstallerApp {
    ui_state: Arc<Mutex<InstallerUiState>>,
    installer_theme: InstallerTheme,
}

impl InstallerApp {
    fn new(
        context: &egui::Context,
        ui_state: Arc<Mutex<InstallerUiState>>,
    ) -> Self {
        let installer_theme = InstallerTheme::default();
        installer_theme.apply(context);
        start_installer(ui_state.clone());
        Self { ui_state, installer_theme }
    }

    fn launch_app(&self) {
        match AppProvisionerConfig::get_default_install_dir() {
            Ok(app_install_directory) => {
                let executable_path = app_install_directory.join("squalr.exe");
                UpdateOperationLaunch::launch_app(&executable_path);
            }
            Err(error) => {
                log::error!("Failed to resolve install directory: {}", error);
            }
        }
    }
}

#[derive(Clone)]
struct InstallerTheme {
    color_background_primary: Color32,
    color_background_panel: Color32,
    color_background_control: Color32,
    color_background_control_primary: Color32,
    color_background_control_primary_dark: Color32,
    color_background_control_success: Color32,
    color_background_control_success_dark: Color32,
    color_foreground: Color32,
    color_foreground_preview: Color32,
    color_border_blue: Color32,
    color_border_panel: Color32,
    color_log_background: Color32,
    corner_radius_panel: u8,
}

impl Default for InstallerTheme {
    fn default() -> Self {
        Self {
            color_background_primary: Color32::from_rgb(0x33, 0x33, 0x33),
            color_background_panel: Color32::from_rgb(0x27, 0x27, 0x27),
            color_background_control: Color32::from_rgb(0x44, 0x44, 0x44),
            color_background_control_primary: Color32::from_rgb(0x1E, 0x54, 0x92),
            color_background_control_primary_dark: Color32::from_rgb(0x06, 0x1E, 0x3E),
            color_background_control_success: Color32::from_rgb(0x14, 0xA4, 0x4D),
            color_background_control_success_dark: Color32::from_rgb(0x0E, 0x72, 0x36),
            color_foreground: Color32::WHITE,
            color_foreground_preview: Color32::from_rgb(0xAF, 0xAF, 0xAF),
            color_border_blue: Color32::from_rgb(0x00, 0x7A, 0xCC),
            color_border_panel: Color32::from_rgb(0x20, 0x1C, 0x1C),
            color_log_background: Color32::from_rgb(0x1E, 0x1E, 0x1E),
            corner_radius_panel: 8,
        }
    }
}

impl InstallerTheme {
    fn apply(
        &self,
        context: &egui::Context,
    ) {
        let mut style = (*context.style()).clone();
        let visuals = &mut style.visuals;

        visuals.override_text_color = Some(self.color_foreground);
        visuals.panel_fill = self.color_background_primary;
        visuals.window_fill = self.color_background_panel;
        visuals.faint_bg_color = self.color_background_panel;
        visuals.extreme_bg_color = self.color_background_control;
        visuals.code_bg_color = self.color_log_background;
        visuals.window_stroke = Stroke::new(1.0, self.color_border_panel);
        visuals.selection.bg_fill = self.color_border_blue;
        visuals.selection.stroke = Stroke::new(1.0, self.color_border_blue);

        visuals.widgets.noninteractive.bg_fill = self.color_background_panel;
        visuals.widgets.noninteractive.weak_bg_fill = self.color_background_panel;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.color_border_panel);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.color_foreground_preview);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.inactive.bg_fill = self.color_background_control_primary;
        visuals.widgets.inactive.weak_bg_fill = self.color_background_control_primary_dark;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.color_border_blue);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.hovered.bg_fill = self.color_border_blue;
        visuals.widgets.hovered.weak_bg_fill = self.color_background_control_primary;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.color_border_blue);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.active.bg_fill = self.color_background_control_primary_dark;
        visuals.widgets.active.weak_bg_fill = self.color_background_control_primary_dark;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.color_border_blue);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.active.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.open.bg_fill = self.color_background_control;
        visuals.widgets.open.weak_bg_fill = self.color_background_control;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, self.color_border_panel);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.open.corner_radius = CornerRadius::same(self.corner_radius_panel);

        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        style.spacing.window_margin = Margin::same(12);
        context.set_style(style);
    }
}

fn install_phase_string(install_phase: InstallPhase) -> &'static str {
    match install_phase {
        InstallPhase::Download => "Downloading installer package.",
        InstallPhase::Extraction => "Extracting installation payload.",
        InstallPhase::Complete => "Installation complete.",
    }
}

impl eframe::App for InstallerApp {
    fn update(
        &mut self,
        context: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        context.request_repaint_after(Duration::from_millis(50));

        let state_snapshot = match self.ui_state.lock() {
            Ok(ui_state) => ui_state.clone(),
            Err(_) => InstallerUiState::new(),
        };

        let progress_status_text = if state_snapshot.install_complete {
            "Squalr installed successfully."
        } else {
            install_phase_string(state_snapshot.installer_phase)
        };

        egui::TopBottomPanel::top("header")
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(self.installer_theme.color_background_primary)
                    .stroke(Stroke::new(1.0, self.installer_theme.color_border_blue))
                    .inner_margin(Margin::symmetric(14, 10)),
            )
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Squalr").strong().size(22.0));
                        ui.label(
                            RichText::new("Installer")
                                .size(13.0)
                                .color(self.installer_theme.color_foreground_preview),
                        );
                    });
                });
            });

        egui::TopBottomPanel::bottom("footer")
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(self.installer_theme.color_background_primary)
                    .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                    .inner_margin(Margin::symmetric(14, 10)),
            )
            .show(context, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("Install location: default user application directory.")
                            .color(self.installer_theme.color_foreground_preview)
                            .size(12.0),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if state_snapshot.install_complete && ui.button("Launch Squalr").clicked() {
                            self.launch_app();
                        }
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(self.installer_theme.color_background_panel)
                    .inner_margin(Margin::same(14)),
            )
            .show(context, |ui| {
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    Frame::new()
                        .fill(self.installer_theme.color_background_primary)
                        .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                        .corner_radius(CornerRadius::same(self.installer_theme.corner_radius_panel))
                        .inner_margin(Margin::same(12))
                        .show(ui, |ui| {
                            ui.label(RichText::new("Installation Status").strong().size(15.0));
                            ui.label(
                                RichText::new(progress_status_text)
                                    .size(13.0)
                                    .color(self.installer_theme.color_foreground_preview),
                            );
                            ui.add_space(4.0);

                            ui.add(
                                egui::ProgressBar::new(state_snapshot.installer_progress)
                                    .fill(if state_snapshot.install_complete {
                                        self.installer_theme.color_background_control_success
                                    } else {
                                        self.installer_theme.color_background_control_primary
                                    })
                                    .text(state_snapshot.installer_progress_string),
                            );
                        });

                    Frame::new()
                        .fill(self.installer_theme.color_background_primary)
                        .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                        .corner_radius(CornerRadius::same(self.installer_theme.corner_radius_panel))
                        .inner_margin(Margin::same(12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Installer Log").strong().size(14.0));
                                if state_snapshot.install_complete {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new("Ready")
                                                .size(12.0)
                                                .color(self.installer_theme.color_background_control_success_dark),
                                        );
                                    });
                                }
                            });

                            Frame::new()
                                .fill(self.installer_theme.color_log_background)
                                .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                                .corner_radius(CornerRadius::same(self.installer_theme.corner_radius_panel))
                                .inner_margin(Margin::same(10))
                                .show(ui, |ui| {
                                    ui.set_min_height(290.0);
                                    ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                                        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                                            ui.label(
                                                RichText::new(state_snapshot.installer_logs.as_str())
                                                    .monospace()
                                                    .size(12.0)
                                                    .color(self.installer_theme.color_foreground_preview),
                                            );
                                        });
                                    });
                                });
                        });
                });
            });
    }
}

fn trim_log_buffer(
    log_buffer: &mut String,
    max_buffer_bytes: usize,
) {
    if log_buffer.len() <= max_buffer_bytes {
        return;
    }

    let mut trim_start_index = log_buffer.len().saturating_sub(max_buffer_bytes);
    while trim_start_index < log_buffer.len() && !log_buffer.is_char_boundary(trim_start_index) {
        trim_start_index += 1;
    }

    log_buffer.drain(..trim_start_index);
}

fn initialize_logger(ui_state: Arc<Mutex<InstallerUiState>>) -> Result<(), SetLoggerError> {
    let logger = InstallerLogger::new(ui_state);
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}

fn start_installer(ui_state: Arc<Mutex<InstallerUiState>>) {
    let progress_tracker = ProgressTracker::new();
    let progress_receiver = progress_tracker.subscribe();
    let progress_ui_state = ui_state.clone();

    thread::spawn(move || {
        for install_progress in progress_receiver {
            if let Ok(mut state) = progress_ui_state.lock() {
                state.set_progress(install_progress);
            }
        }
    });

    match AppProvisionerConfig::get_default_install_dir() {
        Ok(install_directory) => {
            AppInstaller::run_installation(install_directory, progress_tracker);
        }
        Err(error) => {
            log::error!("Failed to resolve install directory: {}", error);
        }
    }
}

pub fn main() {
    let ui_state = Arc::new(Mutex::new(InstallerUiState::new()));

    if let Err(error) = initialize_logger(ui_state.clone()) {
        eprintln!("Failed to initialize installer logger: {}", error);
    }

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([640.0, 640.0])
            .with_min_inner_size([480.0, 420.0]),
        ..NativeOptions::default()
    };

    let run_result = eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(move |creation_context| Ok(Box::new(InstallerApp::new(&creation_context.egui_ctx, ui_state.clone())))),
    );

    if let Err(error) = run_result {
        eprintln!("Fatal error starting installer GUI: {}", error);
    }
}

#[cfg(test)]
mod tests {
    use super::trim_log_buffer;

    #[test]
    fn trim_log_buffer_keeps_recent_log_data() {
        let mut log_buffer = "line-1\nline-2\nline-3\n".to_string();
        trim_log_buffer(&mut log_buffer, 8);
        assert_eq!(log_buffer, "\nline-3\n");
    }

    #[test]
    fn trim_log_buffer_does_not_modify_when_under_limit() {
        let mut log_buffer = "line-1\n".to_string();
        trim_log_buffer(&mut log_buffer, 1024);
        assert_eq!(log_buffer, "line-1\n");
    }
}
