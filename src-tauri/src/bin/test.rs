use std::{error::Error, io::stdin};
use tauri_app_lib::{
    audio::{audio::Audio, audio_pipeline::AudioPipeline},
    utils::ws::WebsocketProtocol,
};

fn main() -> Result<(), Box<dyn Error>> {
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

    let _ = AudioPipeline::get_instance();

    tauri::async_runtime::block_on(async {
        Audio::get_instance().write().await.start();

        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(n) => {
                println!("{n} bytes read");
                println!("{input}");
            }
            Err(error) => println!("error: {error}"),
        }

        Audio::get_instance().write().await.stop();
    });
    Ok(())
}
