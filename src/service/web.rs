use crate::{config::Sword, utils::dirs};
use anyhow::Result;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::sync::Arc;
use tauri::{async_runtime::JoinHandle, AppHandle};
use warp::Filter;

#[derive(Debug, Clone)]
pub struct Web {
    pub web_handler: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl Web {
    pub fn global() -> &'static Web {
        static WEB: OnceCell<Web> = OnceCell::new();
        WEB.get_or_init(|| Web {
            web_handler: Arc::new(RwLock::new(None)),
        })
    }

    /// 启动/重启服务器
    pub fn run_web(&self, app_handle: &AppHandle) -> Result<()> {
        let mut server_handler = self.web_handler.write();
        server_handler.take().map(|sh| sh.abort());

        let app_handle = app_handle.clone();
        *server_handler = Some(tauri::async_runtime::spawn(async move {
            let (port, allow_lan, _, _) = Sword::global().web_info();

            let server = match allow_lan {
                true => [0, 0, 0, 0],
                false => [127, 0, 0, 1],
            };

            let api = api::get_version()
                .or(api::get_config())
                .or(api::get_sing_box())
                .or(api::put_config())
                .or(api::put_sing_box())
                .with(warp::cors().allow_any_origin());

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

mod api {
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
