use std::path::PathBuf;

pub mod dirs;
pub mod init;

#[macro_export]
macro_rules! log_err {
    ($result: expr) => {
        if let Err(err) = $result {
            log::error!(target: "app", "{err}");
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
