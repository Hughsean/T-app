<template>
  <div class="login-container">
    <div class="login-background">
      <div class="circle circle-1"></div>
      <div class="circle circle-2"></div>
      <div class="circle circle-3"></div>
    </div>

    <el-card class="login-card">
      <div class="logo-container">
        <img src="/public/logo.svg" alt="Logo" class="logo">
        <h2>数字人多模态陪伴系统</h2>
      </div>

      <template #header>
        <div class="auth-header">
          <el-radio-group v-model="authMode" size="large">
            <el-radio-button label="login">登录</el-radio-button>
            <el-radio-button label="register">注册</el-radio-button>
          </el-radio-group>
        </div>
      </template>

      <!-- 登录表单 -->
      <el-form
        v-if="authMode === 'login'"
        ref="loginFormRef"
        :model="loginForm"
        :rules="loginRules"
        label-position="top"
      >
        <el-form-item label="用户名/邮箱" prop="username">
          <el-input
            v-model="loginForm.username"
            prefix-icon="User"
            placeholder="请输入用户名或邮箱"
          />
        </el-form-item>
        
        <el-form-item label="密码" prop="password">
          <el-input
            v-model="loginForm.password"
            prefix-icon="Lock"
            type="password"
            placeholder="请输入密码"
            show-password
          />
        </el-form-item>

        <div class="form-actions">
          <el-checkbox v-model="rememberMe">记住我</el-checkbox>
          <el-link type="primary" @click="forgotPassword">忘记密码？</el-link>
        </div>

        <el-button type="primary" class="submit-btn" @click="handleLogin" :loading="loading">
          登录
        </el-button>

        <div class="social-login">
          <div class="divider">
            <span>其他登录方式</span>
          </div>
          <div class="social-icons">
            <el-button circle class="social-icon wechat">
              <el-icon><svg-icon name="wechat" /></el-icon>
            </el-button>
            <el-button circle class="social-icon qq">
              <el-icon><svg-icon name="qq" /></el-icon>
            </el-button>
          </div>
        </div>
      </el-form>

      <!-- 注册表单 -->
      <el-form
        v-else
        ref="registerFormRef"
        :model="registerForm"
        :rules="registerRules"
        label-position="top"
      >
        <el-form-item label="邮箱" prop="email">
          <el-input
            v-model="registerForm.email"
            prefix-icon="Message"
            placeholder="请输入邮箱"
          />
        </el-form-item>

        <el-form-item label="用户名" prop="username">
          <el-input
            v-model="registerForm.username"
            prefix-icon="User"
            placeholder="请输入用户名"
          />
        </el-form-item>

        <el-form-item label="密码" prop="password">
          <el-input
            v-model="registerForm.password"
            prefix-icon="Lock"
            type="password"
            placeholder="请输入密码"
            show-password
          />
        </el-form-item>

        <el-form-item label="确认密码" prop="confirmPassword">
          <el-input
            v-model="registerForm.confirmPassword"
            prefix-icon="Lock"
            type="password"
            placeholder="请再次输入密码"
            show-password
          />
        </el-form-item>

        <el-button type="primary" class="submit-btn" @click="handleRegister" :loading="loading">
          注册
        </el-button>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { userApi } from '@/api/user'
import type { LoginForm, RegisterForm } from '@/types/user'

const router = useRouter()
const authMode = ref('login')
const loading = ref(false)
const rememberMe = ref(false)

// 登录表单
const loginFormRef = ref<FormInstance>()
const loginForm = reactive({
  username: '',
  password: ''
})

// 注册表单
const registerFormRef = ref<FormInstance>()
const registerForm = reactive({
  email: '',
  username: '',
  password: '',
  confirmPassword: ''
})

// 验证规则
const loginRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名或邮箱', trigger: 'blur' },
    { min: 3, message: '长度至少为3个字符', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码长度至少为6个字符', trigger: 'blur' }
  ]
}

const registerRules: FormRules = {
  email: [
    { required: true, message: '请输入邮箱地址', trigger: 'blur' },
    { type: 'email', message: '请输入正确的邮箱地址', trigger: 'blur' },
    { 
      validator: (rule: any, value: string, callback: (error?: Error) => void) => {
        if (!value) {
          callback()
          return
        }
        userApi.checkEmail(value)
          .then(response => {
            if (response.data) {
              callback(new Error('该邮箱已被注册'))
            } else {
              callback()
            }
          })
          .catch(() => {
            callback() // 如果检查失败，暂时允许通过
          })
      },
      trigger: 'blur'
    }
  ],
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 3, max: 20, message: '用户名长度应在3-20个字符之间', trigger: 'blur' },
    { pattern: /^[a-zA-Z0-9_-]+$/, message: '用户名只能包含字母、数字、下划线和横线', trigger: 'blur' },
    {
      validator: (rule: any, value: string, callback: (error?: Error) => void) => {
        if (!value) {
          callback()
          return
        }
        userApi.checkUsername(value)
          .then(response => {
            if (response.data) {
              callback(new Error('该用户名已被使用'))
            } else {
              callback()
            }
          })
          .catch(() => {
            callback() // 如果检查失败，暂时允许通过
          })
      },
      trigger: 'blur'
    }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, max: 20, message: '密码长度应在6-20个字符之间', trigger: 'blur' },
    { 
      pattern: /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[a-zA-Z\d]{6,20}$/,
      message: '密码必须包含大小写字母和数字',
      trigger: 'blur'
    }
  ],
  confirmPassword: [
    { required: true, message: '请再次输入密码', trigger: 'blur' },
    {
      validator: (rule: any, value: string, callback: Function) => {
        if (value !== registerForm.password) {
          callback(new Error('两次输入的密码不一致'))
        } else {
          callback()
        }
      },
      trigger: 'blur'
    }
  ]
}

// 处理登录
const handleLogin = async () => {
  if (!loginFormRef.value) return
  
  await loginFormRef.value.validate(async (valid) => {
    if (valid) {
      loading.value = true
      try {
        const loginData: LoginForm = {
          username: loginForm.username,
          password: loginForm.password
        }
        
        const response = await userApi.login(loginData)
        
        if (response.data.code === 0) {
          // 假设后端返回的 token 在 response.data.token 中
          if (response.data.data?.token) {
            localStorage.setItem('token', response.data.data.token)
          } else {
            console.error('Token is undefined')
          }
          ElMessage.success('登录成功')
          router.push('/') // 登录成功后跳转到首页
        } else {
          ElMessage.error(response.data?.message || '登录失败')
        }
      } catch (error: any) {
        console.error('登录错误:', error)
        ElMessage.error(error.response?.data?.message || '登录失败，请稍后重试')
      } finally {
        loading.value = false
      }
    }
  })
}

// 处理注册
const handleRegister = async () => {
  if (!registerFormRef.value) return
  
  await registerFormRef.value.validate(async (valid) => {
    if (valid) {
      try {
        loading.value = true
        const registerData: RegisterForm = {
          username: registerForm.username,
          password: registerForm.password,
          email: registerForm.email,
          nickname: registerForm.username // 默认使用用户名作为昵称
        }
        
        console.log('发送注册请求:', registerData)
        const response = await userApi.register(registerData)
        console.log('注册响应:', response)
        
        if (response.data.code === 0) {
          ElMessage.success('注册成功！')
          // 清空表单
          registerForm.username = ''
          registerForm.password = ''
          registerForm.confirmPassword = ''
          registerForm.email = ''
          // 切换到登录模式
          authMode.value = 'login'
        } else {
          ElMessage.error(response.data?.message || '注册失败')
        }
      } catch (error: any) {
        console.error('注册错误:', error)
        const errorMessage = error.response?.data?.message || '注册失败，请稍后重试'
        ElMessage.error(errorMessage)
      } finally {
        loading.value = false
      }
    }
  })
}

// 忘记密码
const forgotPassword = () => {
  // TODO: 实现忘记密码逻辑
  ElMessage.info('忘记密码功能开发中...')
}
</script>

<style scoped>
.login-container {
  min-height: 100vh;
  width: 100vw;
  display: flex;
  justify-content: center;
  align-items: center;
  background: linear-gradient(135deg, #409EFF 0%, #36cfc9 100%);
  position: relative;
  overflow: hidden;
}

.login-background {
  position: absolute;
  width: 100%;
  height: 100%;
  z-index: 0;
}

.circle {
  position: absolute;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
}

.circle-1 {
  width: 300px;
  height: 300px;
  top: -150px;
  right: -150px;
}

.circle-2 {
  width: 200px;
  height: 200px;
  bottom: -100px;
  left: -100px;
}

.circle-3 {
  width: 150px;
  height: 150px;
  top: 50%;
  right: 15%;
}

.login-card {
  width: 420px;
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
  position: relative;
  z-index: 1;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
}

.logo-container {
  text-align: center;
  margin-bottom: 30px;
}

.logo {
  width: 80px;
  height: 80px;
  margin-bottom: 16px;
}

.auth-header {
  display: flex;
  justify-content: center;
  margin-bottom: 20px;
}

.form-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.submit-btn {
  width: 100%;
  padding: 12px 0;
  font-size: 16px;
  margin-bottom: 24px;
}

.social-login {
  text-align: center;
}

.divider {
  position: relative;
  text-align: center;
  margin: 20px 0;
}

.divider::before,
.divider::after {
  content: '';
  position: absolute;
  top: 50%;
  width: 30%;
  height: 1px;
  background-color: #dcdfe6;
}

.divider::before {
  left: 0;
}

.divider::after {
  right: 0;
}

.divider span {
  background-color: white;
  padding: 0 10px;
  color: #909399;
  font-size: 14px;
}

.social-icons {
  display: flex;
  justify-content: center;
  gap: 20px;
}

.social-icon {
  font-size: 24px;
}

.social-icon.wechat {
  color: #07c160;
}

.social-icon.qq {
  color: #12b7f5;
}

@media (max-width: 480px) {
  .login-card {
    width: 90%;
    margin: 20px;
  }

  .circle {
    display: none;
  }
}
</style> 