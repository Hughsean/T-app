use super::device::{DeviceConfig, DeviceType, get_device_config};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error, info, level_filters::LevelFilter};

const DEFAULT_CONFIG: &str = r#"

[websocket]
url = "ws://localhost:8080"
frame_size = 960


[opus]
sample_rate = 16000
channels = 1

[logger]
level = "debug"
"#;

lazy_static! {
    static ref CONFIG: Arc<Config> = Arc::new(Config::new());
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpusCfg {
    pub sample_rate: usize,
    pub channels: usize,
}

impl Into<opus::Channels> for OpusCfg {
    fn into(self) -> opus::Channels {
        match self.channels {
            1 => opus::Channels::Mono,
            2 => opus::Channels::Stereo,
            _ => opus::Channels::Stereo,
        }
    }
}
#[derive(Debug, Clone, Deserialize)]

pub struct WsCfg {
    pub url: String,
    pub frame_size: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogCfg {
    pub level: String,
}

impl Into<LevelFilter> for LogCfg {
    fn into(self) -> tracing::level_filters::LevelFilter {
        match self.level.to_lowercase().as_str() {
            "trace" => tracing::Level::TRACE,
            "debug" => tracing::Level::DEBUG,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => {
                info!("日志级别设置错误, 使用默认级别: info");
                tracing::Level::INFO
            }
        }
        .into()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// WebSocket URL
    pub websocket: WsCfg,
    pub opus: OpusCfg,
    pub logger: LogCfg,
    #[serde(skip)]
    pub input_device: DeviceConfig,
    #[serde(skip)]
    pub output_device: DeviceConfig,
}

impl Config {
    fn new() -> Self {
        let config_str = std::fs::read_to_string(".Config.toml").unwrap_or_else(|_| {
            info!(
                "{} 未发现配置文件, 使用默认配置",
                std::env::current_dir().unwrap().display()
            );
            DEFAULT_CONFIG.to_string()
        });

        std::fs::write(".Config.toml", config_str.clone()).unwrap_or_else(|_| {
            error!("配置文件写入失败: {}", config_str);
        });

        let mut config: Config = toml::from_str(&config_str).unwrap_or_else(|_| {
            info!("配置文件解析失败, 使用默认配置: {}", config_str);
            toml::from_str(DEFAULT_CONFIG).unwrap()
        });

        config.input_device = get_device_config(DeviceType::Input)
            .inspect_err(|e| error!("获取输入设备配置失败: {}", e))
            .unwrap()
            .into();
        config.output_device = get_device_config(DeviceType::Output)
            .inspect_err(|e| error!("获取输入设备配置失败: {}", e))
            .unwrap()
            .into();

        debug!("配置: \n{:#?}", config);
        return config;
    }

    pub fn get_instance() -> Arc<Config> {
        CONFIG.clone()
    }
}
