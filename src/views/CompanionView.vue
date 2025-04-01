<template>
  <div class="com">
    <h2>情感陪伴系统</h2>
    <div class="com-container">
      <div class="com-messages" ref="comMessages">
        <div v-for="(message, index) in messages" :key="index" :class="['message', message.type]">
          <span>{{ message.content }}</span>
          <span class="timestamp">{{ message.timestamp }}</span>
        </div>
      </div>
      <div class="com-input">
        <el-input
          v-model="inputMessage"
          placeholder="输入消息..."
          @keyup.enter="sendMessage"
        />
        <div class="button-group">
          <el-button @click="sendMessage" type="primary">发送</el-button>
          <el-button @click="startVoiceRecognition" type="success" :disabled="isRecognizing">语音输入</el-button>
          <el-button v-if="isRecognizing" @click="stopVoiceRecognition" type="warning">停止</el-button>
        </div>
      </div>
    </div>

    <!-- 生活助手模块 -->
    <div class="life-assistant">
      <h3>生活助手</h3>
      <el-input v-model="scheduleInput" placeholder="添加日程..." />
      <el-button @click="addSchedule" type="primary">添加日程</el-button>
      
      <div v-if="schedules.length > 0">
        <h4>我的日程</h4>
        <ul>
          <li v-for="(schedule, index) in schedules" :key="index">
            <span>{{ schedule.text }} ({{ schedule.time }})</span>
            <el-button @click="editSchedule(index)" size="mini">编辑</el-button>
            <el-button @click="removeSchedule(index)" size="mini" type="danger">删除</el-button>
          </li>
        </ul>
      </div>

      <el-input v-model="questionInput" placeholder="问我问题..." />
      <el-button @click="askQuestion" type="primary">提问</el-button>
      
      <div v-if="answer" class="answer">
        <p>答复: {{ answer }}</p>
      </div>
      
      <el-button @click="giveAdvice" type="success">获取生活建议</el-button>
      <div v-if="lifeAdvice" class="advice">
        <p>{{ lifeAdvice }}</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick } from 'vue';

// 消息部分
const inputMessage = ref('');
const messages = ref([
  { type: 'received', content: '你好，我在这里倾听。你有什么想说的吗？', timestamp: new Date().toLocaleTimeString() }
]);

const analyzeEmotion = (message) => {
  const sadWords = ['难过', '悲伤', '痛苦'];
  const happyWords = ['开心', '高兴', '快乐'];
  const angryWords = ['生气', '愤怒'];

  if (sadWords.some(word => message.includes(word))) {
    return 'sad';
  }
  if (happyWords.some(word => message.includes(word))) {
    return 'happy';
  }
  if (angryWords.some(word => message.includes(word))) {
    return 'angry';
  }
  return 'neutral';
};

const addTimestamp = () => {
  return new Date().toLocaleString(); // 返回日期和时间
};

const sendMessage = () => {
  if (inputMessage.value.trim() === '') return;
  messages.value.push({ type: 'sent', content: inputMessage.value, timestamp: addTimestamp() });
  inputMessage.value = '';
  const emotion = analyzeEmotion(messages.value[messages.value.length - 1].content);

  setTimeout(() => {
    let responseMessage = '';
    switch (emotion) {
      case 'sad':
        responseMessage = '我能理解你现在的感受，遇到困难时可以多和我说说。我在这里支持你。';
        break;
      case 'happy':
        responseMessage = '太好了！很高兴听到你这么开心，希望你每一天都充满快乐！';
        break;
      case 'angry':
        responseMessage = '我能理解你的愤怒，有时候生活中确实有让人不开心的事情。深呼吸一下，冷静下来看问题。';
        break;
      default:
        responseMessage = '我在这里听你说，任何情绪都可以告诉我。';
        break;
    }
    messages.value.push({ type: 'received', content: responseMessage, timestamp: addTimestamp() });
    scrollToBottom();
  }, 1000);
};

const scrollToBottom = () => {
  nextTick(() => {
    if (comMessages.value) {
      comMessages.value.scrollTop = comMessages.value.scrollHeight;
    }
  });
};

onMounted(() => {
  scrollToBottom();
});

const comMessages = ref(null); // Use ref to access the messages container

// 语音输入功能
const isRecognizing = ref(false); // 控制语音识别状态
let recognition = null; // 将识别对象存储在外部，方便停止

const startVoiceRecognition = () => {
  if (!('webkitSpeechRecognition' in window)) {
    alert("抱歉，您的浏览器不支持语音识别功能。");
    return;
  }

  recognition = new window.webkitSpeechRecognition();
  recognition.lang = 'zh-CN';
  recognition.continuous = true; // 支持连续语音识别
  recognition.interimResults = true;

  recognition.onstart = () => {
    console.log('语音识别已启动');
    isRecognizing.value = true;
  };

  recognition.onresult = (event) => {
    const transcript = event.results[0][0].transcript;
    inputMessage.value = transcript; // 更新文本框内容
  };

  recognition.onerror = (event) => {
    console.error('语音识别错误:', event.error);
    alert('语音识别发生错误，请再试一次');
  };

  recognition.onend = () => {
    console.log('语音识别已结束');
    isRecognizing.value = false;
  };

  recognition.start();
};

const stopVoiceRecognition = () => {
  if (recognition) {
    recognition.stop();
    isRecognizing.value = false; // 停止识别
    console.log('语音识别已停止');
  }
};

// 生活助手部分
const schedules = ref([]);
const scheduleInput = ref('');
const addSchedule = () => {
  if (scheduleInput.value.trim() !== '') {
    const currentTime = new Date().toLocaleString(); // 获取当前日期和时间
    schedules.value.push({ text: scheduleInput.value, time: currentTime });
    scheduleInput.value = ''; // 清空输入框
  }
};

const editSchedule = (index) => {
  const newSchedule = prompt('编辑日程', schedules.value[index].text);
  if (newSchedule) {
    schedules.value[index].text = newSchedule;
  }
};

const removeSchedule = (index) => {
  schedules.value.splice(index, 1);
};

const questionInput = ref('');
const answer = ref('');
const askQuestion = () => {
  const question = questionInput.value.trim().toLowerCase();
  if (question === '') return;

  if (question.includes('今天是什么日子')) {
    answer.value = `今天是：${new Date().toLocaleDateString()}`;
  } else if (question.includes('你是谁')) {
    answer.value = '我是你的生活助手，随时准备为你提供帮助！';
  } else {
    answer.value = '很抱歉，我无法回答这个问题。';
  }
};

const lifeAdvice = ref('');
const giveAdvice = () => {
  const userEmotion = messages.value[messages.value.length - 1]?.content;
  if (userEmotion) {
    if (userEmotion.includes('难过') || userEmotion.includes('悲伤')) {
      lifeAdvice.value = '多做些自己喜欢的事情，和朋友聊天，放松心情，试着做些运动。';
    } else if (userEmotion.includes('开心')) {
      lifeAdvice.value = '保持积极的心态，享受生活中的每一刻，和他人分享你的快乐！';
    } else {
      lifeAdvice.value = '每天保持规律的生活，饮食均衡，适当运动，和他人保持良好的沟通。';
    }
  } else {
    lifeAdvice.value = '给自己一些休息和放松的时间，保持健康的生活方式。';
  }
};
</script>

<style scoped>
.com {
  height: 100%;
  display: flex;
  flex-direction: column;
  font-family: 'Arial', sans-serif;
  background: linear-gradient(to right, #e0f7fa, #ffffff);
  color: #333;
}

.com-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  border: 1px solid #ccc;
  border-radius: 10px;
  background-color: #fff;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  padding: 20px;
  margin: 20px;
}

.com-messages {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
  max-height: 350px;
}

.message {
  background-color: #f7f7f7;
  padding: 12px 20px;
  border-radius: 8px;
  margin-bottom: 10px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.sent {
  background-color: #e1f5fe;
  align-self: flex-end;
  max-width: 70%;
  border-radius: 12px;
}

.received {
  background-color: #e8f5e9;
  max-width: 70%;
  border-radius: 12px;
}

.timestamp {
  font-size: 12px;
  color: #888;
  margin-left: 10px;
}

.com-input {
  padding: 10px 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.el-input {
  width: 100%;
  max-width: 500px;
  margin-bottom: 15px;
  padding: 12px;
  border-radius: 25px;
  border: 1px solid #ccc;
}

.button-group {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.el-button {
  border-radius: 30px;
  padding: 8px 16px;
  font-size: 14px;
  transition: background-color 0.3s ease;
}

.el-button:hover {
  background-color: #0288d1;
  color: white;
}

.el-button[type="primary"] {
  background-color: #039be5;
  color: white;
}

.el-button[type="success"] {
  background-color: #66bb6a;
  color: white;
}

.el-button[type="warning"] {
  background-color: #ff9800;
  color: white;
}

.life-assistant {
  margin-top: 30px;
  padding: 20px;
  border-top: 1px solid #e0e0e0;
  background-color: #f9f9f9;
  border-radius: 10px;
}

.life-assistant h3 {
  font-size: 20px;
  font-weight: bold;
  color: #0288d1;
}

.answer, .advice {
  margin-top: 15px;
  padding: 12px;
  background-color: #f1f1f1;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

ul {
  margin: 0;
  padding-left: 20px;
}

li {
  font-size: 14px;
  padding: 8px;
  list-style-type: none;
  border-bottom: 1px solid #eee;
}

li:last-child {
  border-bottom: none;
}

@media (max-width: 768px) {
  .com-container {
    margin: 10px;
  }

  .com-messages {
    max-height: 300px;
  }

  .el-input, .el-button {
    width: 100%;
  }
}
</style>
