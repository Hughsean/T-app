use std::sync::Arc;

use crate::audio::audio::Audio;

pub type State = Arc<tokio::sync::RwLock<State_>>;

pub struct State_ {
    pub audio: Audio,
}

pub fn new_state() -> State {
    Arc::new(tokio::sync::RwLock::new(State_ { audio: todo!() }))
}
