// TODO

use std::sync::Arc;
use tokio::sync::RwLock as AsyncRwLock;

pub type AppState = Arc<AsyncRwLock<AppState_>>;
pub struct AppState_ {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub token: Option<String>,
}
