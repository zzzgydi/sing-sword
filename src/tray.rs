use crate::{config, service};
use anyhow::Result;
use once_cell::sync::OnceCell;
use tauri::{
    AppHandle, CustomMenuItem, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    SystemTraySubmenu,
};

#[derive(Debug, Clone)]
pub struct Tray {}

impl Tray {
    pub fn global() -> &'static Tray {
        static SERVICE: OnceCell<Tray> = OnceCell::new();
        SERVICE.get_or_init(|| Tray {})
    }

    pub fn tray_menu() -> SystemTrayMenu {
        let mut service = SystemTrayMenu::new();
        let core_name = config::Sword::global().core_name();

        if let Ok(core_list) = service::Service::list_core() {
            core_list.iter().for_each(|core| {
                let core_id = format!("service_core_{core}");
                let selected = Some(core) == core_name.as_ref();
                let item = CustomMenuItem::new(core_id, core);
                let item = if selected { item.selected() } else { item };
                service = service.to_owned().add_item(item);
            });

            if core_list.len() > 0 {
                service = service.add_native_item(SystemTrayMenuItem::Separator);
            }
        }

        SystemTrayMenu::new()
            .add_item(CustomMenuItem::new("dashboard", "Dashboard"))
            .add_submenu(SystemTraySubmenu::new(
                "Service",
                service
                    .add_item(CustomMenuItem::new("run_core", "Restart Core"))
                    .add_item(CustomMenuItem::new("run_server", "Restart Server")),
            ))
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(CustomMenuItem::new("quit", "Quit").accelerator("CmdOrControl+Q"))
    }

    pub fn on_event(&self, app_handle: &AppHandle, id: &str) -> Result<()> {
        Ok(match id {
            "dashboard" => {
                let (port, _, secret, web_ui) = config::Sword::global().web_info();

                let url = web_ui.unwrap_or(format!("http://localhost:{port}"));
                let mut link = format!("{url}?server=127.0.0.1&port={port}");
                if let Some(secret) = secret {
                    link = format!("{link}&token={secret}");
                }
                open::that(link)?;
            }
            "run_core" => service::Service::global().run_core()?,
            "run_server" => service::Service::global().run_web_server(app_handle)?,
            "quit" => app_handle.exit(0),
            _ => {
                // 更换核心
                if id.starts_with("service_core_") {
                    let core = format!("{}", &id[13..]);

                    service::Service::global().change_core(core)?;
                    app_handle.tray_handle().set_menu(Tray::tray_menu())?;
                }
            }
        })
    }
}

pub fn on_system_tray_event(app_handle: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            crate::log_err!(Tray::global().on_event(app_handle, id.as_str()))
        }
        _ => {}
    }
}
