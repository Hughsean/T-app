<template>
  <div class="digital-human-interface">
    <!-- 顶部状态栏 -->
    <div class="status-bar">
      <div class="logo">
        <span class="icon">🤖</span>
        <span class="name">AI Companion</span>
      </div>
      <div class="system-status">
        <span class="wifi">📶</span>
        <span class="battery">🔋 92%</span>
      </div>
    </div>

    <div class="main-content">
      <!-- 左侧数字人区域 -->
      <div class="avatar-container">
        <div class="placeholder-text">
          3D Digital Human Area
          <div class="hint">(Dynamic model will be loaded here)</div>
        </div>
      </div>

      <!-- 右侧对话区 -->
      <div class="chat-container">
        <!-- 侧边面板 -->
        <div class="side-panel">
          <h3>Context Summary</h3>
          <div class="placeholder-line"></div>
          <div class="placeholder-line"></div>
          <div class="placeholder-line"></div>
        </div>

        <!-- 主聊天区 -->
        <div class="chat-main">
          <!-- 对话记录 -->
          <div class="chat-history">
            <div class="message ai">
              <div class="bubble">How can I assist you today?</div>
            </div>
            <div class="message user">
              <div class="bubble">Recommend a weekend travel plan</div>
            </div>
            <div class="loading-placeholder">
              <div class="dot-flashing"></div>
            </div>
          </div>

          <!-- 输入区域 -->
          <div class="input-area">
            <div class="voice-input">
              <button class="voice-button">🎤</button>
              <div class="placeholder">Text input (under development)</div>
            </div>
            <div class="multimodal-icons">
              <span class="icon">📎</span>
              <span class="icon">🖼️</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'DigitalHumanInterface'
}
</script>

<style scoped>
.digital-human-interface {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #0F2027 0%, #2C5364 100%);
  color: white;
}

/* 顶部状态栏优化 */
.status-bar {
  flex-shrink: 0;
  display: flex;
  justify-content: space-between;
  padding: 12px 24px;
  border-bottom: 1px solid rgba(255,255,255,0.1);
  height: 60px;
}

/* 主内容区布局优化 */
.main-content {
  flex: 1;
  display: grid;
  grid-template-columns: 30% 70%;
  min-height: 0; /* 修复内容溢出 */
}

/* 数字人容器调整 */
.avatar-container {
  border-right: 2px dashed #4A90E2;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0,0,0,0.2);
  padding: 20px;
}

/* 对话区网格布局 */
.chat-container {
  display: grid;
  grid-template-columns: 1fr 25%;
  height: 100%;
}

.chat-main {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* 对话记录高度优化 */
.chat-history {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  scrollbar-width: thin;
  scrollbar-color: #4A90E2 transparent;
}

/* 输入区域固定高度 */
.input-area {
  flex-shrink: 0;
  padding: 20px;
  background: rgba(0,0,0,0.3);
  display: flex;
  gap: 16px;
  align-items: center;
}

/* 侧边面板布局修正 */
.side-panel {
  padding: 20px;
  background: rgba(0,0,0,0.15);
  border-left: 1px solid rgba(255,255,255,0.1);
  overflow-y: auto;
}

/* 响应式布局 */
@media (max-width: 768px) {
  .main-content {
    grid-template-columns: 1fr;
    grid-template-rows: 40vh auto;
  }
  
  .chat-container {
    grid-template-columns: 1fr;
    grid-template-rows: auto 200px;
  }
  
  .side-panel {
    border-left: none;
    border-top: 1px solid rgba(255,255,255,0.1);
  }
}

.placeholder-line {
  height: 12px;
  background: #555;
  margin: 8px 0;
  border-radius: 4px;
}
</style>