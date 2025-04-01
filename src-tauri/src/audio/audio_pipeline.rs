use crate::constraint::BUFFER_N;
use crate::constraint::FRAME_SIZE;
use crate::constraint::INPUT_CHANNELS;
use crate::constraint::OPUS_SAMPLE_RATE;
use crate::utils::ws::WebsocketProtocol;
use lazy_static::lazy_static;
use rubato::FftFixedIn;
use rubato::Resampler;
use std::ops::Mul;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Once;
use std::sync::RwLock;

lazy_static! {
    static ref AUDIO_PIPELINE: Arc<RwLock<AudioPipeline>> =
        Arc::new(RwLock::new(AudioPipeline::new()));
}
static INIT: Once = Once::new();

#[allow(non_snake_case)]
pub struct AudioPipeline {
    input_rate: Option<u32>,
    rawPCMData: Arc<RwLock<Vec<f32>>>,
    resampledData: Arc<RwLock<Vec<f32>>>,
    opusInData: Arc<RwLock<Vec<Vec<u8>>>>,
    opusOutData: Arc<RwLock<Vec<Vec<u8>>>>,
    opsuEncoder: Arc<Mutex<opus::Encoder>>,
    opsuDecoder: Arc<Mutex<opus::Decoder>>,
    sendThread: Option<std::thread::JoinHandle<()>>,
    stop: Arc<RwLock<bool>>,
}

impl AudioPipeline {
    fn new() -> Self {
        let mut st = None;
        let flag = Arc::new(RwLock::new(false));

        let st_ = &mut st;
        let flag_ = flag.clone();

        // 线程安全的单例模式
        INIT.call_once(move || {
            println!("AudioPipeline sender thread init");
            let st = std::thread::spawn(move || {
                while *flag_.read().unwrap() {
                    AudioPipeline::get_instance().write().unwrap().send_audio();
                    std::thread::sleep(std::time::Duration::from_millis(80));
                }
            });
            *st_ = Some(st);
        });

        AudioPipeline {
            input_rate: None,
            rawPCMData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),
            resampledData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),
            opusInData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),
            opusOutData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),
            opsuEncoder: Arc::new(Mutex::new(
                opus::Encoder::new(
                    OPUS_SAMPLE_RATE as u32,
                    opus::Channels::Mono,
                    opus::Application::Audio,
                )
                .unwrap(),
            )),
            opsuDecoder: Arc::new(Mutex::new(
                opus::Decoder::new(OPUS_SAMPLE_RATE as u32, opus::Channels::Mono).unwrap(),
            )),
            sendThread: st,
            stop: flag,
        }
    }

    pub fn get_instance() -> Arc<RwLock<AudioPipeline>> {
        AUDIO_PIPELINE.clone()
    }
}

impl Drop for AudioPipeline {
    fn drop(&mut self) {
        *self.stop.write().unwrap() = true;

        if let Some(st) = self.sendThread.take() {
            st.join().unwrap();
        }
    }
}

// input
impl AudioPipeline {
    pub fn set_input_rate(&mut self, rate: u32) {
        // self.resampler.s
        // self.resampler.
        self.input_rate = Some(rate);
    }

    pub fn write_input_data(&self, data: Vec<f32>) {
        self.rawPCMData.write().unwrap().append(&mut data.clone());
    }

    pub fn resample(&mut self) {
        let len = self.rawPCMData.read().unwrap().len();

        if len > FRAME_SIZE * INPUT_CHANNELS {
            let mut rawdata = self.rawPCMData.write().unwrap();

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
            for chunk in chunks {
                for (channel, &value) in chunk.iter().enumerate() {
                    input[channel].push(value);
                }
            }
            let resampled = resampler.process(&input, None).expect("重采样失败");

            // 将多通道数据转换为单通道数据
            let single_channel: Vec<f32> = resampled[0].clone();
            self.resampledData
                .write()
                .unwrap()
                .append(&mut single_channel.clone());

            *rawdata = remain;
        }
    }

    pub fn encode(&mut self) {
        let len = self.resampledData.read().unwrap().len();
        if len >= FRAME_SIZE {
            let mut encoder = self.opsuEncoder.lock().unwrap();
            let mut resampled = self.resampledData.write().unwrap();

            // let remain = resampled.split_off(n * FRAME_SIZE);
            let chunks = resampled.chunks_exact(FRAME_SIZE);
            let remain = chunks.remainder().to_vec();

            for chunk in chunks {
                let mut output = vec![0u8; FRAME_SIZE * BUFFER_N];
                let input = chunk
                    .into_iter()
                    .map(|e| e.mul(i16::MAX as f32) as i16)
                    .collect::<Vec<_>>();

                let encode_size = encoder.encode(&input, &mut output).unwrap();
                self.opusInData
                    .write()
                    .unwrap()
                    .push(output[0..encode_size].to_vec());
            }

            *resampled = remain;
            // self.resampler=Some(res)
        } else {
            self.resample();
        }
    }

    pub fn send_audio(&mut self) {
        let len = self.opusInData.read().unwrap().len();
        if len > 0 {
            let mut opusdata = self.opusInData.write().unwrap();
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
        }
    }
}

// output
impl AudioPipeline {
    pub fn write_output_data(&self, data: Vec<u8>) {
        self.opusOutData.write().unwrap().push(data);
    }

    pub fn read(&self) -> Vec<f32> {
        // let opusdata = WebsocketProtocol::get_instance()
        //     .blocking_read()
        //     .receive_audio()
        //     .unwrap();

        // let mut decoder = self.opsuDecoder.lock().unwrap();

        // opusdata.iter().for_each(|e|

        // );
        // decoder.

        // let mut opusData = self.opusOutData.write().unwrap();
        let len = self.opusOutData.read().unwrap().len();
        if len > 0 {
            let mut opusData = self.opusOutData.write().unwrap();

            let mut decoder = &mut self.opsuDecoder.lock().unwrap();

            let mut output = vec![0i16; FRAME_SIZE * len];

            let mut opyputref = &mut output;

            opusData.iter().for_each(|e| {
                // decoder
            });
            // let mut data = opusData.pop().unwrap();

            // let decode_size = decoder.decode(&data, &mut output, false).unwrap();
            // output[0..decode_size].to_vec()
            unimplemented!()
        } else {
            vec![]
        }
    }
}
