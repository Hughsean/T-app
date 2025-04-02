<template>
  <div class="voice-assistant" ref="assistantWrapperRef">
    <!-- 连接状态指示 -->
    <div class="connection-status">
      <div :class="['status-dot', connectionStatus]"></div>

      <!-- TODO -->
      <!-- <span>{{ connectionStatusLabel }}</span> -->
    </div>

    <transition name="slide-fade">
      <div v-if="isListening" class="voice-input-container">
        <div class="input-wrapper">
          <el-input
            v-model="transcriptText"
            type="textarea"
            :rows="3"
            readonly
            placeholder="正在聆听您的指令..."
            class="gradient-input"
          />
          <div class="volume-indicator">
            <div
              v-for="n in 5"
              :key="n"
              class="volume-bar"
              :style="getBarStyle(n)"
            ></div>
          </div>
          <div v-if="isProcessing" class="processing-indicator">
            <el-icon class="is-loading"><Loading /></el-icon>
            <span>正在处理中...</span>
          </div>
          <div v-if="isPlaying" class="playback-indicator">
            <el-icon class="is-loading"><VideoPlay /></el-icon>
            <span>正在播放语音反馈...</span>
          </div>
        </div>
      </div>
    </transition>

    <div class="assistant-button-wrapper">
      <div
        class="assistant-button"
        :class="{ active: isListening }"
        @click="
          () => {
            // TODO: 语音助手交互逻辑
          }
        "
      >
        <el-icon :size="36">
          <svg viewBox="0 0 1024 1024">
            <path
              fill="currentColor"
              d="M512 128a128 128 0 0 0-128 128v256a128 128 0 0 0 256 0V256a128 128 0 0 0-128-128zm0-64a192 192 0 0 1 192 192v256a192 192 0 0 1-384 0V256A192 192 0 0 1 512 64zm-32 832v-64a288 288 0 0 1-288-288v-32a32 32 0 0 1 64 0v32a224 224 0 0 0 224 224h64a224 224 0 0 0 224-224v-32a32 32 0 0 1 64 0v32a288 288 0 0 1-288 288v64h64a32 32 0 0 1 0 64H416a32 32 0 0 1 0-64h64z"
            />
          </svg>
        </el-icon>
      </div>

      <transition name="assistant-scale">
        <div v-if="isListening" class="assistant-status">
          <div class="ripple-effect"></div>
          <div class="ripple-effect delay-1"></div>
          <div class="ripple-effect delay-2"></div>
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { ElMessage, ElIcon } from "element-plus";
import { Loading, VideoPlay } from "@element-plus/icons-vue";
import { useRouter } from "vue-router";
import { useVoiceStore } from "@/stores/voiceStore";
import { wsEvent, audioConfig, protocolHeader } from "@/constants/constants";
// import WebsocketProtocol from "@/lib/websocket";
import { onClickOutside } from "@vueuse/core";

// 状态声明（完全无需修改）
const router = useRouter();
const assistantRef = ref<HTMLElement | null>(null);
const isListening = ref(false);
const isProcessing = ref(false);
const isPlaying = ref(false);
const transcriptText = ref("");
const volumeLevel = ref(0);
const connectionStatus = ref<"connected" | "connecting" | "disconnected">(
  "disconnected"
);
const audioContext = ref<AudioContext | null>(null);
// const wsProtocol = ref<WebsocketProtocol | null>(null);
const assistantWrapperRef = ref<HTMLElement | null>(null);
const mediaStream = ref<MediaStream | null>(null);
const voiceStore = useVoiceStore();

const convertFloat32ToInt16 = (float32Array: Float32Array) => {
  const int16Array = new Int16Array(float32Array.length);
  for (let i = 0; i < float32Array.length; i++) {
    const val = Math.max(-1, Math.min(1, float32Array[i]));
    int16Array[i] = val < 0 ? val * 0x8000 : val * 0x7fff;
  }
  return int16Array;
};

declare global {
  interface Window {
    AudioContext: typeof AudioContext;
    webkitAudioContext: typeof AudioContext;
  }
}

// 修改后的录音初始化逻辑
const initAudioRecorder = async () => {
  // try {
  //   mediaStream.value = await navigator.mediaDevices.getUserMedia({
  //     audio: {
  //       sampleRate: audioConfig.SAMPLE_RATE,
  //       channelCount: audioConfig.CHANNELS,
  //       echoCancellation: false,
  //       noiseSuppression: false,
  //     },
  //   });
  //   // 浏览器兼容处理
  //   const AudioContext = window.AudioContext || window.webkitAudioContext;
  //   const context = new AudioContext({
  //     sampleRate: audioConfig.SAMPLE_RATE,
  //     latencyHint: "interactive",
  //   });
  //   // 加载并注册Worklet
  //   await context.audioWorklet.addModule("/src/audio-processor.js");
  //   // 创建处理节点
  //   const processor = new AudioWorkletNode(context, "pcm-processor", {
  //     processorOptions: {
  //       frameSize: audioConfig.FRAME_SIZE,
  //     },
  //   });
  //   // 连接音频节点
  //   const source = context.createMediaStreamSource(mediaStream.value);
  //   source.connect(processor);
  //   processor.connect(context.destination);
  //   // 接收处理后的数据
  //   processor.port.onmessage = (e) => {
  //     if (wsProtocol.value?.connected && e.data.byteLength > 0) {
  //       wsProtocol.value.sendAudio(e.data);
  //     }
  //   };
  //   // 上下文恢复安全处理
  //   if (context.state === "suspended") {
  //     await context.resume();
  //   }
  // } catch (error) {
  //   ElMessage.error("音频初始化失败: " + (error as Error).message);
  //   connectionStatus.value = "disconnected";
  // }
};

// 网络通信控制（完整实现）
// const setupWebsocketListeners = () => {
//   if (!wsProtocol.value) return;

//   wsProtocol.value.on(wsEvent.audioChannelOpened, () => {
//     connectionStatus.value = "connected";
//     initAudioRecorder();
//   });

//   wsProtocol.value.on(wsEvent.audioChannelClosed, () => {
//     connectionStatus.value = "disconnected";
//     stopRecording();
//   });

//   wsProtocol.value.on(wsEvent.incomingJson, (data: any) => {
//     transcriptText.value = data.text;
//     isProcessing.value = false;
//   });

//   wsProtocol.value.on(wsEvent.networkError, (error: any) => {
//     ElMessage.error(`网络错误: ${error}`);
//     connectionStatus.value = "disconnected";
//   });
// };

// 用户交互控制（完整方法）
// const toggleVoice = () => {
//   if (!isListening.value) {
//     wsProtocol.value?.connect();
//     isListening.value = true;
//     isProcessing.value = true;
//   } else {
//     stopRecording();
//   }
// };

const stopRecording = () => {
  mediaStream.value?.getTracks().forEach((track) => track.stop());
  isListening.value = false;
  isProcessing.value = false;
  transcriptText.value = "";
};

// 可视化效果（完整实现）
const getBarStyle = (n: number) => ({
  height: `${Math.min(100, volumeLevel.value * n * 20)}%`,
  transition: "height 0.2s ease",
});

// 生命周期管理
onMounted(() => {
  // wsProtocol.value = new WebsocketProtocol();
  // setupWebsocketListeners();
  // onClickOutside(assistantWrapperRef, () => {
  //   if (isListening.value) stopRecording();
  // });
});

onUnmounted(() => {
  stopRecording();
  // wsProtocol.value?.disconnect();
});
</script>
<style scoped>
.voice-assistant {
  position: fixed;
  bottom: 30px;
  right: 30px;
  z-index: 9999;
}

.connection-status {
  position: absolute;
  top: -28px;
  right: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #666;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}
.status-dot.connected {
  background: #67c23a;
}
.status-dot.connecting {
  background: #e6a23c;
}
.status-dot.disconnected {
  background: #f56c6c;
}

.voice-input-container {
  position: absolute;
  bottom: 80px;
  right: 0;
  width: 320px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 16px;
}

.gradient-input {
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  border-radius: 8px;
}

.volume-indicator {
  display: flex;
  justify-content: center;
  gap: 4px;
  height: 40px;
  margin: 12px 0;
}

.volume-bar {
  width: 6px;
  background: #409eff;
  transition: all 0.2s ease;
}

.processing-indicator,
.playback-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #409eff;
  font-size: 14px;
  margin-top: 12px;
}

.assistant-button-wrapper {
  position: relative;
}

.assistant-button {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: white;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.3s ease;
}
.assistant-button.active {
  background: #409eff;
  box-shadow: 0 4px 16px rgba(64, 158, 255, 0.3);
}
.assistant-button.active svg {
  color: white;
}

.assistant-status {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
}

.ripple-effect {
  position: absolute;
  width: 100%;
  height: 100%;
  border-radius: 50%;
  border: 2px solid rgba(64, 158, 255, 0.3);
  animation: ripple 1.5s infinite;
}
.delay-1 {
  animation-delay: 0.5s;
}
.delay-2 {
  animation-delay: 1s;
}

@keyframes ripple {
  0% {
    transform: scale(1);
    opacity: 1;
  }
  100% {
    transform: scale(1.8);
    opacity: 0;
  }
}

.slide-fade-enter-active {
  transition: all 0.3s ease-out;
}
.slide-fade-leave-active {
  transition: all 0.3s cubic-bezier(1, 0.5, 0.8, 1);
}
.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateY(20px);
  opacity: 0;
}
</style>
