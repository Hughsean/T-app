import { websocketURL, wsEvent, audioConfig, protocolHeader } from "@/constants/constants";
import { EventEmitter } from "eventemitter3";

class WebsocketProtocol extends EventEmitter {
    private websocket: WebSocket | null = null;
    private connected: boolean = false;
    private helloReceived: Promise<void>;
    private resolveHelloReceived: (() => void) | undefined;
    private WEBSOCKET_URL: string;
    private CLIENT_ID: string;
    private DEVICE_ID: string;
    private retries = 0;
    private audioBuffer: Uint8Array = new Uint8Array(0);

    constructor() {
        super();
        this.WEBSOCKET_URL = websocketURL;
        this.CLIENT_ID = "test-client-id";
        this.DEVICE_ID = "test-device-id";
        this.helloReceived = new Promise((resolve) => {
            this.resolveHelloReceived = resolve;
        });
    }

    async connect(): Promise<boolean> {
        try {
            this.websocket = new WebSocket(this.WEBSOCKET_URL);

            this.websocket.onopen = async () => {
                this.websocket?.send(JSON.stringify({
                    type: 'auth',
                    token: 'your-auth-token',
                    device_id: this.DEVICE_ID,
                    client_id: this.CLIENT_ID
                }));
                await this.sendHelloMessage();
            };

            this.websocket.onmessage = async (message) => {
                await this.messageHandler(message.data);
            };
            

            this.websocket.onclose = () => {
                this.connected = false;
                this.emit(wsEvent.audioChannelClosed);
            };

            await Promise.race([
                this.helloReceived,
                new Promise((_, reject) => 
                    setTimeout(() => reject(new Error('Connection timeout')), 10000))
            ]);

            this.connected = true;
            return true;
        } catch (error) {
            console.error(`Connection failed: ${error}`);
            if (this.retries++ < 3) {
                await new Promise(r => setTimeout(r, 1000 * this.retries));
                return this.connect();
            }
            this.emit(wsEvent.networkError, error);
            return false;
        }
    }

    async sendText(message: string) {
        if (this.websocket?.readyState === WebSocket.OPEN) {
            try {
                this.websocket.send(message);
            } catch (error) {
                this.emit(wsEvent.networkError, error);
            }
        }
    }
    async sendAudio(buffer: ArrayBuffer) {
        if (this.websocket?.readyState === WebSocket.OPEN) {
          try {
            // 创建协议头
            const header = new ArrayBuffer(11)
            const headerView = new DataView(header)
            
            // 协议头设置
            headerView.setUint8(0, protocolHeader.AUDIO_FRAME)
            const timestamp = BigInt(Date.now())
            headerView.setUint32(1, Number(timestamp >> BigInt(32)), false)
            headerView.setUint32(5, Number(timestamp & BigInt(0xFFFFFFFF)), false)
            headerView.setUint16(9, buffer.byteLength, false)
      
            // 构造数据包
            const packet = new Uint8Array(header.byteLength + buffer.byteLength)
            packet.set(new Uint8Array(header), 0)
            packet.set(new Uint8Array(buffer), 11)
      
            this.websocket.send(packet.buffer)
            
          } catch (error) {
            console.error('音频发送失败:', error)
            this.emit(wsEvent.networkError, error)
          }
        }
      }


    

    private async sendHelloMessage() {
        const helloMessage = {
            type: 'hello',
            audio_params: {
                format: 'opus',
                sample_rate: audioConfig.SAMPLE_RATE,
                channels: audioConfig.CHANNELS,
                frame_duration: audioConfig.FRAME_DURATION,
                frame_size: audioConfig.FRAME_SIZE
            }
        };
        await this.sendText(JSON.stringify(helloMessage));
    }

    private async messageHandler(message: any) {
        console.warn(message);
    
        if (typeof message !== 'string') {
            this.emit(wsEvent.incomingAudio, message);
        } else {
            try {
                const data = JSON.parse(message);
                const msgType = data.type;
    
                if (msgType === 'hello') {
                    await this.handleServerHello(data);
                    
                    // **调试日志，检查后端是否触发了 wsEvent.test**
                    console.log("🔹 服务器触发 wsEvent.test 事件");
                    console.log("🔹 hello 消息内容:", data);
    
                    this.emit(wsEvent.test, data);
                } else {
                    this.emit(wsEvent.incomingJson, data);
                }
            } catch (error) {
                console.error(`无效的JSON消息: ${message}, 错误: ${error}`);
            }
        }
    }
    

    private handleServerHello(data: any) {
        console.log("🚀 收到服务器 hello 消息:", data);
    
        // **打印服务器返回的音频参数**
        console.log("📩 服务器返回的 audio_params:", data.audio_params);
        console.log("📩 前端预期的 audio_params:", {
            format: 'opus',
            sample_rate: audioConfig.SAMPLE_RATE,
             frame_size: audioConfig.FRAME_SIZE
        });
    
        const requiredParams = [
            data.audio_params?.format === 'opus',
            data.audio_params?.sample_rate === audioConfig.SAMPLE_RATE
            // ❌ **去掉 frame_size 的检查**
        ];
        
        if (!requiredParams.every(Boolean)) {
            console.error("❌ 协议参数不匹配:", data.audio_params);
            this.emit(wsEvent.networkError, 'Protocol parameter mismatch');
            this.closeAudioChannel();
            return;
        }
        
        // **如果服务器没有返回 frame_size，前端自己计算**
        if (!data.audio_params?.frame_size) {
            data.audio_params.frame_size = Math.floor(data.audio_params.sample_rate * (data.audio_params.frame_duration / 1000));
            console.warn("⚠️ 服务器未返回 frame_size，前端自动计算:", data.audio_params.frame_size);
        }
        
        console.log("✅ 协议参数匹配，WebSocket 连接成功");
        if (this.resolveHelloReceived) {
            this.resolveHelloReceived();
        }
        this.emit(wsEvent.audioChannelOpened);
        
    }
    

    getConnectionStatus(): string {
        return this.connected ? 'connected' : 'disconnected';
    }

    isAudioChannelOpened(): boolean {
        return this.connected && this.websocket?.readyState === WebSocket.OPEN;
    }

    async openAudioChannel(): Promise<boolean> {
        if (!this.connected) {
            return this.connect();
        }
        return true;
    }

    async closeAudioChannel() {
        if (this.websocket) {
            try {
                this.websocket.close();
                this.connected = false;
                this.emit(wsEvent.audioChannelClosed);
            } catch (error) {
                console.error('Close failed:', error);
            }
        }
    }
}

export default WebsocketProtocol;