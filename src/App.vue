<template>
  <router-view v-if="isAuthPage"></router-view>

  <el-container v-else class="app-container">
    <el-aside :width="isCollapse ? '64px' : '240px'" class="sidebar">
      <div class="logo" :class="{ 'logo-collapsed': isCollapse }">
        <img src="@/assets/logo.svg" alt="Logo" class="logo-image" />
        <span v-if="!isCollapse" class="logo-text">数字人伴侣</span>
      </div>
      <el-menu
        :default-active="activeMenu"
        class="el-menu-vertical"
        :collapse="isCollapse"
        :router="true"
      >
        <el-menu-item index="/home">
          <el-icon><House /></el-icon>
          <template #title>首页</template>
        </el-menu-item>
        <el-menu-item index="/chat">
          <el-icon><ChatDotRound /></el-icon>
          <template #title>对话系统</template>
        </el-menu-item>
        <el-menu-item index="/companion">
          <el-icon><ChatDotRound /></el-icon>
          <template #title>情感陪伴</template>
        </el-menu-item>
        <el-menu-item index="/voice">
          <el-icon><Microphone /></el-icon>
          <template #title>语音交互</template>
        </el-menu-item>
        <el-menu-item index="/visual">
          <el-icon><VideoCamera /></el-icon>
          <template #title>视觉系统</template>
        </el-menu-item>
        <el-menu-item index="/profile">
          <el-icon><User /></el-icon>
          <template #title>个人中心</template>
        </el-menu-item>
      </el-menu>
    </el-aside>

    <el-container class="main-container">
      <el-header class="header">
        <el-button
          class="toggle-sidebar"
          @click="toggleSidebar"
          :icon="isCollapse ? Expand : Fold"
        />
        <div class="header-content">
          <el-input
            v-model="searchQuery"
            placeholder="搜索..."
            :prefix-icon="Search"
            class="search-input"
          />
          <el-dropdown>
            <el-avatar :size="40" src="https://example.com/avatar.jpg" />
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item>个人设置</el-dropdown-item>
                <el-dropdown-item>退出登录</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </el-header>

      <el-main>
        <VoiceAssistant />
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup>
import { ref, computed } from "vue";
import { useRoute } from "vue-router";
import {
  House,
  ChatDotRound,
  VideoCamera,
  User,
  Search,
  Expand,
  Fold,
  Microphone,
} from "@element-plus/icons-vue";
import VoiceAssistant from "@/components/VoiceAssistant.vue";

const route = useRoute();
const isCollapse = ref(false);
const searchQuery = ref("");

// 新增：动态菜单激活状态
const activeMenu = computed(() => {
  return route.matched[0]?.path || route.path;
});

const isAuthPage = computed(() => {
  return ["/login", "/register", "/forgot-password"].includes(route.path);
});

const toggleSidebar = () => {
  isCollapse.value = !isCollapse.value;
};
</script>

<style>
html,
body {
  margin: 0;
  padding: 0;
  height: 100%;
  width: 100%;
}

#app {
  height: 100%;
}

.app-container {
  height: 100vh;
  background-color: #f0f2f5;
}

.sidebar {
  background-color: #001529;
  transition: width 0.3s;
  overflow: hidden;
}

.logo {
  height: 60px;
  display: flex;
  align-items: center;
  padding: 0 20px;
  color: #fff;
  transition: all 0.3s;
}

.logo-collapsed {
  padding: 0;
  justify-content: center;
}

.logo-image {
  width: 30px;
  height: 30px;
  margin-right: 10px;
}

.logo-text {
  font-size: 18px;
  font-weight: bold;
  white-space: nowrap;
  color: #fff;
}

.el-menu-vertical:not(.el-menu--collapse) {
  width: 240px;
}

.el-menu-vertical.el-menu--collapse {
  width: 64px;
}

.main-container {
  display: flex;
  flex-direction: column;
  width: calc(100% - 240px);
  transition: width 0.3s;
}

.main-container:has(+ .sidebar .el-menu--collapse) {
  width: calc(100% - 64px);
}

.header {
  background-color: #fff;
  box-shadow: 0 1px 4px rgba(0, 21, 41, 0.08);
  display: flex;
  align-items: center;
  padding: 0 20px;
}

.toggle-sidebar {
  margin-right: 20px;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-grow: 1;
}

.search-input {
  width: 200px;
  margin-right: 20px;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.el-menu-vertical {
  background-color: #001529;
}

.el-menu-item {
  color: #a6adb4 !important;
}

.el-menu-item.is-active {
  color: #409eff !important;
  background-color: #f0f7ff !important;
}

.el-menu-item:hover {
  color: #fff !important;
  background-color: #1890ff !important;
}

@media (max-width: 768px) {
  .sidebar {
    position: fixed;
    z-index: 1000;
    height: 100vh;
  }

  .main-container {
    width: 100% !important;
  }

  .header {
    padding: 0 10px;
  }

  .search-input {
    width: 150px;
    margin-right: 10px;
  }
}
</style>
