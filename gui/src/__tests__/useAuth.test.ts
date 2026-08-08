import { describe, expect, it, vi } from 'vitest'
import { useAuth, type Invoke } from '../composables/useAuth'
import type { AuthDto } from '../types/bridge'

const unauthenticated: AuthDto = {
  authenticated: false,
  mode: null,
  identity: null,
  project_id: null,
  selected_organization: null,
}

const authenticated: AuthDto = {
  authenticated: true,
  mode: 'desktop',
  identity: 'developer@example.com',
  project_id: null,
  selected_organization: null,
}

describe('useAuth', () => {
  it('keeps the offline state safe when authentication is unavailable', async () => {
    const invoke: Invoke = vi.fn().mockRejectedValue(new Error('not configured'))
    const auth = useAuth(invoke)

    await auth.login()

    expect(auth.context.value).toBeNull()
    expect(auth.error.value).toBe('Desktop authentication is unavailable.')
    expect(invoke).toHaveBeenCalledWith('auth_login')
  })

  it('stores the authenticated context returned by the bridge', async () => {
    const invoke: Invoke = vi.fn().mockResolvedValue(authenticated)
    const auth = useAuth(invoke)

    await auth.refresh()

    expect(auth.context.value).toEqual(authenticated)
    expect(auth.error.value).toBeNull()
    expect(invoke).toHaveBeenCalledWith('auth_status')
  })

  it('supports an unauthenticated response without selecting a workspace', async () => {
    const invoke: Invoke = vi.fn().mockResolvedValue(unauthenticated)
    const auth = useAuth(invoke)

    await auth.refresh()

    expect(auth.context.value?.authenticated).toBe(false)
    expect(auth.context.value?.selected_organization).toBeNull()
  })
})
