#![allow(non_snake_case)]
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::audio::{audio_pipeline::AudioPipeline, process};

pub struct Audio {
    pub audioDeviceisOn: Arc<std::sync::RwLock<bool>>,
    pub audioThread: Option<std::thread::JoinHandle<()>>,
}

impl Audio {
    pub fn new() -> Self {
        Audio {
            audioDeviceisOn: Arc::new(std::sync::RwLock::new(false)),
            audioThread: None,
        }
    }

    pub fn start(&mut self) {
        if *self.audioDeviceisOn.read().unwrap() {
            return;
        }

        *self.audioDeviceisOn.write().unwrap() = true;
        let audioDeviceisOn_ = self.audioDeviceisOn.clone();

        let audio_thread = std::thread::spawn(move || {
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
                    process::input_callback(),
                    |e| {
                        eprintln!("Error: {}", e);
                    },
                    None,
                )
                .unwrap();
            //

            stream.play().unwrap();

            loop {
                if !*audioDeviceisOn_.read().unwrap() {
                    println!("Audio thread stopping...");
                    break;
                }
                AudioPipeline::get_instance().read().unwrap().send_audio();
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        });
        self.audioThread = Some(audio_thread);
    }

    pub fn stop(&mut self) {
        if *self.audioDeviceisOn.read().unwrap() {
            return;
        }

        *self.audioDeviceisOn.write().unwrap() = false;

        if let Some(thread) = self.audioThread.take() {
            thread.join().unwrap();
        }
    }
}
