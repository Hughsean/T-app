use std::{error::Error, io::stdin};
use tauri_app_lib::{
    audio::{audio::Audio, cache::AudioCache},
    init_logger,
    utils::{config::CONFIG, ws::WebsocketProtocol},
};
use tracing::{error, info};

fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    let r = tauri::async_runtime::block_on(async {
        WebsocketProtocol::get_instance()
            .write()
            .await
            .connect()
            .await
    });

    if let Err(e) = r {
        info!("websocket( {} ) 连接失败: {}", CONFIG.websocket.url, e);
        return Ok(());
    }

    let _ = AudioCache::get_instance();

    tauri::async_runtime::block_on(async {
        Audio::get_instance().write().await.start();

        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(n) => {
                info!("{n} bytes read");
                info!("{input}");
            }
            Err(error) => error!("error: {error}"),
        }

        Audio::get_instance().write().await.stop();
    });
    Ok(())
}
