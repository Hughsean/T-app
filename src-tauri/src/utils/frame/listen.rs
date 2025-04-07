#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ListenMode {
    Auto,
    Manual,
    RealTime,
}
impl ToString for ListenMode {
    fn to_string(&self) -> String {
        match self {
            ListenMode::Auto => "auto".to_string(),
            ListenMode::Manual => "manual".to_string(),
            ListenMode::RealTime => "realtime".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ListenState {
    Start,
    Stop,
    Detect,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListenFrame {
    pub state: ListenState,
    pub mode: Option<ListenMode>,
    pub text: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[test]
fn f() {
    let json = r#"
    {
        "session_id": "<会话ID>",
        "type": "listen",
        "state": "start",
        "mode": "auto"
    }
    "#;
    let frame: ListenFrame = serde_json::from_str(json).unwrap();
    println!("{:#?}", frame);
}
