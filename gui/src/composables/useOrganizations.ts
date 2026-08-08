import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import type { EnvironmentDto, OrganizationDto } from '../types/bridge'

export function useOrganizations() {
  const organizations = ref<OrganizationDto[]>([])
  const environments = ref<EnvironmentDto[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadOrganizations() {
    loading.value = true
    error.value = null
    try {
      organizations.value = await invoke<OrganizationDto[]>('list_organizations')
    } catch {
      error.value = 'Organizations could not be loaded.'
    } finally {
      loading.value = false
    }
  }

  async function loadEnvironments(organization: string) {
    loading.value = true
    error.value = null
    try {
      environments.value = await invoke<EnvironmentDto[]>('list_environments', { organization })
    } catch {
      error.value = 'Environments could not be loaded.'
    } finally {
      loading.value = false
    }
  }

  return { organizations, environments, loading, error, loadOrganizations, loadEnvironments }
}
