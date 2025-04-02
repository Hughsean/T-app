use crate::utils::config::CONFIG;
use crate::utils::ws::WebsocketProtocol;
use lazy_static::lazy_static;
use rubato::FftFixedIn;
use rubato::Resampler;
use std::i16;
use std::ops::Mul;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Once;
use std::sync::RwLock;
use tracing::debug;
use tracing::info;

const BUFFER_N: usize = 10;
lazy_static! {
    static ref AUDIO_CACHE: Arc<RwLock<AudioCache>> = Arc::new(RwLock::new(AudioCache::new()));
}
static INIT: Once = Once::new();

#[allow(non_snake_case)]
pub struct AudioCache {
    /// 输入音频采样率
    input_rate: u32,
    /// 输出音频采样率
    output_rate: u32,

    /// 输入音频数据
    rawInPCMData: Arc<RwLock<Vec<f32>>>,
    /// 采样率转换后的音频数据
    resampledInData: Arc<RwLock<Vec<f32>>>,
    /// Opus编码后的音频数据
    opusInData: Arc<RwLock<Vec<Vec<u8>>>>,

    /// 重采样输出音频数据
    rawOutPCMData: Arc<RwLock<Vec<i16>>>,
    /// Opus解码后的音频数据
    decodedOutData: Arc<RwLock<Vec<i16>>>,
    /// 服务器接收的Opus编码音频数据
    opusOutData: Arc<RwLock<Vec<Vec<u8>>>>,

    opsuEncoder: Arc<Mutex<opus::Encoder>>,
    opsuDecoder: Arc<Mutex<opus::Decoder>>,
    /// 发送音频数据的线程
    sendThread: Option<std::thread::JoinHandle<()>>,
    /// 发送音频数据的线程停止标志
    stop: Arc<RwLock<bool>>,
}

impl AudioCache {
    fn new() -> Self {
        let mut st = None;
        let stop_flag = Arc::new(RwLock::new(false));

        let st_ = &mut st;
        let stop_flag_ = stop_flag.clone();

        // 线程安全的单例模式
        INIT.call_once(move || {
            // 持续向服务器发送音频数据
            let st = std::thread::spawn(move || {
                debug!("AudioPipeline sender thread init");
                while !*stop_flag_.read().unwrap() {
                    AudioCache::get_instance().write().unwrap().send_audio();
                    std::thread::sleep(std::time::Duration::from_millis(80));
                }
                debug!("AudioPipeline sender thread exit");
            });
            *st_ = Some(st);
        });

        AudioCache {
            input_rate: CONFIG.input_device.sample_rate,
            output_rate: CONFIG.output_device.sample_rate,
            rawInPCMData: Arc::new(RwLock::new(Vec::with_capacity(
                CONFIG.websocket.frame_size * BUFFER_N,
            ))),
            resampledInData: Arc::new(RwLock::new(Vec::with_capacity(
                CONFIG.websocket.frame_size * BUFFER_N,
            ))),
            opusInData: Arc::new(RwLock::new(Vec::with_capacity(
                CONFIG.websocket.frame_size * BUFFER_N,
            ))),

            rawOutPCMData: Arc::new(RwLock::new(Vec::with_capacity(
                CONFIG.websocket.frame_size * BUFFER_N,
            ))),
            decodedOutData: Arc::new(RwLock::new(Vec::with_capacity(
                CONFIG.websocket.frame_size * BUFFER_N,
            ))),
            opusOutData: Arc::new(RwLock::new(Vec::with_capacity(
                CONFIG.websocket.frame_size * BUFFER_N,
            ))),

            opsuEncoder: Arc::new(Mutex::new(
                opus::Encoder::new(
                    CONFIG.opus.sample_rate as u32,
                    opus::Channels::Mono,
                    opus::Application::Audio,
                )
                .unwrap(),
            )),
            opsuDecoder: Arc::new(Mutex::new(
                opus::Decoder::new(CONFIG.opus.sample_rate as u32, opus::Channels::Mono).unwrap(),
            )),
            sendThread: st,
            stop: stop_flag,
        }
    }

    pub fn get_instance() -> Arc<RwLock<AudioCache>> {
        AUDIO_CACHE.clone()
    }
}

impl Drop for AudioCache {
    fn drop(&mut self) {
        *self.stop.write().unwrap() = true;

        if let Some(st) = self.sendThread.take() {
            st.join().unwrap();
        }
    }
}

// input
impl AudioCache {
    // pub fn set_input_rate(&mut self, rate: u32) {
    //     // self.resampler.s
    //     // self.resampler.
    //     self.input_rate = Some(rate);
    // }

    pub fn write_input_data(&self, data: Vec<f32>) {
        self.rawInPCMData.write().unwrap().append(&mut data.clone());
    }

    fn resample_in(&self) {
        let len = self.rawInPCMData.read().unwrap().len();

        if len > CONFIG.websocket.frame_size * CONFIG.input_device.channels {
            let mut rawdata = self.rawInPCMData.write().unwrap();

            let mut resampler = FftFixedIn::<f32>::new(
                self.input_rate as usize,
                CONFIG.opus.sample_rate,
                len / CONFIG.input_device.channels,
                10,
                CONFIG.input_device.channels,
            )
            .unwrap();

            let chunks = rawdata.chunks_exact(CONFIG.input_device.channels);
            let remain = chunks.remainder().to_vec();

            let mut input = vec![Vec::new(); CONFIG.input_device.channels];
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

    fn encode(&mut self) {
        let len = self.resampledInData.read().unwrap().len();
        if len >= CONFIG.websocket.frame_size {
            let mut encoder = self.opsuEncoder.lock().unwrap();
            let mut resampled = self.resampledInData.write().unwrap();

            // let remain = resampled.split_off(n * CONFIG.websocket.frame_size);
            let chunks = resampled.chunks_exact(CONFIG.websocket.frame_size);
            let remain = chunks.remainder().to_vec();

            for chunk in chunks {
                let mut output = vec![0u8; CONFIG.websocket.frame_size * BUFFER_N];
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
        } else {
            self.resample_in();
        }
    }

    fn send_audio(&mut self) {
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
                    info!("发送数据帧失败: {}", e);
                }
            }

            opusdata.clear();
        } else {
            self.encode();
        }
    }
}

// output
impl AudioCache {
    // pub fn set_output_rate(&mut self, rate: u32) {
    //     // self.resampler.s
    //     // self.resampler.
    //     self.output_rate = Some(rate);
    // }

    pub fn write_output_data(&self, data: Vec<u8>) {
        self.opusOutData.write().unwrap().push(data);
    }

    fn resample_out(&self, size: usize) {
        let len = self.decodedOutData.read().unwrap().len();
        if len > CONFIG.websocket.frame_size * size {
            let mut decoded = self.decodedOutData.write().unwrap();

            let mut resampler = FftFixedIn::<f32>::new(
                CONFIG.opus.sample_rate,
                self.output_rate as usize,
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

    fn decode(&self) {
        let len = self.opusOutData.read().unwrap().len();
        if len > 0 {
            let mut opus_data = self.opusOutData.write().unwrap();

            let decoder = &mut self.opsuDecoder.lock().unwrap();

            let mut output = Vec::with_capacity(len);

            opus_data.iter().for_each(|e| {
                let mut temp = vec![0i16; CONFIG.websocket.frame_size * 10];
                let size = decoder.decode(&e, &mut temp, false).unwrap();
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
        }
    }

    pub fn read(&self, size: usize) -> Option<Vec<i16>> {
        let len = self.rawOutPCMData.read().unwrap().len();
        if len > size {
            let mut output = self.rawOutPCMData.write().unwrap();
            let remain = output.split_off(size);
            let ret = output[0..size].to_vec();
            *output = remain;
            Some(ret)
        } else {
            self.resample_out(3);
            Some(vec![0; size])
        }
    }
}
