use app_lib::types::SharedAsyncRwLock;

fn main() {
    let l = SharedAsyncRwLock::new(true.into());
    tauri::async_runtime::spawn(async move {
        let _ = l.blocking_read();
    });
}
