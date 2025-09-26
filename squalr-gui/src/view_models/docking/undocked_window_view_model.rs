use crate::UndockedWindowView;
use crate::WindowViewModelBindings;
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use std::sync::Arc;

pub struct UndockedWindowViewModel {
    view_binding: Arc<ViewBinding<UndockedWindowView>>,
}

impl UndockedWindowViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        let view = UndockedWindowView::new().unwrap();
        let view_binding = Arc::new(ViewBinding::new(ComponentHandle::as_weak(&view)));

        let view_model = Arc::new(UndockedWindowViewModel {
            view_binding: view_binding.clone(),
        });

        create_view_bindings!(view_binding, {
            WindowViewModelBindings => {
                on_minimize() -> [view_binding] -> Self::on_minimize,
                on_maximize() -> [view_binding] -> Self::on_maximize,
                on_close() -> [] -> Self::on_close,
                on_double_clicked() -> [view_binding] -> Self::on_double_clicked ,
                on_drag(delta_x: i32, delta_y: i32) -> [view_binding] -> Self::on_drag
            }
        });

        dependency_container.register::<UndockedWindowViewModel>(view_model);
    }

    /// Shows the undocked window
    pub fn show(&self) {
        self.view_binding
            .execute_on_ui_thread(move |undocked_window_view, _| {
                if let Err(error) = undocked_window_view.show() {
                    log::error!("Error showing an undocked window: {}", error);
                }
            });
    }

    /// Hides the undocked window
    pub fn hide(&self) {
        self.view_binding
            .execute_on_ui_thread(move |undocked_window_view, _| {
                if let Err(error) = undocked_window_view.hide() {
                    log::error!("Error hiding an undocked window: {}", error);
                }
            });
    }

    fn on_minimize(view_binding: Arc<ViewBinding<UndockedWindowView>>) {
        view_binding.execute_on_ui_thread(move |undocked_window_view, _| {
            let window = undocked_window_view.window();
            window.set_minimized(true);
        });
    }

    fn on_maximize(view_binding: Arc<ViewBinding<UndockedWindowView>>) {
        view_binding.execute_on_ui_thread(move |undocked_window_view, _| {
            let window = undocked_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_close() {
        if let Err(error) = slint::quit_event_loop() {
            log::error!("Failed to quit event loop: {}", error);
        }
    }

    fn on_double_clicked(view_binding: Arc<ViewBinding<UndockedWindowView>>) {
        view_binding.execute_on_ui_thread(move |undocked_window_view, _| {
            let window = undocked_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_drag(
        view_binding: Arc<ViewBinding<UndockedWindowView>>,
        delta_x: i32,
        delta_y: i32,
    ) {
        view_binding.execute_on_ui_thread(move |undocked_window_view, _| {
            let window = undocked_window_view.window();
            let mut position = window.position();
            position.x += delta_x;
            position.y += delta_y;
            window.set_position(position);
        });
    }
}
