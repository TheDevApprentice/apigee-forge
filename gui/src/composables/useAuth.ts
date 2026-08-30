import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import type { AuthDto, AuthStorageDto } from '../types/bridge'

export type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>

const defaultInvoke: Invoke = (command, args) => tauriInvoke(command, args)

function errorMessage(error: unknown, fallback: string): string {
  if (typeof error === 'object' && error !== null && 'message' in error) {
    const message = (error as { message?: unknown }).message
    if (typeof message === 'string' && message.length > 0) return message
  }
  return fallback
}

export type AuthState = 'idle' | 'restoring' | 'authenticated' | 'reauthentication_required' | 'error'

export function useAuth(invoke: Invoke = defaultInvoke) {
  const context = ref<AuthDto | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const state = ref<AuthState>('idle')

  async function refresh() {
    loading.value = true
    error.value = null
    try {
      context.value = await invoke<AuthDto>('auth_status')
      state.value = context.value?.authenticated ? 'authenticated' : 'idle'
    } catch {
      state.value = 'error'
      error.value = 'Authentication status is unavailable.'
    } finally {
      loading.value = false
    }
  }

  async function restore() {
    loading.value = true
    state.value = 'restoring'
    error.value = null
    try {
      const restored = await invoke<AuthDto>('auth_restore')
      context.value = restored.authenticated ? restored : null
      if (restored.authenticated) state.value = 'authenticated'
      if (!restored.authenticated) {
        try {
          const storage = await invoke<AuthStorageDto>('auth_storage_status')
          if (!storage.refresh_token_stored) {
            state.value = 'reauthentication_required'
            error.value = 'Your Google session needs to be connected once before automatic reconnection can be used.'
          } else {
            state.value = 'idle'
          }
        } catch (caught) {
          error.value = errorMessage(caught, 'Google credential storage is unavailable.')
        }
      }
    } catch (caught) {
      state.value = 'error'
      error.value = errorMessage(caught, 'Saved Google session could not be restored.')
    } finally {
      loading.value = false
    }
  }

  async function login() {
    loading.value = true
    state.value = 'restoring'
    error.value = null
    try {
      context.value = await invoke<AuthDto>('auth_login')
      state.value = 'authenticated'
    } catch (caught) {
      state.value = 'error'
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
      state.value = 'idle'
    } catch {
      state.value = 'error'
      error.value = 'Logout could not be completed.'
    } finally {
      loading.value = false
    }
  }

  return { context, loading, error, state, refresh, restore, login, logout }
}
