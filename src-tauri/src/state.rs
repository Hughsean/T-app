// TODO
use crate::types::SharedAsyncRwLock;

pub type AppState = SharedAsyncRwLock<AppState_>;
pub struct AppState_ {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub token: Option<String>,
}
