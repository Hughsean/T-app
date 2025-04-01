#![allow(non_snake_case)]
use std::sync::Arc;

use crate::audio::func;

lazy_static::lazy_static! {
  pub  static ref AUDIO: AudioT =
    Arc::new(tokio::sync::RwLock::new(Audio::new()));
}

pub type AudioT = Arc<tokio::sync::RwLock<Audio>>;

pub struct Audio {
    pub audioStoped: Arc<std::sync::RwLock<bool>>,
    audioInThread: Option<std::thread::JoinHandle<()>>,
    audioOutThread: Option<std::thread::JoinHandle<()>>,
}

impl Audio {
    fn new() -> Self {
        Audio {
            audioStoped: Arc::new(std::sync::RwLock::new(true)),
            audioInThread: None,
            audioOutThread: None,
        }
    }

    pub fn get_instance() -> AudioT {
        AUDIO.clone()
    }

    pub fn start(&mut self) {
        if !*self.audioStoped.read().unwrap() {
            return;
        }
        *self.audioStoped.write().unwrap() = false;

        let audioStoped_ = self.audioStoped.clone();

        let in_thread = std::thread::spawn(move || {
            func::input(audioStoped_);
        });

        let audioStoped_ = self.audioStoped.clone();
        let out_thread = std::thread::spawn(move || {
            func::output(audioStoped_);
        });

        self.audioInThread = Some(in_thread);
        self.audioOutThread = Some(out_thread);
    }

    pub fn stop(&mut self) {

        println!("Audio thread stopping.");
        if *self.audioStoped.read().unwrap() {
            return;
        }

        *self.audioStoped.write().unwrap() = true;

        if let Some(thread) = self.audioInThread.take() {
            thread.join().unwrap();
        }
        if let Some(thread) = self.audioOutThread.take() {
            thread.join().unwrap();
        }
        println!("Audio thread stopped.");
    }
}
