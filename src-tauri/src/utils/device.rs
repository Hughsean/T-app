use cpal::{
    Device,
    traits::{DeviceTrait, HostTrait},
};
use std::default;

#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub sample_rate: u32,
    pub channels: usize,
    pub raw_config: Option<cpal::SupportedStreamConfig>,
}

impl default::Default for DeviceConfig {
    fn default() -> Self {
        DeviceConfig {
            sample_rate: 0,
            channels: 0,
            raw_config: None,
        }
    }
}

impl From<cpal::SupportedStreamConfig> for DeviceConfig {
    fn from(config: cpal::SupportedStreamConfig) -> Self {
        DeviceConfig {
            sample_rate: config.sample_rate().0,
            channels: config.channels() as usize,
            raw_config: Some(config),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceType {
    Input,
    Output,
}

pub fn get_device(t: DeviceType) -> anyhow::Result<Device> {
    let host = cpal::default_host();
    match t {
        DeviceType::Input => {
            let device = host
                .default_input_device()
                .ok_or(anyhow::anyhow!("No input device found"))?;
            Ok(device)
        }
        DeviceType::Output => {
            let device = host
                .default_output_device()
                .ok_or(anyhow::anyhow!("No input device found"))?;
            Ok(device)
        }
    }
}

pub fn get_device_config(t: DeviceType) -> anyhow::Result<cpal::SupportedStreamConfig> {
    let device = get_device(t)?;

    match t {
        DeviceType::Input => {
            let config = device.default_input_config()?;
            Ok(config)
        }
        DeviceType::Output => {
            let config = device.default_output_config()?;
            Ok(config)
        }
    }
}
