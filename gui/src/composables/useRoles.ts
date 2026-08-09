import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import type { Invoke } from './useAuth'
import type { RoleDto } from '../types/bridge'

const defaultInvoke: Invoke = (command, args) => tauriInvoke(command, args)

export function useRoles(invoke: Invoke = defaultInvoke) {
  const roles = ref<RoleDto[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function load(organization: string) {
    loading.value = true
    error.value = null
    try {
      roles.value = await invoke<RoleDto[]>('get_roles', { organization })
    } catch {
      error.value = 'Roles could not be loaded.'
    } finally {
      loading.value = false
    }
  }

  return { roles, loading, error, load }
}
