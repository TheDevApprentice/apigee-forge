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
import BaseModal from './components/base/BaseModal.vue'
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
const templateView = ref<'catalogue' | 'editor' | 'review'>('catalogue')
const currentTemplate = templateEditor.current
const currentTemplateDirty = templateEditor.dirty
const currentTemplateStatus = templateEditor.status
const currentTemplateValidationErrors = templateEditor.validationErrors
const modal = ref<{ title: string; message: string; confirmLabel?: string; tone?: 'default' | 'danger' } | null>(null)
const modalAction = ref<(() => void | Promise<void>) | null>(null)
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

function templateOwner(template: TemplateDto): string {
  const metadata = template.data.metadata as { owner?: unknown } | undefined
  return typeof metadata?.owner === 'string' && metadata.owner.length > 0 ? metadata.owner : 'No owner'
}

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
const selectedLane = ref<'request' | 'response'>('request')
const selectedPolicyType = ref('security_api_key')
const policyTypes = [
  ['security_api_key', 'API key security'],
  ['security_oauth2', 'OAuth2 security'],
  ['security_jwt', 'JWT security'],
  ['quota', 'Quota'],
  ['spike_arrest', 'Spike arrest'],
  ['cors', 'CORS'],
  ['transform', 'Transform'],
] as const

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

async function askConfirmation(title: string, message: string, action: () => void | Promise<void>, tone: 'default' | 'danger' = 'default') {
  return new Promise<boolean>((resolve) => {
    modalAction.value = async () => { await action(); resolve(true); modal.value = null }
    modal.value = { title, message, confirmLabel: tone === 'danger' ? 'Delete' : 'Continue', tone }
  })
}

async function removeConditionalFlow(index: number) {
  await askConfirmation('Remove conditional flow?', 'The flow and its policies will be removed from this template.', () => {
    updateFlow({ ...flowDraft.value, conditional_flows: flowDraft.value.conditional_flows.filter((_, flowIndex) => flowIndex !== index) })
    selectedFlow.value = 'pre_flow'
  }, 'danger')
}

function updateConditionalCondition(index: number, condition: string) {
  updateFlow({ ...flowDraft.value, conditional_flows: flowDraft.value.conditional_flows.map((flow, flowIndex) => flowIndex === index ? { ...flow, condition } : flow) })
}

function policyCount(stage: Record<string, any>): number {
  const requestCount = Array.isArray(stage.request) ? stage.request.length : 0
  const responseCount = Array.isArray(stage.response) ? stage.response.length : 0
  return requestCount + responseCount
}

const totalPolicyCount = computed(() => {
  const conditionalCount = flowDraft.value.conditional_flows.reduce((total: number, flow: Record<string, any>) => total + policyCount(flow), 0)
  return policyCount(flowDraft.value.pre_flow) + policyCount(flowDraft.value.post_flow) + conditionalCount
})

const selectedStage = computed(() => selectedFlow.value === 'pre_flow'
  ? flowDraft.value.pre_flow
  : selectedFlow.value === 'post_flow'
    ? flowDraft.value.post_flow
    : flowDraft.value.conditional_flows[Number(selectedFlow.value.split('_')[1])] || { request: [], response: [] })
const selectedPolicies = computed(() => (selectedStage.value[selectedLane.value] || []) as Record<string, any>[])

function policyLabel(policy: Record<string, any>): string {
  return policyTypes.find(([value]) => value === String(policy.type))?.[1] || String(policy.type || 'Policy')
}

function policyText(policy: Record<string, any>, field: string): string {
  const value = policy[field]
  return typeof value === 'string' ? value : ''
}

function policyStringList(policy: Record<string, any>, field: string): string {
  return Array.isArray(policy[field]) ? policy[field].join(', ') : ''
}

function updatePolicyText(index: number, field: string, event: Event) {
  updatePolicyField(index, field, (event.target as HTMLInputElement | HTMLSelectElement).value)
}

function updatePolicyNumber(index: number, field: string, event: Event) {
  updatePolicyField(index, field, Number((event.target as HTMLInputElement).value))
}

function updatePolicyList(index: number, field: string, event: Event) {
  updatePolicyField(index, field, (event.target as HTMLInputElement).value.split(',').map((value) => value.trim()).filter(Boolean))
}

function policyFactory(type: string): Record<string, any> {
  const defaults: Record<string, Record<string, any>> = {
    security_api_key: { type, key_location: 'header', key_param_name: 'apikey' },
    security_oauth2: { type, scopes: [] },
    security_jwt: { type, algorithm: 'RS256', issuer: '', audience: '', public_key_source: 'jwks_url', jwks_url: '' },
    quota: { type, allow: 1000, interval: 1, time_unit: 'day', quota_type: 'default' },
    spike_arrest: { type, rate: 10, rate_unit: 'ps' },
    cors: { type, allow_origins: ['*'], allow_headers: [], allow_methods: ['GET', 'POST'], expose_headers: [], max_age_seconds: 3600, support_credentials: false },
    transform: { type, direction: 'json_to_xml' },
  }
  return JSON.parse(JSON.stringify(defaults[type] || defaults.security_api_key))
}

function updatePolicies(policies: Record<string, any>[]) {
  updateFlow({ ...flowDraft.value, [selectedFlow.value === 'pre_flow' ? 'pre_flow' : selectedFlow.value === 'post_flow' ? 'post_flow' : 'conditional_flows']: selectedFlow.value.startsWith('conditional_')
    ? flowDraft.value.conditional_flows.map((flow, index) => index === Number(selectedFlow.value.split('_')[1]) ? { ...flow, [selectedLane.value]: policies } : flow)
    : { ...selectedStage.value, [selectedLane.value]: policies } })
}

function addPolicy() {
  updatePolicies([...selectedPolicies.value, policyFactory(selectedPolicyType.value)])
}

function removePolicy(index: number) {
  updatePolicies(selectedPolicies.value.filter((_: unknown, policyIndex: number) => policyIndex !== index))
}

function movePolicy(index: number, direction: number) {
  const policies = [...selectedPolicies.value]
  const target = index + direction
  if (target < 0 || target >= policies.length) return
  const [policy] = policies.splice(index, 1)
  policies.splice(target, 0, policy)
  updatePolicies(policies)
}

function updatePolicyField(index: number, field: string, value: unknown) {
  const policies = selectedPolicies.value.map((policy: Record<string, any>, policyIndex: number) => policyIndex === index ? { ...policy, [field]: value } : policy)
  updatePolicies(policies)
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
  if (await templateEditor.load(name)) templateView.value = 'editor'
}

async function closeTemplateEditor() {
  if (templateEditor.dirty.value) {
    await askConfirmation('Leave editor?', 'Your unsaved changes will be discarded.', () => {
      templateEditor.discardChanges()
      templateView.value = 'catalogue'
    })
    return
  }
  templateEditor.discardChanges()
  templateView.value = 'catalogue'
}

function continueToReview() {
  if (metadataValid.value && templateEditor.current.value) templateView.value = 'review'
}
async function newTemplate() {
  if (templateEditor.dirty.value) {
    await askConfirmation('Start a new template?', 'Your unsaved changes will be discarded.', () => {
      templateEditor.startNew({ metadata: { name: '', owner: '', naming_convention: { prefix: '', case: 'kebab-case' } }, flow: { pre_flow: {}, post_flow: {} } })
      templateView.value = 'editor'
    })
    return
  }
  templateEditor.startNew({ metadata: { name: '', owner: '', naming_convention: { prefix: '', case: 'kebab-case' } }, flow: { pre_flow: {}, post_flow: {} } })
  templateView.value = 'editor'
}

async function deleteTemplate(name: string) {
  await askConfirmation('Delete template?', `Delete "${name}" from local storage?`, async () => {
    await performDeleteTemplate(name)
  }, 'danger')
}

async function performDeleteTemplate(name: string) {
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

function openCreateProxy() {
  modal.value = {
    title: 'Create proxy',
    message: 'The guided proxy wizard will be available in M8. It will support templates, OpenAPI specifications, bundle upload and an explicit deployment review.',
    confirmLabel: 'Close',
  }
  modalAction.value = () => { modal.value = null }
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

          <section class="dashboard-actions" aria-label="Quick actions">
            <button type="button" class="dashboard-action-card" @click="newTemplate">
              <span class="dashboard-action-card__icon">+</span>
              <span><strong>Create template</strong><small>Build a reusable template for the CLI and future proxies.</small></span>
              <span class="dashboard-action-card__arrow">→</span>
            </button>
            <button type="button" class="dashboard-action-card" @click="openCreateProxy">
              <span class="dashboard-action-card__icon">+</span>
              <span><strong>Create proxy</strong><small>Start the guided Apigee proxy workflow.</small></span>
              <span class="dashboard-action-card__arrow">→</span>
            </button>
          </section>

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
            <div class="proxy-catalogue__header">
              <div><p class="proxy-catalogue__intro">Manage proxies in the selected Apigee organization and environment.</p></div>
              <button type="button" class="primary-action" @click="openCreateProxy">Create proxy</button>
            </div>
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
          <template v-if="templateView === 'catalogue'">
          <BaseCard eyebrow="Template catalogue">
            <p class="template-catalogue__intro">Start from an existing template or create a new one. Your templates are stored locally and can be reused by the CLI.</p>
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
              <li v-for="template in visibleTemplates" :key="template.name" :class="{ 'template-list__item--selected': currentTemplate?.name === template.name }">
                <button type="button" class="template-list__select" @click="selectTemplate(template.name)">
                  <strong>{{ template.name || 'Untitled template' }}</strong>
                  <span>{{ templateOwner(template) }}</span>
                </button>
                <button type="button" class="template-list__delete" :disabled="templateDeletePending === template.name" @click="deleteTemplate(template.name)">Delete</button>
              </li>
            </ul>
          </BaseCard>
          </template>
          <template v-if="templateView === 'editor'">
          <BaseCard v-if="currentTemplate" eyebrow="Template workspace">
            <div class="template-workspace-header">
              <div>
                <span class="template-workspace__eyebrow">Editing template</span>
                <h2>{{ metadataDraft.name || 'New template' }}</h2>
                <p>Complete the details, configure the flow, then save your template.</p>
              </div>
              <div class="template-workspace__actions">
                <button type="button" @click="closeTemplateEditor">Back to templates</button>
                <button type="button" class="primary-action" :disabled="!metadataValid" @click="continueToReview">Continue to review</button>
              </div>
            </div>
            <nav class="template-steps" aria-label="Template editing steps">
              <span class="template-step template-step--active"><b>1</b> Details</span>
              <span class="template-step"><b>2</b> Flow</span>
              <span class="template-step"><b>3</b> Policies</span>
              <span class="template-step"><b>4</b> Save</span>
            </nav>
            <div class="metadata-form">
              <label for="template-name"><span>Name</span><input id="template-name" :value="metadataDraft.name" :aria-invalid="Boolean(metadataErrors.name)" aria-describedby="template-name-error" @input="updateMetadata('name', ($event.target as HTMLInputElement).value)" /><small id="template-name-error" v-if="metadataErrors.name">{{ metadataErrors.name }}</small></label>
              <label for="template-description"><span>Description</span><textarea id="template-description" :value="metadataDraft.description" rows="2" @input="updateMetadata('description', ($event.target as HTMLTextAreaElement).value)" /></label>
              <label for="template-owner"><span>Owner</span><input id="template-owner" :value="metadataDraft.owner" :aria-invalid="Boolean(metadataErrors.owner)" aria-describedby="template-owner-error" @input="updateMetadata('owner', ($event.target as HTMLInputElement).value)" /><small id="template-owner-error" v-if="metadataErrors.owner">{{ metadataErrors.owner }}</small></label>
              <label for="template-target-environment"><span>Target environment</span><select id="template-target-environment" :value="metadataDraft.target_environment" @change="updateMetadata('target_environment', ($event.target as HTMLSelectElement).value)"><option value="">None</option><option value="dev">dev</option><option value="test">test</option><option value="prod">prod</option></select></label>
              <label for="template-prefix"><span>Name prefix</span><input id="template-prefix" :value="metadataDraft.naming_convention.prefix" :aria-invalid="Boolean(metadataErrors.prefix)" aria-describedby="template-prefix-error" @input="updatePrefix(($event.target as HTMLInputElement).value)" /><small id="template-prefix-error" v-if="metadataErrors.prefix">{{ metadataErrors.prefix }}</small></label>
              <label for="template-name-case"><span>Name case</span><select id="template-name-case" :value="metadataDraft.naming_convention.case" @change="updateNamingCase(($event.target as HTMLSelectElement).value)"><option value="kebab-case">kebab-case</option><option value="snake_case">snake_case</option><option value="camelCase">camelCase</option></select></label>
            </div>
          </BaseCard>
          <BaseCard v-if="currentTemplate" eyebrow="2 · Flow and policies">
            <p class="editor-section__intro">Choose where a policy runs, then add and configure it in the selected request or response lane.</p>
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
              <span>Request: {{ selectedStage.request?.length || 0 }} policies</span>
              <span>Response: {{ selectedStage.response?.length || 0 }} policies</span>
            </div>
            <div class="policy-editor">
              <div class="policy-editor__toolbar">
                <div class="policy-lanes"><button type="button" :class="{ 'policy-lane--active': selectedLane === 'request' }" @click="selectedLane = 'request'">Request ({{ selectedStage.request?.length || 0 }})</button><button type="button" :class="{ 'policy-lane--active': selectedLane === 'response' }" @click="selectedLane = 'response'">Response ({{ selectedStage.response?.length || 0 }})</button></div>
                <div class="policy-add"><select v-model="selectedPolicyType" aria-label="Policy type"><option v-for="[value, label] in policyTypes" :key="value" :value="value">{{ label }}</option></select><button type="button" @click="addPolicy">Add policy</button></div>
              </div>
              <ol class="policy-list">
                <li v-for="(policy, index) in selectedPolicies" :key="`${policy.type}-${index}`" class="policy-item">
                  <div class="policy-item__header"><strong>{{ policyLabel(policy) }}</strong><div><button type="button" :aria-label="`Move policy ${index + 1} up`" :disabled="index === 0" @click="movePolicy(index, -1)">↑</button><button type="button" :aria-label="`Move policy ${index + 1} down`" :disabled="index === selectedPolicies.length - 1" @click="movePolicy(index, 1)">↓</button><button type="button" :aria-label="`Remove policy ${index + 1}`" @click="removePolicy(index)">Remove</button></div></div>
                  <div v-if="policy.type === 'security_api_key'" class="policy-fields"><label>Location<select :value="policy.key_location" @change="updatePolicyField(index, 'key_location', ($event.target as HTMLSelectElement).value)"><option value="header">Header</option><option value="query_param">Query param</option></select></label><label>Parameter<input :value="policy.key_param_name" @input="updatePolicyField(index, 'key_param_name', ($event.target as HTMLInputElement).value)" /></label></div>
                  <div v-else-if="policy.type === 'security_oauth2'" class="policy-fields"><label>Scopes<input :value="policyStringList(policy, 'scopes')" @input="updatePolicyField(index, 'scopes', ($event.target as HTMLInputElement).value.split(',').map((value) => value.trim()).filter(Boolean))" /></label></div>
                  <div v-else-if="policy.type === 'security_jwt'" class="policy-fields"><label>Algorithm<select :value="policy.algorithm" @change="updatePolicyField(index, 'algorithm', ($event.target as HTMLSelectElement).value)"><option>RS256</option><option>HS256</option></select></label><label>Issuer<input :value="policy.issuer" @input="updatePolicyField(index, 'issuer', ($event.target as HTMLInputElement).value)" /></label><label>Audience<input :value="policy.audience" @input="updatePolicyField(index, 'audience', ($event.target as HTMLInputElement).value)" /></label><label>JWKS URL<input :value="policy.jwks_url" @input="updatePolicyField(index, 'jwks_url', ($event.target as HTMLInputElement).value)" /></label></div>
                  <div v-else-if="policy.type === 'quota'" class="policy-fields"><label>Allow<input type="number" :value="policy.allow" @input="updatePolicyField(index, 'allow', Number(($event.target as HTMLInputElement).value))" /></label><label>Interval<input type="number" :value="policy.interval" @input="updatePolicyField(index, 'interval', Number(($event.target as HTMLInputElement).value))" /></label><label>Time unit<select :value="policy.time_unit" @change="updatePolicyField(index, 'time_unit', ($event.target as HTMLSelectElement).value)"><option>hour</option><option>day</option><option>week</option><option>month</option></select></label></div>
                  <div v-else-if="policy.type === 'spike_arrest'" class="policy-fields"><label>Rate<input type="number" :value="policy.rate" @input="updatePolicyField(index, 'rate', Number(($event.target as HTMLInputElement).value))" /></label><label>Unit<select :value="policy.rate_unit" @change="updatePolicyField(index, 'rate_unit', ($event.target as HTMLSelectElement).value)"><option>ps</option><option>pm</option></select></label></div>
                  <div v-else-if="policy.type === 'cors'" class="policy-fields"><label>Origins<input :value="policyStringList(policy, 'allow_origins')" @input="updatePolicyField(index, 'allow_origins', ($event.target as HTMLInputElement).value.split(',').map((value) => value.trim()).filter(Boolean))" /></label><label>Methods<input :value="policyStringList(policy, 'allow_methods')" @input="updatePolicyField(index, 'allow_methods', ($event.target as HTMLInputElement).value.split(',').map((value) => value.trim()).filter(Boolean))" /></label></div>
                  <div v-else-if="policy.type === 'transform'" class="policy-fields"><label>Direction<select :value="policy.direction" @change="updatePolicyField(index, 'direction', ($event.target as HTMLSelectElement).value)"><option value="json_to_xml">JSON to XML</option><option value="xml_to_json">XML to JSON</option></select></label></div>
                </li>
              </ol>
            </div>
            <div class="flow-canvas__continue"><button type="button" class="primary-action" :disabled="!metadataValid" @click="continueToReview">Continue to review</button></div>
          </BaseCard>
          <div v-if="currentTemplateValidationErrors.length" class="template-validation-errors" role="alert" aria-live="assertive">
            <strong>Template validation</strong>
            <button v-for="validationError in currentTemplateValidationErrors" :key="`${validationError.code}-${validationError.field}`" type="button">{{ validationError.field || 'Template' }}: {{ validationError.message }}</button>
          </div>
          </template>
          <template v-if="templateView === 'review'">
            <BaseCard eyebrow="Review and save">
              <div class="review-header"><div><h2>Ready to save</h2><p>Check the template summary before writing it to local storage.</p></div><BaseChip :label="currentTemplateDirty ? 'Unsaved changes' : 'Saved'" /></div>
              <div class="review-grid"><div><span>Name</span><strong>{{ metadataDraft.name || 'Missing' }}</strong></div><div><span>Owner</span><strong>{{ metadataDraft.owner || 'Missing' }}</strong></div><div><span>Target</span><strong>{{ metadataDraft.target_environment || 'None' }}</strong></div><div><span>Policies</span><strong>{{ totalPolicyCount }}</strong></div></div>
              <div class="review-actions"><button type="button" @click="templateView = 'editor'">Back to editor</button><button type="button" class="primary-action" :disabled="!metadataValid || !currentTemplateDirty || currentTemplateStatus === 'saving'" @click="saveTemplate">{{ currentTemplateStatus === 'saving' ? 'Saving…' : 'Save template' }}</button></div>
            </BaseCard>
          </template>
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
        <BaseModal v-if="modal" :open="true" :title="modal.title" :message="modal.message" :confirm-label="modal.confirmLabel" :tone="modal.tone" @close="modal = null" @confirm="modalAction?.()" />
      </main>
    </div>
  </div>
</template>
