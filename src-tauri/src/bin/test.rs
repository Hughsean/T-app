use app_lib::utils::log::init_logger;
use std::{error::Error, io::stdin};
use tracing::{error, info};

fn main() -> Result<(), Box<dyn Error>> {
    init_logger();

    tauri::async_runtime::block_on(async {
        let state = app_lib::audio::AudioState::new(app_lib::audio::AudioState_::new().await);

        state.write().await.ws_connect().await.unwrap();

        state.read().await.audio_start().await;

        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(n) => {
                info!("{n} bytes read");
            }
            Err(error) => error!("error: {error}"),
        }

        state.write().await.audio_stop().await;
    });

    Ok(())
}
