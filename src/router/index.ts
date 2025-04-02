import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import LoginView from '../views/LoginView.vue'
import ChatView from '@/views/ChatView.vue'
import CompanionView from '@/views/CompanionView.vue'
import VoiceView from '@/views/VoiceView.vue'
import NumbermanView from '@/views/NumbermanView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: LoginView,
      meta: {
        guest: true,
        layout: 'auth'
      }
    },
    {
      path: '/home',
      name: 'home',
      component: HomeView,
      meta: {
        requiresAuth: true,
        layout: 'default'
      }
    },
    {
      path: '/chat',
      name: 'chat',
      component: ChatView,
      meta: {
        requiresAuth: true,
        layout: 'default'
      }
    },
    {
      path: '/companion',
      name: 'companion',
      component: CompanionView,
      meta: {
        requiresAuth: true,
        layout: 'default'
      }
    },
    {
      path: '/voice',
      name: 'voice',
      component: VoiceView,
      meta: {
        requiresAuth: true,
        layout: 'default'
      }
    },
    {
      path: '/numberman',
      name: 'numberman',
      component: NumbermanView,
      meta: {
        requiresAuth: true,
        layout: 'default'
      }
    }
  ]
})

// 路由守卫
// router.beforeEach((to, from, next) => {
//   const isAuthenticated = localStorage.getItem('token')

//   if (to.meta.requiresAuth && !isAuthenticated) {
//     next('/login')
//   } else if (to.meta.guest && isAuthenticated) {
//     next('/')
//   } else {
//     next()
//   }
// })

export default router 