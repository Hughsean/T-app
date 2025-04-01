use std::error::Error;

use tauri_app_lib::{audio::audio::Audio, utils::ws::WebsocketProtocol};

fn main() -> Result<(), Box<dyn Error>> {
    // let connect = WebsocketProtocol::get_instance().connect()?;

    let r = tauri::async_runtime::block_on(async {
        WebsocketProtocol::get_instance()
            .write()
            .await
            .connect()
            .await
    });

    if let Err(e) = r {
        println!("Failed to connect to websocket: {}", e);
        return Ok(());
    }

    tauri::async_runtime::block_on(async {
        Audio::get_instance().write().await.start();
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        Audio::get_instance().write().await.stop();
    });
    Ok(())
}
