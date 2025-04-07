use app_lib::{
    audio::audio::Audio,
    utils::{config::Config, log::init_logger, ws::WebsocketProtocol},
};
use std::{error::Error, io::stdin};
use tracing::{error, info};

fn main() -> Result<(), Box<dyn Error>> {
    init_logger();

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

    tauri::async_runtime::block_on(async {
        Audio::get_instance().write().await.start().await;

        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(n) => {
                info!("{n} bytes read");
            }
            Err(error) => error!("error: {error}"),
        }

        Audio::get_instance().write().await.stop().await;
    });

    Ok(())
}
