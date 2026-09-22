mod adapters;
mod core;
mod ports;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Listener, Manager, WindowEvent};
use tauri_plugin_deep_link::DeepLinkExt;

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(url) = adapters::dispatcher::extract_url_arg(&args) {
                if let Err(e) = adapters::dispatcher::handle_incoming_url(app, &url, None) {
                    log::warn!("could not dispatch url from second instance: {e}");
                }
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                if let Err(e) = app.deep_link().register_all() {
                    log::warn!("could not register deep link schemes at runtime: {e}");
                }
            }

            app.manage(adapters::build_app_state(app.handle()));

            let handle = app.handle().clone();
            app.listen("deep-link://new-url", move |event| {
                if let Ok(urls) = serde_json::from_str::<Vec<String>>(event.payload()) {
                    for url in urls {
                        if let Err(e) =
                            adapters::dispatcher::handle_incoming_url(&handle, &url, None)
                        {
                            log::warn!("could not dispatch deep link url: {e}");
                        }
                    }
                }
            });

            if let Ok(Some(urls)) = app.deep_link().get_current() {
                for url in urls {
                    if let Err(e) =
                        adapters::dispatcher::handle_incoming_url(app.handle(), url.as_str(), None)
                    {
                        log::warn!("could not dispatch pending deep link url: {e}");
                    }
                }
            }

            // Bandeja: menú contextual Mostrar/Cerrar/Salir
            let show_item =
                MenuItem::with_id(app, "show", "Mostrar LinkRouter", true, None::<&str>)?;
            let close_item =
                MenuItem::with_id(app, "close", "Cerrar", true, None::<&str>)?;
            let quit_item = PredefinedMenuItem::quit(app, Some("Salir"))?;
            let menu = Menu::with_items(app, &[&show_item, &close_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(
                    app.default_window_icon()
                        .cloned()
                        .expect("missing default window icon for tray"),
                )
                .tooltip("LinkRouter — selector de navegador")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "close" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    _ => {}
                })
                .build(app)
                .map_err(|e| format!("could not build tray: {e}"))?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            adapters::commands::evaluate_url_route,
            adapters::commands::evaluate_url,
            adapters::commands::get_rules,
            adapters::commands::create_rule,
            adapters::commands::update_rule,
            adapters::commands::delete_rule,
            adapters::commands::open_in_browser,
            adapters::commands::open_with_system_default,
            adapters::commands::list_browser_profiles,
            adapters::commands::set_default_browser,
            adapters::commands::register_scheme,
            adapters::commands::check_scheme
        ])
        .run(tauri::generate_context!())
        .expect("error while running LinkRouter");
}
