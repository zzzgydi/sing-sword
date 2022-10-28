use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISingBox {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log: Option<ILog>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<IDns>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<IRoute>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<IExperimental>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbounds: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbounds: Option<Value>,
}

impl Default for ISingBox {
    fn default() -> Self {
        ISingBox {
            log: Some(ILog::default()),
            dns: None,
            route: None,
            experimental: Some(IExperimental::default()),
            inbounds: None,
            outbounds: None,
        }
    }
}

impl ISingBox {
    pub fn read_file(path: &PathBuf) -> Result<ISingBox> {
        let str = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&str)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ILog {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<bool>,
}

impl Default for ILog {
    fn default() -> Self {
        ILog {
            disabled: Some(false),
            level: Some("info".into()),
            output: Some("box.log".into()),
            timestamp: Some(true),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IDns {
    #[serde(rename = "final")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_cache: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_expire: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Value>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IRoute {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geoip: Option<IGeoSiteIP>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geosite: Option<IGeoSiteIP>,
    #[serde(rename = "final")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_detect_interface: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_android_vpn: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_interface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_mark: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Value>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IGeoSiteIP {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_detour: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IExperimental {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clash_api: Option<IClashAPI>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v2ray_api: Option<IV2rayAPI>,
}

impl Default for IExperimental {
    fn default() -> Self {
        IExperimental {
            clash_api: Some(IClashAPI::default()),
            v2ray_api: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IClashAPI {
    #[serde(default = "default_external_controller")]
    pub external_controller: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ui: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direct_io: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_selected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_file: Option<String>,
}

fn default_external_controller() -> String {
    "127.0.0.1:9090".into()
}

impl Default for IClashAPI {
    fn default() -> Self {
        IClashAPI {
            external_controller: default_external_controller(),
            external_ui: None,
            secret: None,
            direct_io: None,
            default_mode: None,
            store_selected: None,
            cache_file: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IV2rayAPI {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<Value>,
}
