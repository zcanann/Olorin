use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button},
    views::project_explorer::project_selector::view_data::project_selector_view_data::ProjectSelectorViewData,
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, vec2};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectHierarchyToolbarView {
    app_context: Arc<AppContext>,
}

impl ProjectHierarchyToolbarView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let instance = Self { app_context };

        instance
    }
}

impl Widget for ProjectHierarchyToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let height = 28.0;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), height), Sense::empty());
        let theme = &self.app_context.theme;

        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        // Create a child ui constrained to the title bar.
        let builder = UiBuilder::new()
            .max_rect(allocated_size_rectangle)
            .layout(Layout::left_to_right(Align::Center));
        let mut toolbar_user_interface = user_interface.new_child(builder);

        toolbar_user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
            let button_size = vec2(36.0, 28.0);

            // Close project.
            let button_refresh = user_interface.add_sized(
                button_size,
                Button::new_from_theme(&theme)
                    .with_tooltip_text("Close this project.")
                    .background_color(Color32::TRANSPARENT),
            );
            IconDraw::draw(user_interface, button_refresh.rect, &theme.icon_library.icon_handle_close);

            if button_refresh.clicked() {
                ProjectSelectorViewData::close_current_project(self.app_context.clone());
            }
        });

        response
    }
}
