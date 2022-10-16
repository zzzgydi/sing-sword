use super::dirs;
use anyhow::Result;
use chrono::Local;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::fs;

/// initialize this instance's log file
fn init_log() -> Result<()> {
    let log_dir = dirs::log_dir();
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }

    let local_time = Local::now().format("%Y-%m-%d-%H").to_string();
    let log_file = format!("{}.log", local_time);
    let log_file = log_dir.join(log_file);

    let time_format = "{d(%Y-%m-%d %H:%M:%S)} - {m}{n}";

    let tofile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(time_format)))
        .build(log_file)?;

    #[allow(unused_mut)]
    let mut root = Root::builder();
    #[allow(unused_mut)]
    let mut logger = Logger::builder().appender("file");
    #[allow(unused_mut)]
    let mut builder =
        Config::builder().appender(Appender::builder().build("file", Box::new(tofile)));

    #[cfg(feature = "stdout-log")]
    {
        use log4rs::append::console::ConsoleAppender;

        let name = "stdout";
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new(time_format)))
            .build();
        builder = builder.appender(Appender::builder().build(name, Box::new(stdout)));
        root = root.appender(name);
    }

    let config = builder
        .logger(logger.build("app", LevelFilter::Info))
        .build(root.build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    Ok(())
}

/// 初始化 拷贝内核执行文件
fn init_core(app_handle: &tauri::AppHandle) -> Result<()> {
    let core_dir = dirs::core_dir()?;
    if !core_dir.exists() {
        fs::create_dir_all(&core_dir)?;

        let res_dir = dirs::resources_dir(app_handle)?.join("core");
        for entry in fs::read_dir(res_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let _ = fs::copy(entry.path(), core_dir.join(entry.file_name()));
                // let _ = fs::remove_file(entry.path());
            }
        }
    }

    Ok(())
}

static mut APP_VERSION: &str = "0.0.1";

pub fn app_version() -> String {
    unsafe { APP_VERSION.into() }
}

pub fn init_app(app_handle: &tauri::AppHandle) {
    let _ = init_log();
    let _ = init_core(app_handle);

    let pkg = app_handle.package_info();
    unsafe {
        APP_VERSION = Box::leak(Box::new(pkg.version.to_string()));
    };
}
