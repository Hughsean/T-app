use crate::constraint::BUFFER_N;
use crate::constraint::FRAME_SIZE;
use crate::constraint::INPUT_CHANNELS;
use crate::constraint::OPUS_SAMPLE_RATE;
use crate::utils::ws::WebsocketProtocol;
use lazy_static::lazy_static;
use rubato::FftFixedIn;
use rubato::Resampler;
use std::i16;
use std::ops::Div;
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
    output_rate: Option<u32>,

    rawInPCMData: Arc<RwLock<Vec<f32>>>,
    resampledInData: Arc<RwLock<Vec<f32>>>,
    opusInData: Arc<RwLock<Vec<Vec<u8>>>>,

    rawOutPCMData: Arc<RwLock<Vec<i16>>>,
    decodedOutData: Arc<RwLock<Vec<i16>>>,
    opusOutData: Arc<RwLock<Vec<Vec<u8>>>>,

    opsuEncoder: Arc<Mutex<opus::Encoder>>,
    opsuDecoder: Arc<Mutex<opus::Decoder>>,
    sendThread: Option<std::thread::JoinHandle<()>>,
    stop: Arc<RwLock<bool>>,
}

impl AudioPipeline {
    fn new() -> Self {
        let mut st = None;
        let stop_flag = Arc::new(RwLock::new(false));

        let st_ = &mut st;
        let stop_flag_ = stop_flag.clone();

        // 线程安全的单例模式
        INIT.call_once(move || {
            println!("AudioPipeline sender thread init");
            // 持续向服务器发送音频数据
            let st = std::thread::spawn(move || {
                while !*stop_flag_.read().unwrap() {
                    AudioPipeline::get_instance().write().unwrap().send_audio();
                    std::thread::sleep(std::time::Duration::from_millis(80));
                }
            });
            *st_ = Some(st);
        });

        AudioPipeline {
            input_rate: None,
            output_rate: None,
            rawInPCMData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),
            resampledInData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),
            opusInData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),

            rawOutPCMData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),
            decodedOutData: Arc::new(RwLock::new(Vec::with_capacity(FRAME_SIZE * BUFFER_N))),
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
            stop: stop_flag,
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
        self.rawInPCMData.write().unwrap().append(&mut data.clone());
    }

    pub fn resample_in(&self) {
        let len = self.rawInPCMData.read().unwrap().len();

        if len > FRAME_SIZE * INPUT_CHANNELS {
            let mut rawdata = self.rawInPCMData.write().unwrap();

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
            self.resampledInData
                .write()
                .unwrap()
                .append(&mut single_channel.clone());

            *rawdata = remain;
        }
    }

    pub fn encode(&mut self) {
        let len = self.resampledInData.read().unwrap().len();
        if len >= FRAME_SIZE {
            let mut encoder = self.opsuEncoder.lock().unwrap();
            let mut resampled = self.resampledInData.write().unwrap();

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
            self.resample_in();
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
    pub fn set_output_rate(&mut self, rate: u32) {
        // self.resampler.s
        // self.resampler.
        self.output_rate = Some(rate);
    }

    pub fn write_output_data(&self, data: Vec<u8>) {
        self.opusOutData.write().unwrap().push(data);
    }

    pub fn resample_out(&self) {
        let len = self.decodedOutData.read().unwrap().len();
        if len > FRAME_SIZE {
            let mut decoded = self.decodedOutData.write().unwrap();

            let mut resampler = FftFixedIn::<f32>::new(
                OPUS_SAMPLE_RATE,
                self.output_rate.unwrap() as usize,
                len,
                10,
                1,
            )
            .unwrap();

            let input = vec![decoded.iter().map(|e| *e as f32).collect::<Vec<_>>()];

            let resampled = resampler.process(&input, None).expect("重采样失败");

            let mut stereo_frame = Vec::new();
            for sample in resampled[0].iter() {
                stereo_frame.push(*sample as i16);
                stereo_frame.push(*sample as i16);
            }

            decoded.clear();
            self.rawOutPCMData.write().unwrap().extend(stereo_frame);
            // 将多通道数据转换为单通道数据
        } else {
            self.decode();
        }
    }

    pub fn decode(&self) {
        let len = self.opusOutData.read().unwrap().len();
        if len > 0 {
            let mut opus_data = self.opusOutData.write().unwrap();

            let decoder = &mut self.opsuDecoder.lock().unwrap();

            let mut output = Vec::with_capacity(len);

            opus_data.iter().for_each(|e| {
                let mut temp = vec![0i16; FRAME_SIZE * 10];
                let size = decoder.decode(&e, &mut temp, false).unwrap();
                println!("size: {:?}", size);
                output.push(temp[..size].to_vec());
            });

            opus_data.clear();

            self.decodedOutData.write().unwrap().append(
                output
                    .iter()
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>()
                    .as_mut(),
            );
            println!(
                "output data: {:?}",
                self.decodedOutData.read().unwrap().len()
            );
        }
    }

    pub fn decode_(&self) {
        let len = self.opusOutData.read().unwrap().len();
        if len > 0 {
            let mut opus_data = self.opusOutData.write().unwrap();

            let decoder = &mut self.opsuDecoder.lock().unwrap();

            let mut output = Vec::with_capacity(len);

            opus_data.iter().for_each(|e| {
                let mut temp = vec![0i16; FRAME_SIZE * 10];
                let size = decoder.decode(&e, &mut temp, false).unwrap();
                // println!("size: {:?}", size);
                output.push(temp[..size].to_vec());
            });

            opus_data.clear();

            println!(
                "MAX:{:?} MIN{:?}",
                output.iter().flatten().min(),
                output.iter().flatten().max()
            );
            self.rawOutPCMData.write().unwrap().append(
                output
                    .iter()
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>()
                    .as_mut(),
            );
        }
    }

    pub fn read(&self, size: usize) -> Vec<i16> {
        let len = self.rawOutPCMData.read().unwrap().len();
        if len > size {
            let mut output = self.rawOutPCMData.write().unwrap();
            let remain = output.split_off(size);
            let ret = output[0..size].to_vec();
            *output = remain;
            ret
        } else {
            self.resample_out();
            vec![0; size]
        }
    }
}
