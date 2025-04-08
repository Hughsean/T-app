#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TtsState {
    Start,
    Stop,
    #[serde(rename = "sentence_start")]
    SentenceStart,
    #[serde(rename = "sentence_end")]
    SentenceEnd,
}

impl ToString for TtsState {
    fn to_string(&self) -> String {
        match self {
            TtsState::Start => "start".to_string(),
            TtsState::Stop => "stop".to_string(),
            TtsState::SentenceStart => "sentence_start".to_string(),
            TtsState::SentenceEnd => "sentence_end".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub struct TtsFrame {
    pub state: TtsState,
    pub text: Option<String>,
}

#[test]
fn f() {
    let json = r#"
    {
    "type": "tts",
    "state": "sentence_start",
    "text": "<文本内容>"
    }
    "#;
    // serde_json::j(TtsState::SentenceStart).unwrap();
    let frame: TtsFrame = serde_json::from_str(json).unwrap();
    println!("{:#?}", frame);
}
