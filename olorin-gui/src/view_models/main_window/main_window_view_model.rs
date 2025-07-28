use crate::MainWindowView;
use crate::WindowViewModelBindings;
use crate::models::audio::audio_player::AudioPlayer;
use crate::view_models::conversions_view_model::conversions_view_model::ConversionsViewModel;
use crate::view_models::docking::dock_root_view_model::DockRootViewModel;
use crate::view_models::element_scanner::element_scan_results_view_model::ElementScanResultsViewModel;
use crate::view_models::element_scanner::element_scanner_view_model::ElementScannerViewModel;
use crate::view_models::output::output_view_model::OutputViewModel;
use crate::view_models::pointer_scanner::pointer_scan_results_view_model::PointerScanResultsViewModel;
use crate::view_models::pointer_scanner::pointer_scanner_view_model::PointerScannerViewModel;
use crate::view_models::process_selector::process_selector_view_model::ProcessSelectorViewModel;
use crate::view_models::project_explorer::project_explorer_view_model::ProjectExplorerViewModel;
use crate::view_models::settings::memory_settings_view_model::MemorySettingsViewModel;
use crate::view_models::settings::scan_settings_view_model::ScanSettingsViewModel;
use crate::view_models::struct_viewer::struct_viewer_view_model::StructViewerViewModel;
use crate::view_models::validation_view_model::validation_view_model::ValidationViewModel;
use olorin_engine_api::dependency_injection::dependency_container::DependencyContainer;
use slint::ComponentHandle;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use std::sync::Arc;

pub struct MainWindowViewModel {}

impl MainWindowViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        let view = MainWindowView::new().unwrap();
        let view_binding = Arc::new(ViewBinding::new(ComponentHandle::as_weak(&view)));

        dependency_container.register::<ViewBinding<MainWindowView>>(view_binding.clone());
        dependency_container.register::<AudioPlayer>(Arc::new(AudioPlayer::new()));

        DockRootViewModel::register(dependency_container);
        ElementScannerViewModel::register(dependency_container);
        ElementScanResultsViewModel::register(dependency_container);
        PointerScannerViewModel::register(dependency_container);
        PointerScanResultsViewModel::register(dependency_container);
        MemorySettingsViewModel::register(dependency_container);
        OutputViewModel::register(dependency_container);
        ProcessSelectorViewModel::register(dependency_container);
        ProjectExplorerViewModel::register(dependency_container);
        ScanSettingsViewModel::register(dependency_container);
        StructViewerViewModel::register(dependency_container);
        ConversionsViewModel::register(dependency_container);
        ValidationViewModel::register(dependency_container);

        let view_model = Arc::new(MainWindowViewModel {});

        create_view_bindings!(view_binding, {
            WindowViewModelBindings => {
                on_minimize() -> [view_binding] -> Self::on_minimize,
                on_maximize() -> [view_binding] -> Self::on_maximize,
                on_close() -> [] -> Self::on_close,
                on_double_clicked() -> [view_binding] -> Self::on_double_clicked,
                on_drag(delta_x: i32, delta_y: i32) -> [view_binding] -> Self::on_drag
            }
        });

        Self::show(view_binding);

        dependency_container.register::<MainWindowViewModel>(view_model);
    }

    pub fn show(view_binding: Arc<ViewBinding<MainWindowView>>) {
        if let Ok(handle) = view_binding.get_view_handle().lock() {
            if let Some(view) = handle.upgrade() {
                if let Err(error) = view.show() {
                    log::error!("Error showing the main window: {}", error);
                }
            }
        }
    }

    pub fn hide(view_binding: Arc<ViewBinding<MainWindowView>>) {
        if let Ok(handle) = view_binding.get_view_handle().lock() {
            if let Some(view) = handle.upgrade() {
                if let Err(error) = view.hide() {
                    log::error!("Error hiding the main window: {}", error);
                }
            }
        }
    }

    fn on_minimize(view_binding: Arc<ViewBinding<MainWindowView>>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_minimized(true);
        });
    }

    fn on_maximize(view_binding: Arc<ViewBinding<MainWindowView>>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_close() {
        if let Err(error) = slint::quit_event_loop() {
            log::error!("Failed to quit event loop: {}", error);
        }
    }

    fn on_double_clicked(view_binding: Arc<ViewBinding<MainWindowView>>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_drag(
        view_binding: Arc<ViewBinding<MainWindowView>>,
        delta_x: i32,
        delta_y: i32,
    ) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            let mut position = window.position();
            position.x += delta_x;
            position.y += delta_y;
            window.set_position(position);
        });
    }
}
