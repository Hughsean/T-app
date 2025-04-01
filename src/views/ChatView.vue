<template>
    <div class="chat">
      <h2>对话系统</h2>
      <div class="chat-container">
        <div class="chat-messages" ref="chatMessages">
          <div v-for="(message, index) in messages" :key="index" :class="['message', message.type]">
            {{ message.content }}
          </div>
        </div>
        <div class="chat-input">
          <el-input
            v-model="inputMessage"
            placeholder="输入消息..."
            @keyup.enter="sendMessage"
          >
            <template #append>
              <el-button @click="sendMessage">发送</el-button>
            </template>
          </el-input>
        </div>
      </div>
    </div>
  </template>
  
  <script setup>
  import { ref, onMounted, nextTick } from 'vue';
  
  const inputMessage = ref('');
  const messages = ref([
    { type: 'received', content: '你好！我是你的数字人伴侣。有什么我可以帮助你的吗？' }
  ]);
  const chatMessages = ref(null);
  
  const sendMessage = () => {
    if (inputMessage.value.trim() === '') return;
    
    messages.value.push({ type: 'sent', content: inputMessage.value });
    inputMessage.value = '';
    
    // 模拟接收消息
    setTimeout(() => {
      messages.value.push({ type: 'received', content: '我收到了你的消息，正在处理中...' });
      scrollToBottom();
    }, 1000);
  };
  
  const scrollToBottom = () => {
    nextTick(() => {
      if (chatMessages.value) {
        chatMessages.value.scrollTop = chatMessages.value.scrollHeight;
      }
    });
  };
  
  onMounted(() => {
    scrollToBottom();
  });
  </script>
  
  <style scoped>
  .chat {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .chat-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--el-border-color);
    border-radius: 4px;
  }
  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }
  .message {
    margin-bottom: 10px;
    padding: 10px;
    border-radius: 4px;
    max-width: 70%;
  }
  .sent {
    background-color: var(--el-color-primary-light-9);
    align-self: flex-end;
    margin-left: auto;
  }
  .received {
    background-color: var(--el-fill-color-lighter);
  }
  .chat-input {
    padding: 20px;
  }
  </style>