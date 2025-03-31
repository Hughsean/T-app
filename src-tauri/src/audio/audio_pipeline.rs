use lazy_static::lazy_static;
use rubato::FftFixedIn;
use rubato::Resampler;
use std::ops::Div;
use std::ops::Mul;
use std::sync::Arc;
use std::sync::RwLock;

use crate::constraint::BUFFER_N;
use crate::constraint::FRAME_SIZE;
use crate::constraint::INPUT_CHANNELS;
use crate::constraint::OPUS_SAMPLE_RATE;
use crate::constraint::OUTPUT_CHANNELS;
use crate::ws::WebsocketProtocol;

lazy_static! {
    static ref AUDIO_PIPELINE: Arc<RwLock<AudioPipeline>> =
        Arc::new(RwLock::new(AudioPipeline::new()));
}

#[allow(non_snake_case)]
pub struct AudioPipeline {
    input_rate: Option<u32>,
    rawPCMData: Arc<RwLock<Vec<f32>>>,
    resampledData: Arc<RwLock<Vec<f32>>>,
    opusData: Arc<RwLock<Vec<Vec<u8>>>>,
}

impl AudioPipeline {
    fn new() -> Self {
        AudioPipeline {
            input_rate: None,
            rawPCMData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),
            resampledData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),
            opusData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),
        }
    }

    pub fn set_input_rate(&mut self, rate: u32) {
        self.input_rate = Some(rate);
    }

    pub fn get_instance() -> Arc<RwLock<AudioPipeline>> {
        AUDIO_PIPELINE.clone()
    }

    pub fn write(&self, data: Vec<f32>) {
        // println!("rawPCMData len: {}", self.rawPCMData.read().unwrap().len());
        self.rawPCMData.write().unwrap().append(&mut data.clone());
    }

    pub fn resample(&self) {
        let len = self.rawPCMData.read().unwrap().len();
        println!("resample len: {}", len);

        if len > FRAME_SIZE * INPUT_CHANNELS {
            let mut rawdata = self.rawPCMData.write().unwrap();

            // println!("{}", self.input_rate.unwrap());
            let mut resampler = FftFixedIn::<f32>::new(
                self.input_rate.unwrap() as usize,
                OPUS_SAMPLE_RATE,
                len / INPUT_CHANNELS,
                10,
                INPUT_CHANNELS,
            )
            .unwrap();

            let chunks = rawdata.chunks_exact(INPUT_CHANNELS);
            let remain = chunks.remainder().to_vec();

            let mut input = vec![Vec::new(); INPUT_CHANNELS];
            // let mut resampled = vec![Vec::new(); OUTPUT_CHANNELS];
            for chunk in chunks {
                // let samples = chunk.chunks(INPUT_CHANNELS);
                // for sample in samples {
                for (channel, &value) in chunk.iter().enumerate() {
                    input[channel].push(value);
                }
                // resampled.append(&mut resampled_);
                // input.clear();
                // }
            }
            let resampled = resampler.process(&input, None).expect("重采样失败");

            // 将多通道数据转换为单通道数据
            let single_channel: Vec<f32> = resampled[0].clone();
            self.resampledData
                .write()
                .unwrap()
                .append(&mut single_channel.clone());

            println!(
                "resampled data len: {}; remain len: {}",
                single_channel.len(),
                remain.len()
            );

            // let chunks = rawdata.chunks_exact(FRAME_SIZE * INPUT_CHANNELS);
            // let remain = chunks.remainder().to_vec();

            // for chunk in chunks {
            //     let mono_data = chunk
            //         .chunks(2)
            //         .map(|ee| ee.iter().sum::<f32>().div(INPUT_CHANNELS as f32))
            //         .collect::<Vec<_>>();

            //     self.resampledData
            //         .write()
            //         .unwrap()
            //         .append(&mut resampler.process(&[mono_data], None).unwrap()[0].clone())
            // }
            *rawdata = remain;
        }
    }

    pub fn encode(&self) {
        let len = self.resampledData.read().unwrap().len();
        if len >= FRAME_SIZE {
            let mut encoder = opus::Encoder::new(
                OPUS_SAMPLE_RATE as u32,
                opus::Channels::Mono,
                opus::Application::Audio,
            )
            .unwrap();

            // let n = len.div(FRAME_SIZE);

            let mut resampled = self.resampledData.write().unwrap();

            // let remain = resampled.split_off(n * FRAME_SIZE);
            let chunks = resampled.chunks_exact(FRAME_SIZE);
            let remain = chunks.remainder().to_vec();

            for chunk in chunks {
                let mut output = vec![0u8; FRAME_SIZE * 2];
                let input = chunk
                    .into_iter()
                    .map(|e| e.mul(i16::MAX as f32) as i16)
                    .collect::<Vec<_>>();

                let encode_size = encoder.encode(&input, &mut output).unwrap();
                println!("encode size: {}", encode_size);
                self.opusData
                    .write()
                    .unwrap()
                    .push(output[0..encode_size].to_vec());
            }

            *resampled = remain;
        } else {
            println!("resampled data");
            self.resample();
        }
    }

    pub fn send_audio(&self) {
        let len = self.opusData.read().unwrap().len();
        if len > 0 {
            let mut opusdata = self.opusData.write().unwrap();
            for e in opusdata.iter() {
                let rst = tauri::async_runtime::block_on(async {
                    WebsocketProtocol::get_instance()
                        .read()
                        .await
                        .send_audio(e.clone())
                        .await
                });

                if let Err(e) = rst {
                    println!("send opus data error: {}", e);
                }
            }

            opusdata.clear();
        } else {
            self.encode();
            println!("opus data is empty");
        }
    }
}
