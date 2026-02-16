use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::state_layer::StateLayer},
    views::project_explorer::project_hierarchy::view_data::project_hierarchy_frame_action::ProjectHierarchyFrameAction,
};
use eframe::egui::{Align2, Rect, Response, Sense, TextureHandle, Ui, Widget, pos2, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use std::{path::PathBuf, sync::Arc};

pub struct ProjectItemEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    project_item_path: &'lifetime PathBuf,
    display_name: &'lifetime str,
    preview_value: &'lifetime str,
    depth: usize,
    icon: Option<TextureHandle>,
    is_selected: bool,
    is_directory: bool,
    has_children: bool,
    is_expanded: bool,
    project_hierarchy_frame_action: &'lifetime mut ProjectHierarchyFrameAction,
}

impl<'lifetime> ProjectItemEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_item_path: &'lifetime PathBuf,
        display_name: &'lifetime str,
        preview_value: &'lifetime str,
        depth: usize,
        icon: Option<TextureHandle>,
        is_selected: bool,
        is_directory: bool,
        has_children: bool,
        is_expanded: bool,
        project_hierarchy_frame_action: &'lifetime mut ProjectHierarchyFrameAction,
    ) -> Self {
        Self {
            app_context,
            project_item_path,
            display_name,
            preview_value,
            depth,
            icon,
            is_selected,
            is_directory,
            has_children,
            is_expanded,
            project_hierarchy_frame_action,
        }
    }
}

impl<'lifetime> Widget for ProjectItemEntryView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let icon_size = vec2(16.0, 16.0);
        let expand_arrow_size = vec2(10.0, 10.0);
        let row_left_padding = 8.0;
        let tree_level_indent = 18.0;
        let text_left_padding = 4.0;
        let row_height = 28.0;
        let (allocated_size_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_size().x, row_height), Sense::click_and_drag());

        if self.is_selected {
            user_interface
                .painter()
                .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.selected_background);

            user_interface.painter().rect_stroke(
                allocated_size_rectangle,
                CornerRadius::ZERO,
                Stroke::new(1.0, theme.selected_border),
                StrokeKind::Inside,
            );
        }

        StateLayer {
            bounds_min: allocated_size_rectangle.min,
            bounds_max: allocated_size_rectangle.max,
            enabled: true,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        if response.clicked() {
            *self.project_hierarchy_frame_action = ProjectHierarchyFrameAction::SelectProjectItem(self.project_item_path.clone());
        }

        if self.is_directory && self.has_children && response.double_clicked() {
            *self.project_hierarchy_frame_action = ProjectHierarchyFrameAction::ToggleDirectoryExpansion(self.project_item_path.clone());
        }

        let indentation = self.depth as f32 * tree_level_indent;
        let arrow_center = pos2(
            allocated_size_rectangle.min.x + row_left_padding + indentation + expand_arrow_size.x * 0.5,
            allocated_size_rectangle.center().y,
        );

        if self.is_directory && self.has_children {
            let expand_icon = if self.is_expanded {
                &theme.icon_library.icon_handle_navigation_down_arrow_small
            } else {
                &theme.icon_library.icon_handle_navigation_right_arrow_small
            };

            IconDraw::draw_sized(user_interface, arrow_center, expand_arrow_size, expand_icon);
        }

        let icon_pos_x = allocated_size_rectangle.min.x + row_left_padding + indentation + expand_arrow_size.x + text_left_padding;
        let icon_pos_y = allocated_size_rectangle.center().y - icon_size.y * 0.5;
        let icon_rect = Rect::from_min_size(pos2(icon_pos_x, icon_pos_y), icon_size);
        let text_pos = pos2(icon_rect.max.x + text_left_padding, allocated_size_rectangle.center().y - 7.0);
        let preview_pos = pos2(icon_rect.max.x + text_left_padding, allocated_size_rectangle.center().y + 7.0);

        if let Some(icon) = &self.icon {
            IconDraw::draw_sized(user_interface, icon_rect.center(), icon_size, icon);
        }

        user_interface.painter().text(
            text_pos,
            Align2::LEFT_CENTER,
            self.display_name,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            preview_pos,
            Align2::LEFT_CENTER,
            self.preview_value,
            theme.font_library.font_noto_sans.font_small.clone(),
            theme.foreground_preview,
        );

        response
    }
}
