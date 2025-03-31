use crate::audio::audio_pipeline::AudioPipeline;

pub fn input_callback() -> impl FnMut(&[f32], &cpal::InputCallbackInfo) {
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        // println!("input_callback: {:?}", data.len());
        AudioPipeline::get_instance()
            .read()
            .unwrap()
            .write(data.to_vec());
    }
}
