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

pub fn input(flag: Arc<std::sync::RwLock<bool>>) {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();
    let config = device.default_input_config().unwrap();

    AudioPipeline::get_instance()
        .write()
        .unwrap()
        .set_input_rate(config.sample_rate().0);

    println!("{:?}", config);

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
        if !*flag.read().unwrap() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

pub fn output_callback() -> impl FnMut(&mut [f32], &cpal::OutputCallbackInfo) {
    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        // println!("output_callback: {:?}", data.len());
        // AudioPipeline::get_instance()
        //     .read()
        //     .unwrap()
        //     .read(data.to_vec());
    }
}
