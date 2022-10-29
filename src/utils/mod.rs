use std::path::PathBuf;

pub mod dirs;
pub mod init;

pub const IDENTIFIER: &'static str = "sing-sword.com.github.zzzgydi";

#[macro_export]
macro_rules! log_err {
    ($result: expr) => {
        if let Err(err) = $result {
            log::error!(target: "app", "{err}");
        }
    };
}

#[macro_export]
macro_rules! notify_err {
    ($result: expr) => {
        match $result {
            Ok(o) => Ok(o),
            Err(err) => {
                let _ = tauri::api::notification::Notification::new(crate::utils::IDENTIFIER)
                    .title("Error")
                    .body(format!("{err}"))
                    .show();
                Err(err)
            }
        }
    };
}

#[macro_export]
macro_rules! notify_log_err {
    ($result: expr) => {
        if let Err(err) = $result {
            log::error!(target: "app", "{err}");
            let _ = tauri::api::notification::Notification::new(crate::utils::IDENTIFIER)
                .title("Error")
                .body(format!("{err}"))
                .show();
        }
    };
}

pub fn open_by_code(path: &PathBuf) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    open::with(&path, "Visual Studio Code").or_else(|_| open::that(&path))?;
    #[cfg(not(target_os = "macos"))]
    open::with(&path, "code").or_else(|_| open::that(&path))?;
    Ok(())
}
