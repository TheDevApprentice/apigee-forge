import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import type { AuthDto } from '../types/bridge'

export function useAuth() {
  const context = ref<AuthDto | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function refresh() {
    loading.value = true
    error.value = null
    try {
      context.value = await invoke<AuthDto>('auth_status')
    } catch {
      error.value = 'Authentication status is unavailable.'
    } finally {
      loading.value = false
    }
  }

  async function login() {
    loading.value = true
    error.value = null
    try {
      context.value = await invoke<AuthDto>('auth_login')
    } catch {
      error.value = 'Desktop authentication is unavailable.'
    } finally {
      loading.value = false
    }
  }

  async function logout() {
    loading.value = true
    error.value = null
    try {
      await invoke('auth_logout')
      context.value = null
    } catch {
      error.value = 'Logout could not be completed.'
    } finally {
      loading.value = false
    }
  }

  return { context, loading, error, refresh, login, logout }
}
