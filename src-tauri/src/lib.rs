mod adapters;
mod core;
mod ports;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Listener, Manager, Wry, WindowEvent};
use tauri_plugin_deep_link::DeepLinkExt;

pub(crate) fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub(crate) fn tray_labels(locale: &str) -> (&'static str, &'static str, &'static str) {
    match locale {
        "es" => (
            "Mostrar LinkRouter",
            "Salir",
            "LinkRouter — selector de navegador",
        ),
        _ => (
            "Show LinkRouter",
            "Quit",
            "LinkRouter — browser selector",
        ),
    }
}

pub(crate) fn build_tray_menu(app: &AppHandle, locale: &str) -> tauri::Result<Menu<Wry>> {
    let (show, quit, _) = tray_labels(locale);
    let show_item = MenuItem::with_id(app, "show", show, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", quit, true, None::<&str>)?;
    Menu::with_items(app, &[&show_item, &quit_item])
}

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(url) = adapters::dispatcher::extract_url_arg(&args) {
                if let Err(e) = adapters::dispatcher::handle_incoming_url(app, &url, None) {
                    log::warn!("could not dispatch url from second instance: {e}");
                    show_main_window(app);
                }
            } else {
                show_main_window(app);
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

            // Fuerza el icono de la ventana en Linux; en dev GNOME/la barra de
            // tareas puede quedarse con el fallback por defecto de Tauri.
            if let Some(icon) = app.default_window_icon() {
                if let Some(window) = app.get_webview_window("main") {
                    window
                        .set_icon(icon.clone())
                        .map_err(|e| format!("could not set window icon: {e}"))?;
                }
            }

            // La app vive en el tray: la ventana no debe aparecer en la barra
            // de tareas (GNOME a veces ignora la config en Wayland).
            let skip_taskbar = app
                .get_webview_window("main")
                .map(|window| window.set_skip_taskbar(true))
                .transpose()
                .map_err(|e| format!("could not skip taskbar: {e}"))?;
            log::debug!("skip taskbar applied: {skip_taskbar:?}");

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

            // Bandeja: menú contextual Mostrar/Cerrar/Salir en el idioma efectivo
            let state = app.state::<adapters::AppState>();
            let locale = adapters::effective_locale(
                &state
                    .settings
                    .lock()
                    .expect("settings lock poisoned at startup"),
            );
            let menu = build_tray_menu(app.handle(), locale)?;
            let tooltip = tray_labels(locale).2;

            let tray = TrayIconBuilder::new()
                .icon(
                    app.default_window_icon()
                        .cloned()
                        .expect("missing default window icon for tray"),
                )
                .tooltip(tooltip)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)
                .map_err(|e| format!("could not build tray: {e}"))?;

            *state.tray.lock().expect("tray lock poisoned") = Some(tray);

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
            adapters::commands::check_scheme,
            adapters::commands::get_settings,
            adapters::commands::save_settings,
            adapters::commands::get_config,
            adapters::commands::save_config,
            adapters::commands::system_locale
        ])
        .run(tauri::generate_context!())
        .expect("error while running LinkRouter");
}
