#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod service;
mod tray;
mod utils;

use tauri::{Manager, SystemTray};

fn main() {
    let mut app = tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();

            utils::init::init_app(&app_handle);

            let sword = config::Sword::global();
            let _ = sword.init_config();
            let _ = sword.init_sing_box();

            let service = service::Service::global();
            let _ = service.run_core();
            let _ = service.run_web_server(&app_handle);

            let _ = app_handle.tray_handle().set_menu(tray::Tray::tray_menu());
            Ok(())
        })
        .system_tray(SystemTray::new())
        .on_system_tray_event(tray::on_system_tray_event)
        .build(tauri::generate_context!())
        .expect("failed to launch app");

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    app.run(|_, _| {});
}
