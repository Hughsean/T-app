#![allow(non_snake_case)]
use std::sync::Arc;

use crate::audio::func;

lazy_static::lazy_static! {
  pub  static ref AUDIO: AudioT =
    Arc::new(tokio::sync::RwLock::new(Audio::new()));
}

pub type AudioT = Arc<tokio::sync::RwLock<Audio>>;

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

    pub fn get_instance() -> AudioT {
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

        let audioDeviceisOn_ = self.audioDeviceisOn.clone();
        let out_thread = std::thread::spawn(move || {
            func::output(audioDeviceisOn_);
        });

        self.audioInThread = Some(in_thread);
        self.audioOutThread = Some(out_thread);

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
        if let Some(thread) = self.audioOutThread.take() {
            thread.join().unwrap();
        }
        println!("Audio thread stopped.");
    }
}
