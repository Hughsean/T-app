class PCMProcessor extends AudioWorkletProcessor {
    constructor(options) {
      super()
      this.buffer = new Float32Array(0)
      this.frameSize = options.processorOptions.frameSize
    }
  
    process(inputs) {
      const input = inputs[0][0] // 获取第一通道数据
      if (!input) return true
  
      // 合并缓冲区
      const tempBuffer = new Float32Array(this.buffer.length + input.length)
      tempBuffer.set(this.buffer)
      tempBuffer.set(input, this.buffer.length)
      this.buffer = tempBuffer
  
      // 处理完整帧
      while (this.buffer.length >= this.frameSize) {
        const frame = this.buffer.subarray(0, this.frameSize)
        this.buffer = this.buffer.subarray(this.frameSize)
        
        // 转换为Int16并发送
        const int16Frame = this.float32ToInt16(frame)
        this.port.postMessage(int16Frame.buffer, [int16Frame.buffer])
      }
      
      return true
    }
  
    float32ToInt16(buffer) {
      const int16Array = new Int16Array(buffer.length)
      for (let i = 0; i < buffer.length; i++) {
        const val = Math.max(-1, Math.min(1, buffer[i]))
        int16Array[i] = val < 0 ? val * 0x8000 : val * 0x7FFF
      }
      return int16Array
    }
  }
  
  registerProcessor('pcm-processor', PCMProcessor)