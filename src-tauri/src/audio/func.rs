use crate::{
    audio::cache::AudioCache,
    types::{SharedAsyncMutex, SharedAsyncRwLock},
    utils::{config::Config, device::get_device},
};
use cpal::traits::{DeviceTrait, StreamTrait};
use tracing::error;

fn input_callback(
    audio_cacahe: SharedAsyncRwLock<AudioCache>,
) -> impl FnMut(&[f32], &cpal::InputCallbackInfo) {
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        tauri::async_runtime::block_on(async {
            audio_cacahe
                .blocking_read()
                .write_input_data(data.to_vec())
                .await;
        });
    }
}

pub(super) async fn input(
    stopflag: SharedAsyncRwLock<bool>,
    audio_cacahe: SharedAsyncRwLock<AudioCache>,
) {
    let stream = SharedAsyncMutex::new(
        get_device(crate::utils::device::DeviceType::Input)
            .inspect_err(|e| error!("获取输入设备失败: {}", e))
            .unwrap()
            .build_input_stream(
                &Config::get_instance()
                    .input_device
                    .raw_config
                    .clone()
                    .unwrap()
                    .into(),
                input_callback(audio_cacahe),
                |e| {
                    error!("Error: {}", e);
                },
                None,
            )
            .unwrap()
            .into(),
    );

    stream.lock().await.play().unwrap();

    loop {
        if *stopflag.read().await {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

fn output_callback(
    stopflag: SharedAsyncRwLock<bool>,
    audio_cacahe: SharedAsyncRwLock<AudioCache>,
) -> impl FnMut(&mut [i16], &cpal::OutputCallbackInfo) {
    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
        tauri::async_runtime::block_on(async {
            let mut n = 0;
            loop {
                if *stopflag.read().await {
                    break;
                }

                if let Some(recv_data) = audio_cacahe.read().await.read(data.len()).await {
                    data.copy_from_slice(&recv_data);
                    break;
                }

                if n > 12 {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                } else {
                    n += 1;
                };
            }
        })
    }
}

pub(super) async fn output(
    stopflag: SharedAsyncRwLock<bool>,
    audio_cacahe: SharedAsyncRwLock<AudioCache>,
) {
    let stream = SharedAsyncMutex::new(
        get_device(crate::utils::device::DeviceType::Output)
            .inspect_err(|e| error!("获取输出设备失败: {}", e))
            .unwrap()
            .build_output_stream(
                &Config::get_instance()
                    .output_device
                    .raw_config
                    .clone()
                    .unwrap()
                    .into(),
                output_callback(stopflag.clone(), audio_cacahe),
                |e| {
                    error!("Error: {}", e);
                },
                None,
            )
            .unwrap()
            .into(),
    );

    stream.lock().await.play().unwrap();

    loop {
        if *stopflag.read().await {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
