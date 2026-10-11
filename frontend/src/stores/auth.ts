import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { apiFetch, setTokens, getAccessToken } from '../api/client'

export type Role = 'admin' | 'supervisor' | 'agent'

export interface User {
  id: string
  username: string
  email: string | null
  role: Role
  extension: string | null
}

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null)
  const loading = ref(false)

  const isAuthenticated = computed(() => !!user.value && !!getAccessToken())

  async function login(username: string, password: string): Promise<boolean> {
    loading.value = true
    try {
      const res = await fetch('http://localhost:8080/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      })
      if (!res.ok) return false
      const data = await res.json()
      setTokens(data.access_token, data.refresh_token)
      user.value = data.user
      return true
    } finally {
      loading.value = false
    }
  }

  async function fetchMe() {
    const res = await apiFetch('/api/auth/me')
    if (res.ok) user.value = await res.json()
    else user.value = null
  }

  async function logout() {
    const refresh = localStorage.getItem('refresh_token')
    if (refresh) {
      try {
        await fetch('http://localhost:8080/api/auth/logout', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ refresh_token: refresh }),
        })
      } catch { /* ignore */ }
    }
    setTokens(null, null)
    user.value = null
  }

  return { user, loading, isAuthenticated, login, logout, fetchMe }
})
