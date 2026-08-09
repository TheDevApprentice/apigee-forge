import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import type { AuthDto } from '../types/bridge'

export type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>

const defaultInvoke: Invoke = (command, args) => tauriInvoke(command, args)

function errorMessage(error: unknown, fallback: string): string {
  if (typeof error === 'object' && error !== null && 'message' in error) {
    const message = (error as { message?: unknown }).message
    if (typeof message === 'string' && message.length > 0) return message
  }
  return fallback
}

export function useAuth(invoke: Invoke = defaultInvoke) {
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

  async function restore() {
    loading.value = true
    error.value = null
    try {
      const restored = await invoke<AuthDto>('auth_restore')
      context.value = restored.authenticated ? restored : null
    } catch (caught) {
      error.value = errorMessage(caught, 'Saved Google session could not be restored.')
    } finally {
      loading.value = false
    }
  }

  async function login() {
    loading.value = true
    error.value = null
    try {
      context.value = await invoke<AuthDto>('auth_login')
    } catch (caught) {
      error.value = errorMessage(caught, 'Desktop authentication is unavailable.')
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

  return { context, loading, error, refresh, restore, login, logout }
}
