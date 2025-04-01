<template>
  <div class="home">
    <!-- 顶部欢迎区域 -->
    <div class="hero-section">
      <h1 class="welcome-title">欢迎使用数字人多模态陪伴系统</h1>
      <p class="welcome-subtitle">您的智能伙伴，为您提供全方位的交互体验</p>
      <el-button type="primary" size="large" class="get-started-btn" @click="navigateTo('/numberman')">开始体验</el-button>
    </div>

    <!-- 功能特性区域 -->
    <div class="features-section">
      <h2 class="section-title">核心功能</h2>
      <el-row :gutter="30">
        <el-col :xs="24" :sm="12" :md="8" v-for="(feature, index) in features" :key="index">
          <el-card class="feature-card" shadow="hover">
            <div class="feature-icon">
              <el-icon :size="40" :color="feature.iconColor">
                <component :is="feature.icon" />
              </el-icon>
            </div>
            <h3 class="feature-title">{{ feature.title }}</h3>
            <p class="feature-description">{{ feature.description }}</p>
            <el-button 
              type="primary" 
              :icon="feature.buttonIcon"
              class="feature-button"
              @click="navigateTo(feature.route)"
            >
              {{ feature.buttonText }}
            </el-button>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 统计数据展示 -->
    <div class="stats-section">
      <el-row :gutter="20">
        <el-col :xs="12" :sm="6" v-for="(stat, index) in statistics" :key="index">
          <div class="stat-card">
            <div class="stat-number">{{ stat.number }}</div>
            <div class="stat-label">{{ stat.label }}</div>
          </div>
        </el-col>
      </el-row>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { 
  ChatDotRound, 
  Microphone, 
  VideoCamera,
  ArrowRight
} from '@element-plus/icons-vue'

const router = useRouter()

const features = ref([
  {
    title: '智能对话',
    description: '基于先进的自然语言处理技术，提供流畅的对话体验，理解您的需求并给出准确回应',
    icon: 'ChatDotRound',
    iconColor: '#409EFF',
    buttonText: '开始对话',
    buttonIcon: ArrowRight,
    route: '/chat'
  },
  {
    title: '语音交互',
    description: '支持实时语音识别和合成，配合情感分析，让交流更自然、更有温度',
    icon: 'Microphone',
    iconColor: '#67C23A',
    buttonText: '语音体验',
    buttonIcon: ArrowRight,
    route: '/voice'
  },
  {
    title: '视觉系统',
    description: '生动的数字人形象，配合自然的面部表情和动作，带来沉浸式的交互体验',
    icon: 'VideoCamera',
    iconColor: '#E6A23C',
    buttonText: '查看演示',
    buttonIcon: ArrowRight,
    route: '/visual'
  }
])

const statistics = ref([
  { number: '1000+', label: '活跃用户' },
  { number: '5000+', label: '对话次数' },
  { number: '98%', label: '满意度' },
  { number: '24h', label: '在线服务' }
])

const navigateTo = (route: string) => {
  router.push(route)
}
</script>

<style scoped>
.home {
  padding: 0;
  background-color: #f5f7fa;
  min-height: 100%;
}

.hero-section {
  background: linear-gradient(135deg, #409EFF 0%, #36cfc9 100%);
  padding: 60px 20px;
  text-align: center;
  color: white;
  margin-bottom: 40px;
}

.welcome-title {
  font-size: 2.5em;
  font-weight: bold;
  margin-bottom: 20px;
  background: rgba(255, 255, 255, 0.1);
  padding: 10px;
  border-radius: 8px;
  display: inline-block;
}

.welcome-subtitle {
  font-size: 1.2em;
  margin-bottom: 30px;
  opacity: 0.9;
}

.get-started-btn {
  padding: 12px 30px;
  font-size: 1.1em;
  border-radius: 25px;
  background: white;
  color: #409EFF;
  border: none;
  transition: transform 0.3s;
}

.get-started-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.features-section {
  padding: 40px 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.section-title {
  text-align: center;
  font-size: 2em;
  color: #303133;
  margin-bottom: 40px;
}

.feature-card {
  height: 100%;
  text-align: center;
  padding: 30px;
  transition: all 0.3s;
  border: none;
  background: white;
  border-radius: 12px;
}

.feature-icon {
  margin-bottom: 20px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.feature-title {
  font-size: 1.5em;
  color: #303133;
  margin-bottom: 15px;
}

.feature-description {
  color: #606266;
  line-height: 1.6;
  margin-bottom: 25px;
  min-height: 80px;
}

.feature-button {
  width: 80%;
  border-radius: 25px;
  margin-top: auto;
}

.stats-section {
  background: white;
  padding: 40px 20px;
  margin-top: 40px;
  box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.05);
}

.stat-card {
  text-align: center;
  padding: 20px;
}

.stat-number {
  font-size: 2em;
  font-weight: bold;
  color: #409EFF;
  margin-bottom: 10px;
}

.stat-label {
  color: #606266;
  font-size: 1.1em;
}

@media (max-width: 768px) {
  .welcome-title {
    font-size: 1.8em;
  }

  .feature-description {
    min-height: auto;
  }

  .stats-section {
    margin-top: 20px;
  }
}
</style>