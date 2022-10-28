use super::sing_box::ISingBox;
use crate::utils::dirs;
use anyhow::Result;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{fs, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISword {
    pub web_port: u16,
    pub web_allow_lan: bool,
    pub web_secret: Option<String>,
    pub web_ui: Option<String>, // 外部的ui

    pub clash_ui: Option<String>, // clash 的默认外部ui
    pub core_name: Option<String>,
}

impl Default for ISword {
    fn default() -> Self {
        ISword {
            web_port: 33211,
            web_allow_lan: false,
            web_secret: None,
            web_ui: None,
            clash_ui: Some("https://yacd.haishan.me/".into()),
            core_name: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sword {
    pub config: Arc<RwLock<ISword>>,

    pub sing_box: Arc<RwLock<ISingBox>>,
}

impl Sword {
    pub fn global() -> &'static Sword {
        static SWORD: OnceCell<Sword> = OnceCell::new();

        SWORD.get_or_init(|| Sword {
            config: Arc::new(RwLock::new(ISword::default())),
            sing_box: Arc::new(RwLock::new(ISingBox::default())),
        })
    }

    pub fn init_config(&self) -> Result<()> {
        let path = dirs::sword_config_path();

        if !path.exists() {
            fs::create_dir_all(dirs::config_dir())?;

            let config = self.config.read();
            let config_str = serde_json::to_string_pretty(&*config)?;
            fs::write(path, config_str.as_bytes())?;
        } else {
            let mut config = self.config.write();
            *config = serde_json::from_str(fs::read_to_string(&path)?.as_str())?;
        }

        Ok(())
    }

    pub fn init_sing_box(&self) -> Result<()> {
        let path = dirs::sing_box_path();

        if !path.exists() {
            fs::create_dir_all(dirs::sing_box_dir())?;

            let sb = self.sing_box.read();
            let sb_str = serde_json::to_string_pretty(&*sb)?;
            fs::write(path, sb_str.as_bytes())?;
        } else {
            let mut sb = self.sing_box.write();
            *sb = serde_json::from_str(fs::read_to_string(&path)?.as_str())?;
        }

        Ok(())
    }

    pub fn set_config(&self, value: ISword) -> Result<()> {
        let mut config = self.config.write();
        *config = value;
        drop(config);
        self.save_config()
    }

    pub fn set_sing_box(&self, value: ISingBox) -> Result<()> {
        let mut sb = self.sing_box.write();
        *sb = value;
        drop(sb);
        self.save_sing_box()
    }

    /// 保存到文件 sword.json
    pub fn save_config(&self) -> Result<()> {
        let path = dirs::sword_config_path();
        let config = self.config.read();
        let config_str = serde_json::to_string_pretty(&*config)?;
        fs::write(path, config_str.as_bytes())?;
        Ok(())
    }

    /// 保存到文件 sing/config.json
    pub fn save_sing_box(&self) -> Result<()> {
        let path = dirs::sing_box_path();
        let sb = self.sing_box.read();
        let sb_str = serde_json::to_string_pretty(&*sb)?;
        fs::write(path, sb_str.as_bytes())?;
        Ok(())
    }

    pub fn web_info(&self) -> (u16, bool, Option<String>, Option<String>) {
        let config = self.config.read();

        let port = config.web_port.clone();
        let allow_lan = config.web_allow_lan.clone();
        let secret = config.web_secret.clone();
        let ui = config.web_ui.clone();

        (port, allow_lan, secret, ui)
    }

    pub fn core_name(&self) -> Option<String> {
        let config = self.config.read();

        match config.core_name.clone() {
            Some(core) => Some(core),
            None => {
                // 默认拿列表里的第一个
                let core_dir = dirs::core_dir().ok()?;
                let list = fs::read_dir(core_dir).ok()?.next()?.ok()?;
                match list.path().file_stem() {
                    Some(stem) => stem.to_os_string().into_string().ok(),
                    None => None,
                }
            }
        }
    }
}
