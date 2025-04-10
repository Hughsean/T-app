use app_lib::types::SharedAsyncRwLock;

fn main() {
    let l = SharedAsyncRwLock::new(true.into());
    tauri::async_runtime::spawn(async move {
        l.blocking_read();
    });
}
