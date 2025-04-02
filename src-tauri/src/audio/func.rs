use crate::{
    audio::cache::AudioCache,
    utils::{config::CONFIG, device::get_device},
};
use cpal::traits::{DeviceTrait, StreamTrait};
use std::sync::Arc;
use tracing::error;

fn input_callback() -> impl FnMut(&[f32], &cpal::InputCallbackInfo) {
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        // println!("input_callback: {:?}", data.len());
        AudioCache::get_instance()
            .read()
            .unwrap()
            .write_input_data(data.to_vec());
    }
}

pub fn input(stopflag: Arc<std::sync::RwLock<bool>>) {
    // let host = cpal::default_host();
    // let device = host.default_input_device().unwrap();
    // let config = device.default_input_config().unwrap();

    // AudioCache::get_instance()
    //     .write()
    //     .unwrap()
    //     .set_input_rate(config.sample_rate().0);

    // debug!("Input: {:?}", config);

    let stream = get_device(crate::utils::device::DeviceType::Input)
        .inspect_err(|e| error!("获取输入设备失败: {}", e))
        .unwrap()
        .build_input_stream(
            &CONFIG.input_device.raw_config.clone().unwrap().into(),
            input_callback(),
            |e| {
                error!("Error: {}", e);
            },
            None,
        )
        .unwrap();
    //

    stream.play().unwrap();

    loop {
        if *stopflag.read().unwrap() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

fn output_callback(
    stopflag: Arc<std::sync::RwLock<bool>>,
) -> impl FnMut(&mut [i16], &cpal::OutputCallbackInfo) {
    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
        let mut n = 0;
        loop {
            if *stopflag.read().unwrap() {
                break;
            }
            if let Some(recv_data) = AudioCache::get_instance().read().unwrap().read(data.len()) {
                data.copy_from_slice(&recv_data);
                break;
            }

            if n > 12 {
                std::thread::sleep(std::time::Duration::from_millis(20));
            } else {
                n += 1;
            };
        }
    }
}

pub fn output(stopflag: Arc<std::sync::RwLock<bool>>) {
    let stream = get_device(crate::utils::device::DeviceType::Output)
        .inspect_err(|e| error!("获取输出设备失败: {}", e))
        .unwrap()
        .build_output_stream(
            &CONFIG.output_device.raw_config.clone().unwrap().into(),
            output_callback(stopflag.clone()),
            |e| {
                error!("Error: {}", e);
            },
            None,
        )
        .unwrap();

    stream.play().unwrap();

    loop {
        if *stopflag.read().unwrap() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
