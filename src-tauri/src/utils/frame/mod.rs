use listen::ListenFrame;
use serde_json::Value;
use tracing::debug;
use tts::TtsFrame;

pub mod listen;
pub mod tts;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Frame {
    ListenFrame(ListenFrame),
    TtsFrame(TtsFrame),
    Error,
}

impl From<ListenFrame> for Frame {
    fn from(listen_frame: ListenFrame) -> Self {
        Frame::ListenFrame(listen_frame)
    }
}

impl From<Value> for Frame {
    fn from(json: Value) -> Self {
        let frame_type = json["type"].as_str().unwrap_or_default();

        match frame_type {
            "listen" => serde_json::from_value::<ListenFrame>(json)
                .map(|e| Frame::ListenFrame(e))
                .unwrap_or(Frame::Error),
            "tts" => serde_json::from_value::<TtsFrame>(json)
                .map(|e| Frame::TtsFrame(e))
                .unwrap_or(Frame::Error),
            _ => {
                debug!("未知帧类型:\n{:#}", json);
                Frame::Error
            }
        }
    }
}
