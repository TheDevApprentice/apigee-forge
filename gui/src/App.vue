<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import packageJson from '../package.json'
import { useAuth } from './composables/useAuth'
import { useSession } from './composables/useSession'
import type { AppMode, ProxyDto, RevisionDetailDto, SessionDto } from './types/bridge'
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
const selectedProxy = ref<ProxyDto | null>(null)
const selectedRevision = ref<number | null>(null)
const revisionDetail = ref<RevisionDetailDto | null>(null)
const revisionDetailLoading = ref(false)
const revisionDetailError = ref<string | null>(null)
const proxyFilter = ref<'all' | 'deployed' | 'not-deployed'>('all')
const auth = useAuth()
const appSession = useSession()
const selectedMode = appSession.selectedMode
const organizations = useOrganizations()
const proxies = useProxies()
const roles = useRoles()
const roleList = roles.roles
const rolesLoading = roles.loading
const rolesError = roles.error
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

watch(organizationList, (list) => {
  if (!selectedOrganization.value && list.length > 0) {
    selectedOrganization.value = list[0].id
  }
})

watch(environmentList, (list) => {
  if (!selectedEnvironment.value && list.length > 0) {
    selectedEnvironment.value = list[0].name
  }
})

watch(selectedOrganization, (organization) => {
  selectedEnvironment.value = ''
  environmentList.value = []
  proxyList.value = []
  selectedProxy.value = null
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

function openProxy(proxy: ProxyDto) {
  selectedProxy.value = proxy
  selectedRevision.value = null
  revisionDetail.value = null
  activeView.value = 'Proxies'
}

async function toggleRevision(revision: number) {
  if (!selectedProxy.value || selectedRevision.value === revision) {
    selectedRevision.value = null
    revisionDetail.value = null
    return
  }
  selectedRevision.value = revision
  revisionDetail.value = null
  revisionDetailError.value = null
  revisionDetailLoading.value = true
  try {
    revisionDetail.value = await invoke<RevisionDetailDto>('get_revision_detail', {
      organization: selectedOrganization.value,
      proxy_name: selectedProxy.value.name,
      revision,
    })
  } catch (caught) {
    const message = typeof caught === 'object' && caught !== null && 'message' in caught
      ? (caught as { message?: unknown }).message
      : null
    revisionDetailError.value = typeof message === 'string' && message.length > 0
      ? message
      : 'Revision details could not be loaded.'
  } finally {
    revisionDetailLoading.value = false
  }
}

const visibleProxies = computed(() => proxyList.value.filter((proxy) => {
  if (proxyFilter.value === 'all') return true
  return proxy.revisions.some((revision) => proxyFilter.value === 'deployed'
    ? revision.status === 'Succeeded'
    : revision.status === 'NotDeployed')
}))

const dashboardMetrics = computed(() => ({
  proxies: proxyList.value.length,
  revisions: proxyList.value.reduce((total, proxy) => total + proxy.revisions.length, 0),
  deployedProxies: proxyList.value.filter((proxy) => proxy.revisions.some((revision) => revision.status === 'Succeeded')).length,
  deployedRevisions: proxyList.value.reduce((total, proxy) => total + proxy.revisions.filter((revision) => revision.status === 'Succeeded').length, 0),
}))

const appInfo = {
  version: packageJson.version,
  build: 'Development desktop build',
  stack: 'Vue + Tauri + Rust',
  branch: 'feature/m6-bis-gui',
}

const profileIdentity = computed(() => authContext.value?.identity || (isDemo.value ? 'Demo workspace' : 'Not signed in'))
const profileInitials = computed(() => {
  if (isDemo.value) return 'DF'
  const name = profileIdentity.value.split('@')[0].replace(/[._-]+/g, ' ').trim()
  const parts = name.split(/\s+/).filter(Boolean)
  return (parts.length > 1 ? `${parts[0][0]}${parts.at(-1)?.[0]}` : name.slice(0, 2)).toUpperCase() || 'AF'
})

void templateEditor
</script>

<template>
  <div class="app-shell" :class="{ 'app-shell--locked': !isAuthenticated }">
    <aside class="sidebar" aria-label="Primary navigation">
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
        <div class="sidebar__profile" tabindex="0" :aria-label="`User profile: ${profileIdentity}`">
          <div class="sidebar__avatar" aria-hidden="true">{{ profileInitials }}</div>
          <span class="connection-dot" :class="{ 'connection-dot--connected': isAuthenticated }" />
          <div class="sidebar__profile-tooltip" role="status">
            <strong>{{ profileIdentity }}</strong>
            <span>{{ isAuthenticated ? 'Connected' : 'Not connected' }}</span>
            <span>{{ isDemo ? 'Demo mode' : 'Live mode' }}</span>
            <span v-if="selectedOrganization">{{ selectedOrganization }} / {{ selectedEnvironment || 'No environment' }}</span>
          </div>
        </div>
      </div>
    </aside>

    <div class="app-frame">
      <header v-if="isAuthenticated" class="topbar">
        <div class="topbar__workspace">
          <p class="topbar__eyebrow">Workspace</p>
          <div class="workspace-selectors">
            <label>
              <span>Organization</span>
              <select v-model="selectedOrganization" :disabled="organizationsLoading">
                <option value="">Select an organization</option>
                <option v-for="organization in organizationList" :key="organization.id" :value="organization.id">
                  {{ organization.id }}
                </option>
              </select>
            </label>
            <span class="workspace-selector__separator">/</span>
            <label>
              <span>Environment</span>
              <select v-if="selectedOrganization" v-model="selectedEnvironment" :disabled="organizationsLoading || !environmentList.length">
                <option value="">Select an environment</option>
                <option v-for="environment in environmentList" :key="environment.name" :value="environment.name">
                  {{ environment.name }}
                </option>
              </select>
              <span v-else class="workspace-selectors__placeholder">Select organization first</span>
            </label>
          </div>
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
          <template v-if="activeView === 'Dashboard'">
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

          <section class="dashboard-metrics" aria-label="Workspace summary">
            <BaseCard eyebrow="API proxies">
              <strong class="metric-card__value">{{ dashboardMetrics.proxies }}</strong>
              <span class="metric-card__hint">Visible in this environment</span>
            </BaseCard>
            <BaseCard eyebrow="Revisions">
              <strong class="metric-card__value">{{ dashboardMetrics.revisions }}</strong>
              <span class="metric-card__hint">Available revisions</span>
            </BaseCard>
            <BaseCard eyebrow="Deployed proxies">
              <strong class="metric-card__value">{{ dashboardMetrics.deployedProxies }}</strong>
              <span class="metric-card__hint">{{ dashboardMetrics.deployedRevisions }} deployed revisions</span>
            </BaseCard>
          </section>

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
              <span v-else-if="rolesLoading">Loading role…</span>
              <span v-else-if="rolesError">{{ rolesError }}</span>
              <span v-else>{{ roleList.map((role) => role.name).join(', ') || 'No role reported' }}</span>
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
              <li v-for="proxy in visibleProxies" :key="proxy.name">
                <button type="button" class="proxy-list__button" @click="openProxy(proxy)">
                  <span>{{ proxy.name }}</span>
                  <span class="proxy-list__revision">revision {{ proxy.revisions.at(-1)?.number || '—' }}</span>
                </button>
              </li>
            </ul>
          </BaseCard>
        </template>
        <template v-else-if="activeView === 'Proxies'">
          <BaseCard eyebrow="Proxy catalogue">
            <div class="proxy-filter" role="group" aria-label="Proxy deployment filter">
              <button type="button" :class="{ 'proxy-filter--active': proxyFilter === 'all' }" @click="proxyFilter = 'all'">All</button>
              <button type="button" :class="{ 'proxy-filter--active': proxyFilter === 'deployed' }" @click="proxyFilter = 'deployed'">Deployed</button>
              <button type="button" :class="{ 'proxy-filter--active': proxyFilter === 'not-deployed' }" @click="proxyFilter = 'not-deployed'">Not deployed</button>
            </div>
            <BaseEmptyState v-if="!visibleProxies.length">
              <template #title>No proxies match this filter</template>
              <template #hint>Choose another deployment state or change the workspace context.</template>
            </BaseEmptyState>
            <ul v-else class="proxy-list">
              <li v-for="proxy in visibleProxies" :key="proxy.name">
                <button type="button" class="proxy-list__button" @click="selectedProxy = proxy">
                  <span>{{ proxy.name }}</span>
                  <span class="proxy-list__revision">{{ proxy.revisions.some((revision) => revision.status === 'Succeeded') ? 'Deployed' : 'Not deployed' }}</span>
                </button>
              </li>
            </ul>
          </BaseCard>
          <BaseCard v-if="selectedProxy" eyebrow="Selected proxy details">
            <div class="proxy-detail">
              <div class="proxy-detail__header">
                <div>
                  <h2>{{ selectedProxy.name }}</h2>
                  <p>{{ selectedProxy.source === 'cloud' ? 'Live Apigee proxy' : 'Demo proxy' }}</p>
                </div>
                <BaseChip :label="selectedProxy.revisions.some((revision) => revision.status === 'Succeeded') ? 'Deployed' : 'Not deployed'" />
              </div>
              <dl class="proxy-metadata">
                <div><dt>Organization</dt><dd>{{ selectedOrganization }}</dd></div>
                <div><dt>Environment</dt><dd>{{ selectedEnvironment }}</dd></div>
                <div><dt>Revision count</dt><dd>{{ selectedProxy.revisions.length }}</dd></div>
              </dl>
              <h3>Revisions</h3>
              <ul class="proxy-revisions">
                <li v-for="revision in selectedProxy.revisions" :key="revision.number">
                  <button type="button" class="revision-row__button" @click="toggleRevision(revision.number)">
                    <span>Revision {{ revision.number }}</span>
                    <BaseChip :label="revision.status === 'Succeeded' ? 'Deployed' : revision.status === 'NotDeployed' ? 'Not deployed' : revision.status" />
                  </button>
                  <div v-if="selectedRevision === revision.number" class="revision-detail">
                    <BaseSpinner v-if="revisionDetailLoading" />
                    <BaseErrorState v-else-if="revisionDetailError">
                      <template #title>Revision unavailable</template>
                      <template #hint>{{ revisionDetailError }}</template>
                    </BaseErrorState>
                    <dl v-else-if="revisionDetail" class="revision-detail__metadata">
                      <div><dt>Revision</dt><dd>{{ revisionDetail.revision }}</dd></div>
                      <div><dt>Proxy</dt><dd>{{ revisionDetail.proxy_name }}</dd></div>
                      <div><dt>API fields</dt><dd>{{ Object.keys(revisionDetail.data).length }}</dd></div>
                    </dl>
                  </div>
                </li>
              </ul>
            </div>
          </BaseCard>
        </template>
        <template v-else-if="activeView === 'Templates'">
          <BaseCard eyebrow="Templates">
            <BaseEmptyState>
              <template #title>No templates loaded</template>
              <template #hint>Template files and the editor will be connected in the next M7 step.</template>
            </BaseEmptyState>
          </BaseCard>
        </template>
        <template v-else-if="activeView === 'Deployments'">
          <BaseCard eyebrow="Deployments">
            <BaseEmptyState>
              <template #title>No deployment selected</template>
              <template #hint>Select a proxy from the Dashboard to inspect its revisions.</template>
            </BaseEmptyState>
          </BaseCard>
        </template>
        <template v-else-if="activeView === 'Settings'">
          <BaseCard eyebrow="Application">
            <div class="settings-grid">
              <div class="settings-item"><span>Version</span><strong>{{ appInfo.version }}</strong></div>
              <div class="settings-item"><span>Build</span><strong>{{ appInfo.build }}</strong></div>
              <div class="settings-item"><span>Technology</span><strong>{{ appInfo.stack }}</strong></div>
              <div class="settings-item"><span>Source branch</span><strong>{{ appInfo.branch }}</strong></div>
            </div>
          </BaseCard>
          <BaseCard eyebrow="User profile">
            <div class="settings-profile">
              <div class="settings-profile__avatar" aria-hidden="true">{{ profileInitials }}</div>
              <div class="settings-profile__summary">
                <strong>{{ profileIdentity }}</strong>
                <span>{{ isAuthenticated ? 'Connected account' : 'Not connected' }}</span>
              </div>
              <div class="settings-profile__hover" role="status">
                <strong>{{ profileIdentity }}</strong>
                <span>{{ isDemo ? 'Demo mode' : 'Live mode' }}</span>
                <span>{{ isAuthenticated ? 'Session active' : 'Sign in to connect' }}</span>
              </div>
            </div>
          </BaseCard>
          <BaseCard eyebrow="Workspace session">
            <div class="settings-grid">
              <div class="settings-item"><span>Mode</span><strong>{{ isDemo ? 'Demo' : 'Live' }}</strong></div>
              <div class="settings-item"><span>Organization</span><strong>{{ selectedOrganization || 'Not selected' }}</strong></div>
              <div class="settings-item"><span>Environment</span><strong>{{ selectedEnvironment || 'Not selected' }}</strong></div>
              <div class="settings-item"><span>Identity</span><strong>{{ authContext?.identity || 'Local workspace' }}</strong></div>
            </div>
          </BaseCard>
          <BaseCard eyebrow="Resources">
            <nav class="resource-links" aria-label="Project resources">
              <a href="https://github.com/TheDevApprentice/apigee-forge" target="_blank" rel="noreferrer">Project on GitHub</a>
              <a href="https://cloud.google.com/apigee/docs" target="_blank" rel="noreferrer">Apigee documentation</a>
              <a href="https://cloud.google.com/apigee/docs/reference/apis/apigee/rest" target="_blank" rel="noreferrer">Apigee Management API</a>
              <a href="https://cloud.google.com/apigee/support" target="_blank" rel="noreferrer">Apigee support</a>
            </nav>
          </BaseCard>
          <BaseCard eyebrow="Available configuration">
            <BaseEmptyState>
              <template #title>No editable preferences yet</template>
              <template #hint>Authentication, storage and appearance preferences will appear here as they become configurable.</template>
            </BaseEmptyState>
          </BaseCard>
        </template>
        </template>
      </main>
    </div>
  </div>
</template>
