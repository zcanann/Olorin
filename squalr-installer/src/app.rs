use crate::installer_runtime::{install_phase_string, installer_status_string, launch_app, start_installer};
use crate::theme::InstallerTheme;
use crate::ui_assets::{InstallerIconLibrary, draw_icon, load_installer_icon_library};
use crate::ui_state::InstallerUiState;
use eframe::egui;
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, ScrollArea, Sense, Stroke, Visuals};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
        start_installer(ui_state.clone());

        Self {
            ui_state,
            installer_theme,
            installer_icon_library,
        }
    }

    fn render_installer_title_bar(
        &self,
        context: &egui::Context,
    ) {
        egui::TopBottomPanel::top("title_bar")
            .exact_height(self.installer_theme.title_bar_height)
            .resizable(false)
            .frame(Frame::new().fill(Color32::TRANSPARENT))
            .show(context, |ui| {
                let title_bar_rectangle = ui.max_rect();
                ui.painter().rect_filled(
                    title_bar_rectangle,
                    CornerRadius {
                        nw: self.installer_theme.corner_radius_panel,
                        ne: self.installer_theme.corner_radius_panel,
                        sw: 0,
                        se: 0,
                    },
                    self.installer_theme.color_background_primary,
                );

                let mut button_cluster_rectangle: Option<egui::Rect> = None;
                ui.allocate_ui_with_layout(title_bar_rectangle.size(), Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(8.0);

                    if let Some(installer_icon_library) = &self.installer_icon_library {
                        ui.add(egui::Image::new((installer_icon_library.app.id(), egui::vec2(18.0, 18.0))));
                        ui.add_space(4.0);
                    }

                    ui.label(
                        RichText::new(crate::ui_assets::APP_NAME)
                            .font(self.installer_theme.fonts.font_window_title.clone())
                            .color(self.installer_theme.color_foreground),
                    );

                    ui.add_space(ui.available_width());

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let button_size = egui::vec2(36.0, self.installer_theme.title_bar_height);

                        let close_button_response = ui.add_sized(button_size, egui::Button::new("").frame(false));
                        if let Some(installer_icon_library) = &self.installer_icon_library {
                            draw_icon(ui, close_button_response.rect, &installer_icon_library.close);
                        }
                        if close_button_response.clicked() {
                            context.send_viewport_cmd(ViewportCommand::Close);
                        }

                        let maximize_button_response = ui.add_sized(button_size, egui::Button::new("").frame(false));
                        if let Some(installer_icon_library) = &self.installer_icon_library {
                            draw_icon(ui, maximize_button_response.rect, &installer_icon_library.maximize);
                        }
                        if maximize_button_response.clicked() {
                            let currently_maximized = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));
                            context.send_viewport_cmd(ViewportCommand::Maximized(!currently_maximized));
                        }

                        let minimize_button_response = ui.add_sized(button_size, egui::Button::new("").frame(false));
                        if let Some(installer_icon_library) = &self.installer_icon_library {
                            draw_icon(ui, minimize_button_response.rect, &installer_icon_library.minimize);
                        }
                        if minimize_button_response.clicked() {
                            context.send_viewport_cmd(ViewportCommand::Minimized(true));
                        }

                        button_cluster_rectangle = Some(
                            close_button_response
                                .rect
                                .union(maximize_button_response.rect)
                                .union(minimize_button_response.rect),
                        );
                    });
                });

                let drag_region_right_edge = button_cluster_rectangle
                    .map(|rectangle| rectangle.min.x)
                    .unwrap_or(title_bar_rectangle.max.x);
                let drag_region = egui::Rect::from_min_max(title_bar_rectangle.min, egui::pos2(drag_region_right_edge, title_bar_rectangle.max.y));
                let drag_response = ui.interact(drag_region, ui.id().with("installer_title_bar_drag"), Sense::click_and_drag());

                if drag_response.drag_started() {
                    context.send_viewport_cmd(ViewportCommand::StartDrag);
                }

                if drag_response.double_clicked() {
                    let currently_maximized = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));
                    context.send_viewport_cmd(ViewportCommand::Maximized(!currently_maximized));
                }
            });
    }

    fn render_installer_footer(
        &self,
        context: &egui::Context,
        install_complete: bool,
    ) {
        egui::TopBottomPanel::bottom("footer")
            .exact_height(self.installer_theme.footer_height)
            .resizable(false)
            .frame(Frame::new().fill(Color32::TRANSPARENT))
            .show(context, |ui| {
                let footer_rectangle = ui.max_rect();
                ui.painter().rect_filled(
                    footer_rectangle,
                    CornerRadius {
                        nw: 0,
                        ne: 0,
                        sw: self.installer_theme.corner_radius_panel,
                        se: self.installer_theme.corner_radius_panel,
                    },
                    self.installer_theme.color_border_blue,
                );

                ui.allocate_ui_with_layout(footer_rectangle.size(), Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Install location: default user application directory.")
                            .font(self.installer_theme.fonts.font_normal.clone())
                            .color(self.installer_theme.color_foreground),
                    );
                    ui.add_space(ui.available_width());

                    if install_complete && ui.button("Launch Squalr").clicked() {
                        launch_app();
                    }
                });
            });
    }
}

fn log_color_for_line(
    log_line: &str,
    installer_theme: &InstallerTheme,
) -> Color32 {
    if log_line.starts_with("[ERROR]") {
        installer_theme.color_foreground_error
    } else if log_line.starts_with("[WARN]") {
        installer_theme.color_foreground_warning
    } else if log_line.starts_with("[DEBUG]") {
        installer_theme.color_foreground_debug
    } else if log_line.starts_with("[TRACE]") {
        installer_theme.color_foreground_trace
    } else if log_line.starts_with("[INFO]") {
        installer_theme.color_foreground_info
    } else {
        installer_theme.color_foreground
    }
}

impl eframe::App for InstallerApp {
    fn clear_color(
        &self,
        _visuals: &Visuals,
    ) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

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

        let progress_status_text = install_phase_string(state_snapshot.installer_phase);
        let header_status_text = installer_status_string(&state_snapshot);

        self.render_installer_title_bar(context);
        self.render_installer_footer(context, state_snapshot.install_complete);

        let app_frame = Frame::new()
            .fill(self.installer_theme.color_background_panel)
            .corner_radius(CornerRadius::same(self.installer_theme.corner_radius_panel))
            .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
            .outer_margin(2.0)
            .inner_margin(Margin::same(8));

        egui::CentralPanel::default()
            .frame(app_frame)
            .show(context, |ui| {
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.label(
                        RichText::new(header_status_text)
                            .font(self.installer_theme.fonts.font_window_title.clone())
                            .color(self.installer_theme.color_foreground),
                    );

                    Frame::new()
                        .fill(self.installer_theme.color_background_primary)
                        .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("Installation Status")
                                    .font(self.installer_theme.fonts.font_window_title.clone())
                                    .color(self.installer_theme.color_foreground),
                            );
                            ui.label(
                                RichText::new(progress_status_text)
                                    .font(self.installer_theme.fonts.font_normal.clone())
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
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Installer Log")
                                        .font(self.installer_theme.fonts.font_window_title.clone())
                                        .color(self.installer_theme.color_foreground),
                                );
                                if state_snapshot.install_complete {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new("Ready")
                                                .font(self.installer_theme.fonts.font_normal.clone())
                                                .color(self.installer_theme.color_background_control_success_dark),
                                        );
                                    });
                                }
                            });

                            Frame::new()
                                .fill(self.installer_theme.color_log_background)
                                .stroke(Stroke::new(1.0, self.installer_theme.color_border_panel))
                                .inner_margin(Margin::same(8))
                                .show(ui, |ui| {
                                    ui.set_min_height(290.0);
                                    ui.set_min_width(ui.available_width());
                                    ScrollArea::vertical()
                                        .id_salt("installer_log_scroll")
                                        .auto_shrink([false, false])
                                        .stick_to_bottom(true)
                                        .show(ui, |ui| {
                                            if state_snapshot.installer_logs.is_empty() {
                                                ui.label(
                                                    RichText::new("Waiting for installer output.")
                                                        .font(self.installer_theme.fonts.font_ubuntu_mono_normal.clone())
                                                        .color(self.installer_theme.color_foreground_preview),
                                                );
                                            } else {
                                                for installer_log_line in state_snapshot.installer_logs.lines() {
                                                    ui.label(
                                                        RichText::new(installer_log_line)
                                                            .font(self.installer_theme.fonts.font_normal.clone())
                                                            .color(log_color_for_line(installer_log_line, &self.installer_theme)),
                                                    );
                                                }
                                            }
                                        });
                                });
                        });
                });
            });
    }
}
