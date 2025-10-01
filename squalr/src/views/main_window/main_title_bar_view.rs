use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::widgets::controls::button::Button;
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{Align, Id, Layout, Rect, Response, RichText, Sense, Ui, UiBuilder, pos2};
use epaint::{Color32, CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct MainTitleBarView {
    app_context: Rc<AppContext>,
    corner_radius: CornerRadius,
    height: f32,
    title: String,
}

impl MainTitleBarView {
    pub fn new(
        app_context: Rc<AppContext>,
        corner_radius: CornerRadius,
        height: f32,
        title: String,
    ) -> Self {
        Self {
            app_context,
            corner_radius,
            height,
            title,
        }
    }
}

impl eframe::egui::Widget for MainTitleBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::empty());
        let theme = &self.app_context.theme;
        let context = &self.app_context.context;

        user_interface.painter().rect_filled(
            available_size_rectangle,
            CornerRadius {
                nw: self.corner_radius.nw,
                ne: self.corner_radius.ne,
                sw: 0,
                se: 0,
            },
            theme.background_primary,
        );

        // Create a child ui constrained to the title bar.
        let builder = UiBuilder::new()
            .max_rect(available_size_rectangle)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);
        let mut buttons_rectangle: Option<Rect> = None;

        // Hard-clip to the titlebar.
        child_user_interface.set_clip_rect(available_size_rectangle);

        // Create the app icon / name.
        child_user_interface.add_space(8.0);

        let [texture_width, texture_height] = theme.icon_library.icon_handle_logo.size();
        let (_id, app_icon_rect) = child_user_interface.allocate_space(vec2(texture_width as f32, texture_height as f32));
        IconDraw::draw(&child_user_interface, app_icon_rect, &theme.icon_library.icon_handle_logo);

        child_user_interface.add_space(4.0);

        child_user_interface.label(
            RichText::new(&self.title)
                .font(theme.font_library.font_noto_sans.font_window_title.clone())
                .color(theme.foreground),
        );

        // Push the rest (buttons) to the far right within the same row.
        child_user_interface.add_space(child_user_interface.available_width());

        // Create the buttons right-to-left.
        child_user_interface.with_layout(Layout::right_to_left(Align::Center), |user_interface| {
            let button_size = vec2(36.0, 32.0);

            // Close.
            let close = user_interface.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));
            IconDraw::draw(user_interface, close.rect, &theme.icon_library.icon_handle_close);

            if close.clicked() {
                context.send_viewport_cmd(ViewportCommand::Close);
            }

            // Maximize / Restore.
            let max = user_interface.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));
            IconDraw::draw(user_interface, max.rect, &theme.icon_library.icon_handle_maximize);

            if max.clicked() {
                let is_max = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));
                context.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
            }

            // Minimize.
            let min = user_interface.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));
            IconDraw::draw(user_interface, min.rect, &theme.icon_library.icon_handle_minimize);

            if min.clicked() {
                context.send_viewport_cmd(ViewportCommand::Minimized(true));
            }

            buttons_rectangle = Some(close.rect.union(max.rect).union(min.rect));
        });

        // Drag area = everything left of the buttons, inside the titlebar rect.
        let right_edge = buttons_rectangle
            .map(|rectangle| rectangle.min.x)
            .unwrap_or(available_size_rectangle.max.x);
        let drag_rect = Rect::from_min_max(available_size_rectangle.min, pos2(right_edge, available_size_rectangle.max.y));
        let drag = user_interface.interact(drag_rect, Id::new("titlebar"), Sense::click_and_drag());

        if drag.drag_started() {
            context.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        if drag.double_clicked() {
            let is_max = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));
            context.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        }

        response
    }
}
