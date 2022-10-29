use anyhow::Result;
use std::path::PathBuf;
use tauri::api::path::resource_dir;
use tauri::AppHandle;

pub fn app_dir() -> PathBuf {
    #[cfg(not(feature = "win-portable"))]
    {
        tauri::api::path::home_dir()
            .unwrap()
            .join(".config")
            .join("sing-sword")
    }

    #[cfg(feature = "win-portable")]
    {
        tauri::utils::platform::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }
}

pub fn config_dir() -> PathBuf {
    #[cfg(not(feature = "win-portable"))]
    return app_dir();

    #[cfg(feature = "win-portable")]
    return app_dir().join("config");
}

/// 软件主配置路径
pub fn sword_config_path() -> PathBuf {
    config_dir().join("sword.json")
}

pub fn sing_box_dir() -> PathBuf {
    config_dir().join("sing")
}

/// sing-box配置路径
pub fn sing_box_path() -> PathBuf {
    sing_box_dir().join("config.json")
}

pub fn log_dir() -> PathBuf {
    app_dir().join("logs")
}

pub fn core_dir() -> Result<PathBuf> {
    Ok(tauri::utils::platform::current_exe()?
        .parent()
        .ok_or(anyhow::anyhow!("failed to get current_exe parent"))?
        .join("core"))
}

pub fn resources_dir(app_handle: &AppHandle) -> Result<PathBuf> {
    let pkg = app_handle.package_info();

    Ok(resource_dir(pkg, &tauri::Env::default())
        .ok_or(anyhow::anyhow!("failed to get resources_dir"))?
        .join("resources"))
}

pub fn path_to_str(path: &PathBuf) -> Result<&str> {
    let path_str = path
        .as_os_str()
        .to_str()
        .ok_or(anyhow::anyhow!("failed to get path from {:?}", path))?;
    Ok(path_str)
}
