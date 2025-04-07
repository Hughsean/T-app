use crate::types::SharedAsyncMutex;
use crate::types::SharedAsyncRwLock;
use crate::utils::config::Config;
use crate::utils::ws::WebsocketProtocol;
use lazy_static::lazy_static;
use rubato::FftFixedIn;
use rubato::Resampler;
use std::i16;
use std::ops::Mul;
use std::ops::Not;
use std::sync::Once;
use std::time::Duration;
use tracing::debug;
use tracing::error;
use tracing::info;

const BUFFER_N: usize = 10;
static INIT: Once = Once::new();
lazy_static! {
    static ref AUDIO_CACHE: SharedAsyncRwLock<AudioCache> =
        SharedAsyncRwLock::new(AudioCache::new());
}

#[allow(non_snake_case)]
pub struct AudioCache {
    /// 输入音频采样率
    inputRate: u32,
    /// 输出音频采样率
    outputRate: u32,

    /// 输入音频数据
    rawInPCMData: SharedAsyncRwLock<Vec<f32>>,
    /// 采样率转换后的音频数据
    resampledInData: SharedAsyncRwLock<Vec<f32>>,
    /// Opus编码后的音频数据
    opusInData: SharedAsyncRwLock<Vec<Vec<u8>>>,

    /// 重采样输出音频数据
    rawOutPCMData: SharedAsyncRwLock<Vec<i16>>,
    /// Opus解码后的音频数据
    decodedOutData: SharedAsyncRwLock<Vec<i16>>,
    /// 服务器接收的Opus编码音频数据
    opusOutData: SharedAsyncRwLock<Vec<Vec<u8>>>,

    opsuEncoder: SharedAsyncMutex<opus::Encoder>,
    opsuDecoder: SharedAsyncMutex<opus::Decoder>,
    /// 发送音频数据的线程
    sendThread: Option<tauri::async_runtime::JoinHandle<()>>,
    /// 发送音频数据的线程停止标志
    stop: SharedAsyncRwLock<bool>,

    isSessionActive: bool,
    // 会话开始时，进行数据接收缓存
    sessionInit: Once,
}

impl AudioCache {
    fn new() -> Self {
        debug!("AudioCache 初始化");
        let mut st = None;
        let stop_flag = SharedAsyncRwLock::new(false);

        let st_ = &mut st;
        let stop_flag_ = stop_flag.clone();

        // 线程安全的单例模式
        INIT.call_once(move || {
            debug!("AudioCache 初始化线程");
            // 持续向服务器发送音频数据
            *st_ = Some(tauri::async_runtime::spawn(async move {
                debug!("AudioCache 数据发送线程初始化");
                while !*stop_flag_.read().await {
                    // debug!("AudioCache 数据发送线程运行中...");
                    AudioCache::get_instance().write().await.send_audio().await;
                    std::thread::sleep(std::time::Duration::from_millis(60));
                }
                debug!("AudioCache 数据发送线程退出");
            }));
        });

        AudioCache {
            inputRate: Config::get_instance().input_device.sample_rate,
            outputRate: Config::get_instance().output_device.sample_rate,
            rawInPCMData: SharedAsyncRwLock::new(Vec::with_capacity(
                Config::get_instance().websocket.frame_size * BUFFER_N,
            )),
            resampledInData: SharedAsyncRwLock::new(Vec::with_capacity(
                Config::get_instance().websocket.frame_size * BUFFER_N,
            )),
            opusInData: SharedAsyncRwLock::new(Vec::with_capacity(
                Config::get_instance().websocket.frame_size * BUFFER_N,
            )),

            rawOutPCMData: SharedAsyncRwLock::new(Vec::with_capacity(
                Config::get_instance().websocket.frame_size * BUFFER_N,
            )),
            decodedOutData: SharedAsyncRwLock::new(Vec::with_capacity(
                Config::get_instance().websocket.frame_size * BUFFER_N,
            )),
            opusOutData: SharedAsyncRwLock::new(Vec::with_capacity(
                Config::get_instance().websocket.frame_size * BUFFER_N,
            )),

            opsuEncoder: SharedAsyncMutex::new(
                opus::Encoder::new(
                    Config::get_instance().opus.sample_rate as u32,
                    opus::Channels::Mono,
                    opus::Application::Audio,
                )
                .unwrap(),
            ),
            opsuDecoder: SharedAsyncMutex::new(
                opus::Decoder::new(
                    Config::get_instance().opus.sample_rate as u32,
                    opus::Channels::Mono,
                )
                .unwrap(),
            ),
            sendThread: st,
            stop: stop_flag,
            isSessionActive: false,
            sessionInit: Once::new(),
        }
    }

    pub fn get_instance() -> SharedAsyncRwLock<AudioCache> {
        AUDIO_CACHE.clone()
    }
}

impl Drop for AudioCache {
    fn drop(&mut self) {
        *self.stop.blocking_write() = true;

        if let Some(st) = self.sendThread.take() {
            tauri::async_runtime::block_on(async {
                st.await
                    .inspect_err(|e| error!("AudioCache 数据发送线程退出失败: {}", e))
                    .unwrap();
            });
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
        self.rawInPCMData.blocking_write().append(&mut data.clone());
    }

    async fn resample_in(&self) {
        let len = self.rawInPCMData.read().await.len();

        if len
            > Config::get_instance().websocket.frame_size
                * Config::get_instance().input_device.channels
        {
            let mut rawdata = self.rawInPCMData.write().await;

            let mut resampler = FftFixedIn::<f32>::new(
                self.inputRate as usize,
                Config::get_instance().opus.sample_rate,
                len / Config::get_instance().input_device.channels,
                10,
                Config::get_instance().input_device.channels,
            )
            .unwrap();

            let chunks = rawdata.chunks_exact(Config::get_instance().input_device.channels);
            let remain = chunks.remainder().to_vec();

            let mut input = vec![Vec::new(); Config::get_instance().input_device.channels];
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
                .await
                .append(&mut single_channel.clone());

            *rawdata = remain;
        }
    }

    async fn encode(&mut self) {
        let len = self.resampledInData.read().await.len();
        if len >= Config::get_instance().websocket.frame_size {
            let mut encoder = self.opsuEncoder.lock().await;
            let mut resampled = self.resampledInData.write().await;

            // let remain = resampled.split_off(n * Config::get_instance().websocket.frame_size);
            let chunks = resampled.chunks_exact(Config::get_instance().websocket.frame_size);
            let remain = chunks.remainder().to_vec();

            for chunk in chunks {
                let mut output = vec![0u8; Config::get_instance().websocket.frame_size * BUFFER_N];
                let input = chunk
                    .into_iter()
                    .map(|e| e.mul(i16::MAX as f32) as i16)
                    .collect::<Vec<_>>();

                let encode_size = encoder.encode(&input, &mut output).unwrap();
                self.opusInData
                    .write()
                    .await
                    .push(output[0..encode_size].to_vec());
            }

            *resampled = remain;
        } else {
            self.resample_in().await;
        }
    }

    async fn send_audio(&mut self) {
        let len = self.opusInData.read().await.len();
        if len > 0 {
            let mut opusdata = self.opusInData.write().await;
            for e in opusdata.iter() {
                let rst = WebsocketProtocol::get_instance()
                    .read()
                    .await
                    .send_audio(e.clone())
                    .await;

                if let Err(e) = rst {
                    info!("发送数据帧失败: {}", e);
                }
            }

            opusdata.clear();
        } else {
            self.encode().await;
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

    pub async fn write_output_data(&self, data: Vec<u8>) {
        self.opusOutData.write().await.push(data);
    }

    async fn resample_out(&self, size: usize) {
        let len = self.decodedOutData.read().await.len();
        if len > Config::get_instance().websocket.frame_size * size {
            let mut decoded = self.decodedOutData.write().await;

            let mut resampler = FftFixedIn::<f32>::new(
                Config::get_instance().opus.sample_rate,
                self.outputRate as usize,
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
            self.rawOutPCMData.write().await.extend(stereo_frame);
            // 将多通道数据转换为单通道数据
        } else {
            self.decode().await;
        }
    }

    async fn decode(&self) {
        let len = self.opusOutData.read().await.len();
        if len > 0 {
            let mut opus_data = self.opusOutData.write().await;

            let decoder = &mut self.opsuDecoder.lock().await;

            let mut output = Vec::with_capacity(len);

            opus_data.iter().for_each(|e| {
                let mut temp = vec![0i16; Config::get_instance().websocket.frame_size * 10];
                let size = decoder.decode(&e, &mut temp, false).unwrap();
                output.push(temp[..size].to_vec());
            });

            opus_data.clear();

            self.decodedOutData.write().await.append(
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
        // 新一轮会话，进行数据接收缓存
        self.sessionInit.call_once(|| {
            // 会话开始先等待一段时间，使数据缓存填充一些数据，防止音频卡顿
            std::thread::sleep(Duration::from_millis(300));
            debug!("会话开始，进行数据接收缓存");
        });

        if self.isSessionActive.not() {
            return None;
        }

        let len = self.rawOutPCMData.blocking_read().len();
        if len > size {
            let mut output = self.rawOutPCMData.blocking_write();
            let remain = output.split_off(size);
            let ret = output[0..size].to_vec();
            *output = remain;
            Some(ret)
        } else {
            tauri::async_runtime::block_on(self.resample_out(3));
            None
        }
    }
}

impl AudioCache {
    // XXX: 待测试
    // 会话控制只对输出音频有效
    // 目前输入音频数据不需要控制

    pub async fn session_stop(&mut self) {
        self.isSessionActive = false;
        // 重置会话状态
        self.rawOutPCMData.write().await.clear();
        self.decodedOutData.write().await.clear();
        self.opusOutData.write().await.clear();

        debug!("会话重置，清空输出缓存数据");
    }
    pub fn session_start(&mut self) {
        self.sessionInit = Once::new();
        self.isSessionActive = true;
        debug!("会话开始");
    }
}
