<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const username = ref('')
const password = ref('')
const error = ref<string | null>(null)

const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

async function submit() {
  error.value = null
  const ok = await auth.login(username.value, password.value)
  if (!ok) {
    error.value = 'Invalid credentials'
    return
  }
  const redirect = (route.query.redirect as string) || '/dashboard'
  router.push(redirect)
}
</script>

<template>
  <div class="login">
    <h2>Sign in</h2>
    <form @submit.prevent="submit">
      <label>
        Username
        <input v-model="username" autocomplete="username" />
      </label>
      <label>
        Password
        <input v-model="password" type="password" autocomplete="current-password" />
      </label>
      <button type="submit" :disabled="auth.loading">
        {{ auth.loading ? 'Signing in…' : 'Sign in' }}
      </button>
      <p v-if="error" class="error">{{ error }}</p>
    </form>
  </div>
</template>

<style scoped>
.login { max-width: 320px; margin: 60px auto; }
h2 { margin-bottom: 16px; }
label { display: block; margin-bottom: 12px; font-size: 13px; color: #94a3b8; }
input { width: 100%; padding: 8px; margin-top: 4px; background: #1e293b; border: 1px solid #334155; color: #e2e8f0; border-radius: 4px; }
button { width: 100%; padding: 10px; margin-top: 8px; }
.error { color: #f87171; font-size: 13px; margin-top: 12px; }
</style>
