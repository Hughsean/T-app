#![allow(non_snake_case)]
use std::sync::Arc;

use crate::audio::func;

lazy_static::lazy_static! {
    static ref AUDIO: Arc<tokio::sync::RwLock<Audio>> =
        Arc::new(tokio::sync::RwLock::new(Audio::new()));
}

pub struct Audio {
    pub audioDeviceisOn: Arc<std::sync::RwLock<bool>>,
    audioInThread: Option<std::thread::JoinHandle<()>>,
    audioOutThread: Option<std::thread::JoinHandle<()>>,
}

impl Audio {
    fn new() -> Self {
        Audio {
            audioDeviceisOn: Arc::new(std::sync::RwLock::new(false)),
            audioInThread: None,
            audioOutThread: None,
        }
    }

    pub fn get_instance() -> Arc<tokio::sync::RwLock<Audio>> {
        AUDIO.clone()
    }

    pub fn start(&mut self) {
        if *self.audioDeviceisOn.read().unwrap() {
            return;
        }

        let audioDeviceisOn_ = self.audioDeviceisOn.clone();

        let in_thread = std::thread::spawn(move || {
            func::input(audioDeviceisOn_);
        });

        self.audioInThread = Some(in_thread);
        *self.audioDeviceisOn.write().unwrap() = true;
    }

    pub fn stop(&mut self) {
        if !*self.audioDeviceisOn.read().unwrap() {
            return;
        }

        *self.audioDeviceisOn.write().unwrap() = false;

        if let Some(thread) = self.audioInThread.take() {
            thread.join().unwrap();
        }
        println!("Audio thread stopped.");
    }
}
