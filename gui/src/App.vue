<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAuth } from './composables/useAuth'
import { useSession } from './composables/useSession'
import type { AppMode, SessionDto } from './types/bridge'
import { useOrganizations } from './composables/useOrganizations'
import { useProxies } from './composables/useProxies'
import { useRoles } from './composables/useRoles'
import { useTemplateEditor } from './composables/useTemplateEditor'
import BaseButton from './components/base/BaseButton.vue'
import BaseCard from './components/base/BaseCard.vue'
import BaseChip from './components/base/BaseChip.vue'
import BaseEmptyState from './components/base/BaseEmptyState.vue'
import BaseErrorState from './components/base/BaseErrorState.vue'
import BaseSpinner from './components/base/BaseSpinner.vue'

type NavigationItem = {
  label: string
  path: string
}

const navigation: NavigationItem[] = [
  { label: 'Dashboard', path: 'M3 12 12 3l9 9M5 10v10h14V10' },
  { label: 'Templates', path: 'M4 5h16v14H4zM8 9h8M8 13h5' },
  { label: 'Proxies', path: 'M5 7h14M5 12h14M5 17h14' },
  { label: 'Deployments', path: 'M12 3v13M7 11l5 5 5-5M5 21h14' },
  { label: 'Settings', path: 'M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1' },
]

const activeView = ref('Dashboard')
const selectedOrganization = ref('')
const selectedEnvironment = ref('')
const auth = useAuth()
const appSession = useSession()
const selectedMode = appSession.selectedMode
const organizations = useOrganizations()
const proxies = useProxies()
const roles = useRoles()
const templateEditor = useTemplateEditor()
const authContext = auth.context
const authLoading = auth.loading
const authError = auth.error
const organizationList = organizations.organizations
const environmentList = organizations.environments
const organizationsLoading = organizations.loading
const organizationsError = organizations.error
const proxyList = proxies.proxies
const proxiesLoading = proxies.loading
const proxiesError = proxies.error
const isDemo = computed(() => appSession.session.value?.mode === 'demo')
const isAuthenticated = computed(() => isDemo.value || auth.context.value?.authenticated === true)

onMounted(async () => {
  try {
    appSession.apply(await invoke<SessionDto>('session_status'))
  } catch {
    appSession.selectedMode.value = 'cloud'
  }
  if (!isDemo.value) {
    void auth.restore()
  }
})

async function changeMode(mode: AppMode) {
  if (mode === 'demo' && auth.context.value?.authenticated) {
    await auth.logout()
  }
  appSession.apply(await invoke<SessionDto>('set_app_mode', { mode }))
  auth.context.value = null
}

watch(isAuthenticated, (authenticated) => {
  if (authenticated) {
    void organizations.loadOrganizations()
  }
})

watch(selectedOrganization, (organization) => {
  selectedEnvironment.value = ''
  environmentList.value = []
  proxyList.value = []
  if (organization) {
    void organizations.loadEnvironments(organization)
    if (!isDemo.value) void roles.load(organization)
  }
})

watch(selectedEnvironment, (environment) => {
  if (selectedOrganization.value && environment) {
    void proxies.load(selectedOrganization.value, selectedEnvironment.value)
  }
})

function retryContext() {
  if (selectedOrganization.value) {
    void organizations.loadEnvironments(selectedOrganization.value)
  } else {
    void organizations.loadOrganizations()
  }
}

function retryProxies() {
  if (selectedOrganization.value && selectedEnvironment.value) {
    void proxies.load(selectedOrganization.value, selectedEnvironment.value)
  }
}

void templateEditor
</script>

<template>
  <div class="app-shell" :class="{ 'app-shell--locked': !isAuthenticated }">
    <aside v-if="isAuthenticated" class="sidebar" aria-label="Primary navigation">
      <div class="brand-mark" aria-label="Apigee Forge" title="Apigee Forge">AF</div>
      <nav class="sidebar__nav">
        <BaseButton
          v-for="item in navigation"
          :key="item.label"
          :label="item.label"
          :active="activeView === item.label"
          @click="activeView = item.label"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path :d="item.path" />
          </svg>
        </BaseButton>
      </nav>
      <div class="sidebar__footer">
        <span class="connection-dot" :aria-label="isAuthenticated ? 'Connected workspace' : 'Offline workspace'" />
      </div>
    </aside>

    <div class="app-frame">
      <header v-if="isAuthenticated" class="topbar">
        <div>
          <p class="topbar__eyebrow">Workspace</p>
          <p class="topbar__context">
            {{ selectedOrganization || 'No organization selected' }}
            <span>/</span>
            {{ selectedEnvironment || 'No environment selected' }}
          </p>
        </div>
        <label class="mode-switcher">
          <span>Mode</span>
          <select v-model="selectedMode" @change="changeMode(selectedMode as AppMode)">
            <option value="cloud">Live</option>
            <option value="demo">Demo</option>
          </select>
        </label>
      </header>

      <main class="main-content">
        <div class="page-heading">
          <div>
            <p class="page-heading__eyebrow">{{ isAuthenticated ? activeView : 'Login' }}</p>
            <h1>Apigee Forge</h1>
          </div>
          <span class="page-heading__status">{{ isAuthenticated ? 'Workspace connected' : 'Local mode' }}</span>
        </div>

        <template v-if="authLoading && !auth.context">
          <BaseCard eyebrow="Authentication">
            <div class="loading-state"><BaseSpinner /> <span>Checking session…</span></div>
          </BaseCard>
        </template>

        <template v-else-if="!isAuthenticated">
          <div class="login-screen__header">
            <span>Apigee Forge</span>
            <label class="mode-switcher">
              <span>Mode</span>
              <select v-model="selectedMode" @change="changeMode(selectedMode as AppMode)">
                <option value="cloud">Live</option>
                <option value="demo">Demo</option>
              </select>
            </label>
          </div>
          <BaseCard eyebrow="Welcome">
            <section class="login-panel" aria-labelledby="login-title">
              <div class="login-panel__copy">
                <h2 id="login-title">Connect your Apigee workspace.</h2>
                <p>Use the desktop OAuth flow to load organizations, environments and proxies. No credentials are required for this local preview.</p>
              </div>
              <button class="primary-action" type="button" :disabled="authLoading" @click="auth.login">
                {{ authLoading ? 'Opening sign-in…' : 'Sign in with Google' }}
              </button>
            </section>
          </BaseCard>
          <BaseErrorState v-if="authError" @retry="auth.refresh">
            <template #title>Authentication is not configured</template>
            <template #hint>{{ authError }} Set APIGEE_FORGE_OAUTH_CLIENT_ID before starting the GUI; the optional keyring alias defaults to desktop.</template>
          </BaseErrorState>
          <BaseCard v-if="isDemo" eyebrow="Demo workspace">
            <BaseEmptyState>
              <template #title>Offline workspace ready</template>
              <template #hint>The GUI is intentionally usable without a provisioned Apigee organization.</template>
            </BaseEmptyState>
          </BaseCard>
        </template>

        <template v-else>
          <BaseCard eyebrow="Workspace context">
            <div class="context-grid">
              <label>
                <span>Organization</span>
                <select v-model="selectedOrganization">
                  <option value="">Select an organization</option>
                  <option v-for="organization in organizationList" :key="organization.id" :value="organization.id">
                    {{ organization.id }}
                  </option>
                </select>
              </label>
              <label>
                <span>Environment</span>
                <select v-model="selectedEnvironment" :disabled="!selectedOrganization">
                  <option value="">Select an environment</option>
                  <option v-for="environment in environmentList" :key="environment.name" :value="environment.name">
                    {{ environment.name }}
                  </option>
                </select>
              </label>
            </div>
            <BaseSpinner v-if="organizationsLoading" />
            <BaseErrorState v-else-if="organizationsError" @retry="retryContext">
              <template #title>Workspace context unavailable</template>
              <template #hint>{{ organizationsError }}</template>
            </BaseErrorState>
            <BaseErrorState v-else-if="!isDemo && !organizationList.length">
              <template #title>No Apigee organization linked</template>
              <template #hint>Google authentication succeeded, but this account has no accessible Apigee organization or project.</template>
            </BaseErrorState>
            <BaseEmptyState v-else-if="!organizationList.length">
              <template #title>No Demo data loaded</template>
              <template #hint>The Demo dataset is intentionally deferred until the post-MVP tutorial.</template>
            </BaseEmptyState>
          </BaseCard>

          <BaseCard eyebrow="Identity and role">
            <div class="identity-row">
              <BaseChip :label="isDemo ? 'Demo data' : 'Live data'" />
              <div>
                <p class="identity-row__label">Authenticated identity</p>
                <p class="identity-row__value">{{ authContext?.identity || 'Desktop OAuth user' }}</p>
              </div>
              <BaseChip :label="authContext?.mode || 'demo'" />
            </div>
            <div class="role-list">
              <span class="identity-row__label">Role</span>
              <span v-if="isDemo">Demo operator</span>
              <span v-else-if="roles.loading">Loading role…</span>
              <span v-else-if="roles.error">Role unavailable</span>
              <span v-else>{{ roles.roles.map((role) => role.name).join(', ') || 'No role reported' }}</span>
            </div>
          </BaseCard>

          <BaseCard eyebrow="Proxies">
            <div v-if="proxiesLoading" class="loading-state"><BaseSpinner /> <span>Loading proxies…</span></div>
            <BaseErrorState v-else-if="proxiesError" @retry="retryProxies">
              <template #title>Proxies unavailable</template>
              <template #hint>{{ proxiesError }}</template>
            </BaseErrorState>
            <BaseEmptyState v-else-if="!selectedEnvironment || !proxyList.length">
              <template #title>{{ selectedEnvironment ? 'No proxies found' : 'Select an environment' }}</template>
              <template #hint>{{ selectedEnvironment ? 'This organization has no visible proxies.' : 'Choose an organization and environment to load proxies.' }}</template>
            </BaseEmptyState>
            <ul v-else class="proxy-list">
              <li v-for="proxy in proxyList" :key="proxy.name">
                <span>{{ proxy.name }}</span>
                <span class="proxy-list__revision">revision {{ proxy.revisions.at(-1)?.number || '—' }}</span>
              </li>
            </ul>
          </BaseCard>
        </template>
      </main>
    </div>
  </div>
</template>
