#![allow(non_snake_case)]
use crate::audio::func;
use lazy_static::lazy_static;
use std::sync::Arc;
use tracing::debug;

pub type AudioT = Arc<tokio::sync::RwLock<Audio>>;

lazy_static! {
    static ref AUDIO: AudioT = Arc::new(tokio::sync::RwLock::new(Audio::new()));
}
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
        debug!("音频停止中...");
        if *self.audioStoped.read().unwrap() {
            return;
        }

        *self.audioStoped.write().unwrap() = true;

        debug!("音频停机信号已发送...");

        if let Some(thread) = self.audioInThread.take() {
            thread.join().unwrap();
        }

        debug!("输入线程停止");

        if let Some(thread) = self.audioOutThread.take() {
            thread.join().unwrap();
        }
        debug!("输出线程停止");
    }
}
