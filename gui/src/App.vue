<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import packageJson from '../package.json'
import { useAuth } from './composables/useAuth'
import { useSession } from './composables/useSession'
import type { AppMode, ProxyDto, RevisionDetailDto, SessionDto, TemplateDto } from './types/bridge'
import { useOrganizations } from './composables/useOrganizations'
import { useProxies } from './composables/useProxies'
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
const templateEditor = useTemplateEditor()
const templateList = ref<TemplateDto[]>([])
const templateSearch = ref('')
const templatesLoading = ref(false)
const templatesError = ref<string | null>(null)
const templateDeletePending = ref<string | null>(null)
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
    void loadTemplates()
  }
})

watch(activeView, (view) => {
  if (view === 'Templates' && isAuthenticated.value) void loadTemplates()
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

async function loadTemplates() {
  templatesLoading.value = true
  templatesError.value = null
  try {
    templateList.value = await invoke<TemplateDto[]>('list_templates')
  } catch {
    templatesError.value = 'Templates could not be loaded.'
  } finally {
    templatesLoading.value = false
  }
}

const visibleTemplates = computed(() => {
  const query = templateSearch.value.trim().toLowerCase()
  return query ? templateList.value.filter((template) => template.name.toLowerCase().includes(query)) : templateList.value
})

type TemplateMetadataDraft = {
  name: string
  description?: string
  owner: string
  target_environment?: string
  naming_convention: { prefix: string; case: string }
}

const metadataDraft = computed(() => {
  const metadata = templateEditor.current.value?.data.metadata as Partial<TemplateMetadataDraft> | undefined
  return {
    name: metadata?.name || '',
    description: metadata?.description || '',
    owner: metadata?.owner || '',
    target_environment: metadata?.target_environment || '',
    naming_convention: {
      prefix: metadata?.naming_convention?.prefix || '',
      case: metadata?.naming_convention?.case || 'kebab-case',
    },
  }
})

const metadataErrors = computed(() => ({
  name: metadataDraft.value.name.trim() ? '' : 'Name is required.',
  owner: metadataDraft.value.owner.trim() ? '' : 'Owner is required.',
  prefix: metadataDraft.value.naming_convention.prefix.trim() ? '' : 'Prefix is required.',
}))

const metadataValid = computed(() => Object.values(metadataErrors.value).every((message) => !message))
const selectedFlow = ref('pre_flow')

const flowDraft = computed(() => {
  const flow = templateEditor.current.value?.data.flow as Record<string, any> | undefined
  return {
    pre_flow: flow?.pre_flow || { request: [], response: [] },
    conditional_flows: Array.isArray(flow?.conditional_flows) ? flow.conditional_flows : [],
    post_flow: flow?.post_flow || { request: [], response: [] },
  }
})

function updateFlow(flow: Record<string, any>) {
  const current = templateEditor.current.value
  if (!current) return
  templateEditor.updateDraft({ ...current, data: { ...current.data, flow } })
}

function addConditionalFlow() {
  updateFlow({ ...flowDraft.value, conditional_flows: [...flowDraft.value.conditional_flows, { condition: '', request: [], response: [] }] })
  selectedFlow.value = `conditional_${flowDraft.value.conditional_flows.length}`
}

function removeConditionalFlow(index: number) {
  if (!window.confirm('Remove this conditional flow?')) return
  updateFlow({ ...flowDraft.value, conditional_flows: flowDraft.value.conditional_flows.filter((_, flowIndex) => flowIndex !== index) })
  selectedFlow.value = 'pre_flow'
}

function updateConditionalCondition(index: number, condition: string) {
  updateFlow({ ...flowDraft.value, conditional_flows: flowDraft.value.conditional_flows.map((flow, flowIndex) => flowIndex === index ? { ...flow, condition } : flow) })
}

function policyCount(stage: Record<string, any>) {
  return (stage.request?.length || 0) + (stage.response?.length || 0)
}

function updateMetadata(field: 'name' | 'description' | 'owner' | 'target_environment', value: string) {
  const current = templateEditor.current.value
  if (!current) return
  const metadata = { ...metadataDraft.value, [field]: value }
  templateEditor.updateDraft({ ...current, data: { ...current.data, metadata } })
}

function updatePrefix(value: string) {
  const current = templateEditor.current.value
  if (!current) return
  templateEditor.updateDraft({
    ...current,
    data: { ...current.data, metadata: { ...metadataDraft.value, naming_convention: { ...metadataDraft.value.naming_convention, prefix: value } } },
  })
}

function updateNamingCase(value: string) {
  const current = templateEditor.current.value
  if (!current) return
  templateEditor.updateDraft({
    ...current,
    data: { ...current.data, metadata: { ...metadataDraft.value, naming_convention: { ...metadataDraft.value.naming_convention, case: value } } },
  })
}

async function saveTemplate() {
  if (!metadataValid.value) return
  await templateEditor.save()
  if (templateEditor.current.value?.name) void loadTemplates()
}

async function selectTemplate(name: string) {
  await templateEditor.load(name)
}

function newTemplate() {
  if (templateEditor.dirty.value && !window.confirm('Discard the current template changes?')) return
  templateEditor.startNew({ metadata: { name: '', owner: '', naming_convention: { prefix: '', case: 'kebab-case' } }, flow: { pre_flow: {}, post_flow: {} } })
}

async function deleteTemplate(name: string) {
  if (!window.confirm(`Delete template "${name}"?`)) return
  templateDeletePending.value = name
  try {
    await invoke('delete_template', { name })
    templateList.value = templateList.value.filter((template) => template.name !== name)
    if (templateEditor.current.value?.name === name) templateEditor.discardChanges()
  } catch {
    templatesError.value = 'Template could not be deleted.'
  } finally {
    templateDeletePending.value = null
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

async function logout() {
  await auth.logout()
  selectedOrganization.value = ''
  selectedEnvironment.value = ''
  environmentList.value = []
  proxyList.value = []
  selectedProxy.value = null
  revisionDetail.value = null
  activeView.value = 'Dashboard'
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
const profileName = computed(() => authContext.value?.name || [authContext.value?.given_name, authContext.value?.family_name].filter(Boolean).join(' ') || '')
const profilePicture = computed(() => authContext.value?.picture || '')
const profileImageFailed = ref(false)
const profileInitials = computed(() => {
  if (isDemo.value) return 'DF'
  const source = profileName.value || profileIdentity.value.split('@')[0].replace(/[._-]+/g, ' ')
  const parts = source.trim().split(/\s+/).filter(Boolean)
  return (parts.length > 1 ? `${parts[0][0]}${parts.at(-1)?.[0]}` : source.slice(0, 2)).toUpperCase() || 'AF'
})

watch(profilePicture, () => {
  profileImageFailed.value = false
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
          <div class="sidebar__avatar" aria-hidden="true">
            <img v-if="profilePicture && !profileImageFailed" :src="profilePicture" alt="" @error="profileImageFailed = true" />
            <span v-else>{{ profileInitials }}</span>
          </div>
          <span class="connection-dot" :class="{ 'connection-dot--connected': isAuthenticated }" />
          <div class="sidebar__profile-tooltip" role="status">
            <div class="profile-tooltip__heading">
              <div class="profile-tooltip__avatar" aria-hidden="true">
                <img v-if="profilePicture && !profileImageFailed" :src="profilePicture" alt="" @error="profileImageFailed = true" />
                <span v-else>{{ profileInitials }}</span>
              </div>
              <div>
                <strong>{{ profileName || profileIdentity }}</strong>
                <span>{{ profileIdentity }}</span>
              </div>
            </div>
            <div class="profile-tooltip__status"><span class="profile-tooltip__dot" :class="{ 'profile-tooltip__dot--connected': isAuthenticated }" />{{ isAuthenticated ? 'Connected' : 'Not connected' }}</div>
            <span>{{ isDemo ? 'Demo mode' : 'Live mode' }}</span>
            <span v-if="selectedOrganization">Workspace: {{ selectedOrganization }} / {{ selectedEnvironment || 'No environment' }}</span>
            <button v-if="isAuthenticated && !isDemo" type="button" class="profile-tooltip__logout" @click="logout">Sign out</button>
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
            <div class="template-toolbar">
              <input v-model="templateSearch" type="search" placeholder="Search templates" aria-label="Search templates" />
              <button type="button" class="primary-action" @click="newTemplate">New template</button>
            </div>
            <div v-if="templatesLoading" class="loading-state"><BaseSpinner /> <span>Loading templates…</span></div>
            <BaseErrorState v-else-if="templatesError">
              <template #title>Templates unavailable</template>
              <template #hint>{{ templatesError }}</template>
            </BaseErrorState>
            <BaseEmptyState v-else-if="!templateList.length">
              <template #title>No templates loaded</template>
              <template #hint>Create your first local template to start the M7 editor.</template>
            </BaseEmptyState>
            <BaseEmptyState v-else-if="!visibleTemplates.length">
              <template #title>No templates match</template>
              <template #hint>Try another search term.</template>
            </BaseEmptyState>
            <ul v-else class="template-list">
              <li v-for="template in visibleTemplates" :key="template.name" :class="{ 'template-list__item--selected': templateEditor.current?.name === template.name }">
                <button type="button" class="template-list__select" @click="selectTemplate(template.name)">
                  <strong>{{ template.name || 'Untitled template' }}</strong>
                  <span>{{ String(template.data.metadata?.owner || 'No owner') }}</span>
                </button>
                <button type="button" class="template-list__delete" :disabled="templateDeletePending === template.name" @click="deleteTemplate(template.name)">Delete</button>
              </li>
            </ul>
          </BaseCard>
          <BaseCard v-if="templateEditor.current" eyebrow="Template metadata">
            <div class="metadata-form">
              <label><span>Name</span><input :value="metadataDraft.name" @input="updateMetadata('name', ($event.target as HTMLInputElement).value)" /><small v-if="metadataErrors.name">{{ metadataErrors.name }}</small></label>
              <label><span>Description</span><textarea :value="metadataDraft.description" rows="2" @input="updateMetadata('description', ($event.target as HTMLTextAreaElement).value)" /></label>
              <label><span>Owner</span><input :value="metadataDraft.owner" @input="updateMetadata('owner', ($event.target as HTMLInputElement).value)" /><small v-if="metadataErrors.owner">{{ metadataErrors.owner }}</small></label>
              <label><span>Target environment</span><select :value="metadataDraft.target_environment" @change="updateMetadata('target_environment', ($event.target as HTMLSelectElement).value)"><option value="">None</option><option value="dev">dev</option><option value="test">test</option><option value="prod">prod</option></select></label>
              <label><span>Name prefix</span><input :value="metadataDraft.naming_convention.prefix" @input="updatePrefix(($event.target as HTMLInputElement).value)" /><small v-if="metadataErrors.prefix">{{ metadataErrors.prefix }}</small></label>
              <label><span>Name case</span><select :value="metadataDraft.naming_convention.case" @change="updateNamingCase(($event.target as HTMLSelectElement).value)"><option value="kebab-case">kebab-case</option><option value="snake_case">snake_case</option><option value="camelCase">camelCase</option></select></label>
            </div>
          </BaseCard>
          <BaseCard v-if="templateEditor.current" eyebrow="Flow canvas">
            <div class="flow-canvas" aria-label="Template flow stages">
              <button type="button" class="flow-stage" :class="{ 'flow-stage--selected': selectedFlow === 'pre_flow' }" @click="selectedFlow = 'pre_flow'"><strong>PreFlow</strong><span>{{ policyCount(flowDraft.pre_flow) }} policies</span></button>
              <div v-for="(flow, index) in flowDraft.conditional_flows" :key="`conditional-${index}`" class="flow-stage flow-stage--conditional" :class="{ 'flow-stage--selected': selectedFlow === `conditional_${index}` }">
                <button type="button" class="flow-stage__main" @click="selectedFlow = `conditional_${index}`"><strong>Conditional Flow {{ index + 1 }}</strong><span>{{ policyCount(flow) }} policies</span></button>
                <input :value="flow.condition || ''" placeholder="Condition" aria-label="Conditional flow condition" @input="updateConditionalCondition(index, ($event.target as HTMLInputElement).value)" />
                <button type="button" class="flow-stage__remove" @click="removeConditionalFlow(index)">Remove</button>
              </div>
              <button type="button" class="flow-stage" :class="{ 'flow-stage--selected': selectedFlow === 'post_flow' }" @click="selectedFlow = 'post_flow'"><strong>PostFlow</strong><span>{{ policyCount(flowDraft.post_flow) }} policies</span></button>
            </div>
            <div class="flow-canvas__actions"><button type="button" @click="addConditionalFlow">Add conditional flow</button></div>
            <div class="flow-stage-detail">
              <span>Selected stage</span>
              <strong>{{ selectedFlow === 'pre_flow' ? 'PreFlow' : selectedFlow === 'post_flow' ? 'PostFlow' : `Conditional Flow ${Number(selectedFlow.split('_')[1]) + 1}` }}</strong>
              <span>Request: {{ selectedFlow === 'pre_flow' ? flowDraft.pre_flow.request?.length || 0 : selectedFlow === 'post_flow' ? flowDraft.post_flow.request?.length || 0 : flowDraft.conditional_flows[Number(selectedFlow.split('_')[1])]?.request?.length || 0 }} policies</span>
              <span>Response: {{ selectedFlow === 'pre_flow' ? flowDraft.pre_flow.response?.length || 0 : selectedFlow === 'post_flow' ? flowDraft.post_flow.response?.length || 0 : flowDraft.conditional_flows[Number(selectedFlow.split('_')[1])]?.response?.length || 0 }} policies</span>
            </div>
          </BaseCard>
          <BaseCard v-if="templateEditor.current" eyebrow="Template editor">
            <div class="template-editor-summary">
              <div>
                <h2>{{ templateEditor.current.name || 'New template' }}</h2>
                <p>{{ templateEditor.dirty ? 'Unsaved changes' : 'Saved template' }}</p>
              </div>
              <BaseChip :label="templateEditor.status" />
            </div>
            <div class="template-editor-actions">
              <button type="button" :disabled="!templateEditor.dirty" @click="templateEditor.reset">Reset</button>
              <button type="button" class="primary-action" :disabled="!templateEditor.dirty || templateEditor.status === 'saving'" @click="saveTemplate">Save</button>
            </div>
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
              <div class="settings-profile__avatar" aria-hidden="true">
                <img v-if="profilePicture && !profileImageFailed" :src="profilePicture" alt="" @error="profileImageFailed = true" />
                <span v-else>{{ profileInitials }}</span>
              </div>
              <div class="settings-profile__summary">
                <strong>{{ profileName || profileIdentity }}</strong>
                <span>{{ profileIdentity }}</span>
                <span>{{ isAuthenticated ? 'Connected account' : 'Not connected' }}</span>
              </div>
              <div class="settings-profile__hover" role="status">
                <strong>{{ profileName || profileIdentity }}</strong>
                <span>{{ profileIdentity }}</span>
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
