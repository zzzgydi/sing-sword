use crate::{config::Sword, utils::dirs};
use anyhow::{bail, Result};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::sync::Arc;
use tauri::{
    api::process::{Command, CommandChild, CommandEvent},
    async_runtime::JoinHandle,
    AppHandle,
};
use warp::Filter;

#[derive(Debug, Clone)]
pub struct Service {
    pub core_handler: Arc<RwLock<Option<CommandChild>>>,

    pub web_handler: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl Service {
    pub fn global() -> &'static Service {
        static SERVICE: OnceCell<Service> = OnceCell::new();

        SERVICE.get_or_init(|| Service {
            core_handler: Arc::new(RwLock::new(None)),
            web_handler: Arc::new(RwLock::new(None)),
        })
    }

    /// 启动核心
    pub fn run_core(&self) -> Result<()> {
        let mut core_handler = self.core_handler.write();

        core_handler.take().map(|ch| {
            let _ = ch.kill();
        });

        let config_dir = dirs::sing_box_dir();
        let config_dir = config_dir
            .as_os_str()
            .to_str()
            .ok_or(anyhow::anyhow!("failed to get sing-box config dir path"))?;

        fn use_core_path(name: &str) -> String {
            #[cfg(target_os = "windows")]
            return format!("core\\{name}");
            #[cfg(not(target_os = "windows"))]
            return format!("core/{name}");
        }

        let core_name = Sword::global()
            .core_name()
            .ok_or(anyhow::anyhow!("failed to get core name"))?;
        let cmd = Command::new_sidecar(use_core_path(&core_name))?;

        #[allow(unused_mut)]
        let (mut rx, cmd_child) = cmd
            .args(["run", "-c", "config.json", "-D", config_dir])
            .spawn()?;

        *core_handler = Some(cmd_child);

        log::info!(target: "app", "run core {core_name}");

        #[cfg(feature = "stdout-log")]
        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Terminated(_) => break,
                    CommandEvent::Error(err) => log::error!("{err}"),
                    CommandEvent::Stdout(line) => log::info!("{line}"),
                    CommandEvent::Stderr(line) => log::info!("{line}"),
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// 获取所有可执行的文件
    pub fn list_core() -> Result<Vec<String>> {
        let core_dir = dirs::core_dir()?;

        let list = std::fs::read_dir(core_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map_or(false, |f| f.is_file()))
            .map(|e| match e.path().file_stem() {
                Some(stem) => stem.to_os_string().into_string().ok(),
                None => None,
            })
            .filter_map(|e| e)
            .collect();

        Ok(list)
    }

    pub fn change_core(&self, name: String) -> Result<()> {
        let core_dir = dirs::core_dir()?;

        #[cfg(windows)]
        let core_path = format!("{name}.exe");
        #[cfg(not(windows))]
        let core_path = name.clone();
        let core_path = core_dir.join(core_path);

        if !core_path.exists() {
            bail!("core executable file not exists");
        }

        let sword = Sword::global();
        let mut config = sword.config.write();
        config.core_name = Some(name);
        drop(config);
        sword.save_config()?;

        self.run_core()?;
        Ok(())
    }

    /// 启动服务器
    pub fn run_web_server(&self, app_handle: &AppHandle) -> Result<()> {
        let mut server_handler = self.web_handler.write();
        server_handler.take().map(|sh| sh.abort());

        let app_handle = app_handle.clone();
        *server_handler = Some(tauri::async_runtime::spawn(async move {
            let (port, allow_lan, _, _) = Sword::global().web_info();

            let server = match allow_lan {
                true => [0, 0, 0, 0],
                false => [127, 0, 0, 1],
            };

            let api = handlers::get_version()
                .or(handlers::get_config())
                .or(handlers::get_sing_box())
                .or(handlers::put_config())
                .or(handlers::put_sing_box());

            // 启动静态服务器
            if let Ok(dist_dir) = dirs::resources_dir(&app_handle) {
                let dist_dir = dist_dir.join("dist");
                if dist_dir.exists() {
                    let index_file = warp::get()
                        .and(warp::path::end())
                        .and(warp::fs::file(dist_dir.join("index.html")));
                    let any_file = warp::get().and(warp::fs::dir(dist_dir));

                    log::info!(target: "app", "launch web server with ui");

                    return warp::serve(index_file.or(any_file).or(api))
                        .bind((server, port))
                        .await;
                } else {
                    log::error!(target: "app", "web dist folder not exists");
                }
            }

            log::info!(target: "app", "launch web server");
            warp::serve(api).bind((server, port)).await;
        }));
        Ok(())
    }
}

mod handlers {
    use crate::{config, utils::init};
    use serde::{Deserialize, Serialize};
    use warp::{filters::BoxedFilter, hyper::StatusCode, Filter, Rejection};

    fn with_auth() -> impl Filter<Extract = (), Error = Rejection> + Copy {
        warp::header::optional("authorization")
            .and_then(|auth: Option<String>| async move {
                let web_info = config::Sword::global().web_info();
                if let Some(token) = web_info.2 {
                    let auth = auth.unwrap_or("".into());
                    let auth = format!("Bearer {auth}");
                    if token != auth {
                        return Err(warp::reject());
                    }
                }
                Ok(())
            })
            .untuple_one()
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct IVersionDTO {
        pub version: String,
    }

    /// GET /api/version
    pub fn get_version() -> BoxedFilter<(impl warp::Reply,)> {
        warp::path!("api" / "version")
            .and(warp::get())
            .map(|| {
                warp::reply::json(&IVersionDTO {
                    version: init::app_version(),
                })
            })
            .boxed()
    }

    /// GET /api/config
    pub fn get_config() -> BoxedFilter<(impl warp::Reply,)> {
        warp::path!("api" / "config")
            .and(with_auth())
            .and(warp::get())
            .map(|| {
                let config = config::Sword::global().config.read();
                warp::reply::json(&*config)
            })
            .boxed()
    }

    /// PUT /api/config
    pub fn put_config() -> BoxedFilter<(impl warp::Reply,)> {
        warp::path!("api" / "config")
            .and(with_auth())
            .and(warp::put())
            .and(warp::body::json())
            .map(
                |value: config::ISword| match config::Sword::global().set_config(value) {
                    Ok(_) => StatusCode::NO_CONTENT,
                    Err(err) => {
                        log::error!(target: "app", "{err}");
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                },
            )
            .boxed()
    }

    /// GET /api/sing_box
    pub fn get_sing_box() -> BoxedFilter<(impl warp::Reply,)> {
        warp::path!("api" / "sing_box")
            .and(with_auth())
            .and(warp::get())
            .map(|| {
                let config = config::Sword::global().sing_box.read();
                warp::reply::json(&*config)
            })
            .boxed()
    }

    /// PUT /api/sing_box
    pub fn put_sing_box() -> BoxedFilter<(impl warp::Reply,)> {
        warp::path!("api" / "sing_box")
            .and(with_auth())
            .and(warp::put())
            .and(warp::body::json())
            .map(
                |value: config::ISingBox| match config::Sword::global().set_sing_box(value) {
                    Ok(_) => StatusCode::NO_CONTENT,
                    Err(err) => {
                        log::error!(target: "app", "{err}");
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                },
            )
            .boxed()
    }
}
