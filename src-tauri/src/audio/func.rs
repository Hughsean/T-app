use crate::audio::audio_pipeline::AudioPipeline;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;

fn input_callback() -> impl FnMut(&[f32], &cpal::InputCallbackInfo) {
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        // println!("input_callback: {:?}", data.len());
        AudioPipeline::get_instance()
            .read()
            .unwrap()
            .write_input_data(data.to_vec());
    }
}

pub fn input(stopflag: Arc<std::sync::RwLock<bool>>) {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();
    let config = device.default_input_config().unwrap();

    AudioPipeline::get_instance()
        .write()
        .unwrap()
        .set_input_rate(config.sample_rate().0);

    println!("Input: {:?}", config);

    let stream = device
        .build_input_stream(
            &config.into(),
            input_callback(),
            |e| {
                eprintln!("Error: {}", e);
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
            if let Some(recv_data) = AudioPipeline::get_instance()
                .read()
                .unwrap()
                .read(data.len())
            {
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
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    println!("Output: {:?}", config);

    AudioPipeline::get_instance()
        .write()
        .unwrap()
        .set_output_rate(config.sample_rate().0);

    let stream = device
        .build_output_stream(
            &config.into(),
            // &stream_config,
            output_callback(stopflag.clone()),
            |e| {
                eprintln!("Error: {}", e);
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
