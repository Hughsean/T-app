import axios, { InternalAxiosRequestConfig } from 'axios'
import type { RegisterForm, LoginForm, User, ApiResponse } from '@/types/user'
import { ElMessage } from 'element-plus'
import router from '@/router'


// 创建axios实例
const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  timeout: 5000,
  headers: {
    'Content-Type': 'application/json'
  },
  withCredentials: true
})

export const userApi = {
  // 注册
  register: (data: RegisterForm) => {
    return api.post<ApiResponse<User>>('/api/users/register', data, {
      headers: {
        'Content-Type': 'application/json'
      }
    })
  },

  // 登录
  login: (data: LoginForm) => {
    return api.post<ApiResponse<{ token: string; user: User }>>('/api/users/login', data)
  },

  // 获取用户信息
  getUserInfo: () => {
    return api.get<ApiResponse<User>>('/user/info')
  },

  // 检查用户名是否存在
  checkUsername: (username: string) => {
    return api.get<ApiResponse<{ data: boolean }>>(`api/users/check-username/${username}`)
  },

  // 检查邮箱是否存在
  checkEmail: (email: string) => {
    return api.get<ApiResponse<{ data: boolean }>>(`/api/users/check-email/${email}`)
  }
}

// 请求拦截器
api.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    // 添加请求日志
    console.log('Request:', {
      url: config.url,
      method: config.method,
      data: config.data,
      headers: config.headers
    })
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// 响应拦截器
api.interceptors.response.use(
  (response) => {
    // 添加响应日志
    console.log('Response:', response.data)
    return response.data
  },
  (error) => {
    // 添加错误日志
    console.error('API Error:', error.response || error)
    if (error.response) {
      const { status, data } = error.response
      switch (status) {
        case 400:
          ElMessage.error(data.message || '请求参数错误')
          break
        case 401:
          ElMessage.error('登录已过期，请重新登录')
          localStorage.removeItem('token')
          router.push('/login')
          break
        case 404:
          ElMessage.error('请求的资源不存在')
          break
        case 500:
          ElMessage.error('服务器错误')
          break
        default:
          ElMessage.error('网络错误')
      }
    } else if (error.request) {
      ElMessage.error('服务器无响应')
    } else {
      ElMessage.error('请求配置错误')
    }
    return Promise.reject(error)
  }
)

export default api 