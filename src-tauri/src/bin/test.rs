use std::error::Error;

use tauri_app_lib::{audio::audio::Audio, ws::WebsocketProtocol};

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

    let mut audio = Audio::new();
    audio.start();

    std::thread::sleep(std::time::Duration::from_secs(20));
    audio.stop();
    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(())
}
