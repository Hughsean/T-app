
<template>
  <div class="voice-view">
    <div class="header">
      <h1>语音交互历史记录</h1>
      <el-button 
        type="danger" 
        @click="clearHistory"
        :disabled="!history.length"
      >
        清空记录
      </el-button>
    </div>

    <transition-group 
      name="list" 
      tag="div" 
      class="history-list"
    >
      <div 
        v-for="item in history" 
        :key="item.id"
        class="history-item"
      >
        <div class="time">{{ item.time }}</div>
        <div class="content">
          <div class="command">
            <el-icon><Mic /></el-icon>
            {{ item.command }}
          </div>
          <div class="response">
            <el-icon><Comment /></el-icon>
            {{ item.response }}
          </div>
        </div>
      </div>

      <div 
        v-if="!history.length" 
        class="empty"
      >
        <el-empty description="暂无语音交互记录" />
      </div>
    </transition-group>
  </div>
</template>

<script setup>
import { storeToRefs } from 'pinia'
import { useVoiceStore } from '@/stores/voiceStore'
import { Mic, Comment } from '@element-plus/icons-vue'

const voiceStore = useVoiceStore()
const { history } = storeToRefs(voiceStore)

const clearHistory = () => {
  voiceStore.history = []
}
</script>

<style scoped>
.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 8px;
}

.history-list {
  max-width: 800px;
  margin: 0 auto;
}

.history-item {
  padding: 16px;
  margin-bottom: 12px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  transition: all 0.3s ease;
}

.time {
  font-size: 12px;
  color: #909399;
  margin-bottom: 8px;
}

.content div {
  display: flex;
  align-items: center;
  margin: 6px 0;
}

.el-icon {
  margin-right: 8px;
  font-size: 16px;
}

.command .el-icon {
  color: #409EFF;
}

.response .el-icon {
  color: #67C23A;
}

.list-enter-active,
.list-leave-active {
  transition: all 0.5s ease;
}
.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateX(30px);
}

.empty {
  padding: 40px 0;
}
</style>