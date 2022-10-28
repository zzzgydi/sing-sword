use crate::{
    config::{self, ISingBox},
    service,
    utils::{self, dirs, init},
};
use anyhow::Result;
use once_cell::sync::OnceCell;
use std::net::SocketAddr;
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

        if let Ok(core_list) = service::Core::list_core() {
            // if core_list.len() > 0 {
            //     service = service
            //         .to_owned()
            //         .add_item(CustomMenuItem::new("core_label", "Core").disabled());
            // }

            core_list.iter().for_each(|core| {
                let core_id = format!("service_core_{core}");
                let selected = Some(core) == core_name.as_ref();
                let title = format!("{core}");
                let item = CustomMenuItem::new(core_id, title);
                let item = if selected { item.selected() } else { item };
                service = service.to_owned().add_item(item);
            });

            if core_list.len() > 0 {
                service = service.add_native_item(SystemTrayMenuItem::Separator);
            }
        }

        let config = SystemTrayMenu::new()
            .add_item(CustomMenuItem::new("open_sword_config", "Sword Config"))
            .add_item(CustomMenuItem::new("open_sing_config", "SingBox Config"))
            .add_item(CustomMenuItem::new("open_core_dir", "Core Dir"))
            .add_item(CustomMenuItem::new("open_logs_dir", "Logs Dir"));

        let about = SystemTrayMenu::new().add_item(
            CustomMenuItem::new("app_version", format!("Version {}", init::app_version()))
                .disabled(),
        );

        SystemTrayMenu::new()
            .add_item(CustomMenuItem::new("dashboard", "Dashboard"))
            .add_item(CustomMenuItem::new("clash_dashboard", "Clash Dashboard"))
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_submenu(SystemTraySubmenu::new(
                "Service",
                service
                    .add_item(CustomMenuItem::new("run_core", "Restart Core"))
                    .add_item(CustomMenuItem::new("run_server", "Restart Server")),
            ))
            .add_submenu(SystemTraySubmenu::new("Config", config))
            .add_submenu(SystemTraySubmenu::new("About", about))
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
            "clash_dashboard" => {
                let path = dirs::sing_box_path();
                let sing_box = ISingBox::read_file(&path)?;
                let url = "https://yacd.haishan.me/";

                if let Some(exp) = sing_box.experimental {
                    if let Some(clash) = exp.clash_api {
                        let socket: SocketAddr = clash.external_controller.parse()?;
                        let mut link = format!("{url}?host={}&port={}", socket.ip(), socket.port());
                        if let Some(secret) = clash.secret {
                            link = format!("{link}&secret={secret}");
                        }
                        open::that(link)?;
                    }
                }
            }
            "run_core" => service::Core::global().run_core()?,
            "run_server" => service::Web::global().run_web(app_handle)?,
            "open_sword_config" => utils::open_by_code(&&dirs::sword_config_path())?,
            "open_sing_config" => utils::open_by_code(&dirs::sing_box_path())?,
            "open_core_dir" => open::that(dirs::core_dir()?)?,
            "open_logs_dir" => open::that(dirs::log_dir())?,
            "quit" => app_handle.exit(0),
            _ => {
                // 更换核心
                if id.starts_with("service_core_") {
                    let core = format!("{}", &id[13..]);

                    service::Core::global().change_core(core)?;
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
