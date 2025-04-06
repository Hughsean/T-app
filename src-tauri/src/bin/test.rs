use std::{error::Error, io::stdin};
use tauri_app_lib::{
    audio::{audio::Audio, cache::AudioCache},
    utils::{config::Config, log::init_logger, ws::WebsocketProtocol},
};
use tracing::{error, info};

fn main() -> Result<(), Box<dyn Error>> {
    init_logger();

    error!("Test");

    if let Err(e) = tauri::async_runtime::block_on(async {
        WebsocketProtocol::get_instance()
            .write()
            .await
            .connect()
            .await
    }) {
        info!(
            "websocket( {} ) 连接失败: {}",
            Config::get_instance().websocket.url,
            e
        );
        return Ok(());
    }

    let _ = AudioCache::get_instance();

    tauri::async_runtime::block_on(async {
        Audio::get_instance().write().await.start();

        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(n) => {
                info!("{n} bytes read");
            }
            Err(error) => error!("error: {error}"),
        }

        Audio::get_instance().write().await.stop();
    });
    Ok(())
}
