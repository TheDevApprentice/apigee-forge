<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import packageJson from '../package.json'
import { useAuth } from './composables/useAuth'
import { useSession } from './composables/useSession'
import type { AppMode, DeploymentDto, ProxyDto, ProxyRevisionDto, RevisionDetailDto, SessionDto, TemplateDto } from './types/bridge'
import { useOrganizations } from './composables/useOrganizations'
import { useProxies } from './composables/useProxies'
import { useDeployment } from './composables/useDeployment'
import { useProxyCreationPreparation } from './composables/useDeploymentPreparation'
import { useTemplateEditor } from './composables/useTemplateEditor'
import BaseButton from './components/base/BaseButton.vue'
import BaseCard from './components/base/BaseCard.vue'
import BaseChip from './components/base/BaseChip.vue'
import BaseEmptyState from './components/base/BaseEmptyState.vue'
import BaseModal from './components/base/BaseModal.vue'
import BaseErrorState from './components/base/BaseErrorState.vue'
import BaseSpinner from './components/base/BaseSpinner.vue'
import BaseSelect from './components/base/BaseSelect.vue'
import TemplateEditorShell from './components/template/TemplateEditorShell.vue'
import ProxyCreationPreparation from './components/ProxyCreationPreparation.vue'
import FlowDiagram from './components/FlowDiagram.vue'
import ProxyDetailsDrawer from './components/ProxyDetailsDrawer.vue'
import TemplateDetailsDrawer from './components/TemplateDetailsDrawer.vue'
import DeploymentDetailsDrawer from './components/DeploymentDetailsDrawer.vue'

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
const proxySearch = ref('')
const selectedTemplate = ref<TemplateDto | null>(null)
const deploymentRevision = ref<number | null>(null)
const deploymentReviewConfirmed = ref(false)
const deploymentReviewError = ref<string | null>(null)
const revisionDetail = ref<RevisionDetailDto | null>(null)
const revisionDetailLoading = ref(false)
const revisionDetailError = ref<string | null>(null)
const proxyFilter = ref<'all' | 'deployed' | 'not-deployed'>('all')
const auth = useAuth()
const appSession = useSession()
const selectedMode = appSession.selectedMode
const organizations = useOrganizations()
const proxies = useProxies()
const deployment = useDeployment()
const proxyCreationPreparation = useProxyCreationPreparation()
const proxyCreationTemplate = proxyCreationPreparation.selectedTemplate
const proxyCreationOpenApiSource = proxyCreationPreparation.openApiSource
const proxyCreationProxyName = proxyCreationPreparation.proxyName
const proxyCreationErrors = proxyCreationPreparation.errors
const proxyCreationReady = proxyCreationPreparation.ready
const proxyCreationPreview = proxyCreationPreparation.preview
const proxyCreationLogicalTarget = proxyCreationPreparation.logicalTargetEnvironment
const proxyCreationStatus = proxyCreationPreparation.status
const proxyCreationGeneration = proxyCreationPreparation.generation
const proxyCreationCreatedRevision = proxyCreationPreparation.createdRevision
const proxyCreationError = proxyCreationPreparation.error
const deploymentResult = deployment.result
const deploymentStatus = deployment.status
const deploymentError = deployment.error
const deploymentLastUpdated = deployment.lastUpdated
const templateEditor = useTemplateEditor()
const templateList = ref<TemplateDto[]>([])
const templateSearch = ref('')
const templatesLoading = ref(false)
const templatesError = ref<string | null>(null)
const templateDeletePending = ref<string | null>(null)
const templateView = ref<'catalogue' | 'editor' | 'review'>('catalogue')
const proxyCreationMode = ref(false)
const editorStep = ref<1 | 2 | 3 | 4>(1)
const currentTemplate = templateEditor.current
const currentTemplateDirty = templateEditor.dirty
const currentTemplateStatus = templateEditor.status
const currentTemplateValidationErrors = templateEditor.validationErrors
const modal = ref<{ title: string; message: string; confirmLabel?: string; tone?: 'default' | 'danger' } | null>(null)
const modalAction = ref<(() => void | Promise<void>) | null>(null)
const authContext = auth.context
const authLoading = auth.loading
const authState = auth.state
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
let loginObserver: IntersectionObserver | null = null

function setupLoginObserver() {
  loginObserver?.disconnect()
  const sections = [...document.querySelectorAll<HTMLElement>('.login-experience .reveal-on-scroll')]
  if (!sections.length) return
  if (!('IntersectionObserver' in window)) {
    sections.forEach((section) => section.classList.add('is-visible'))
    return
  }
  const root = document.querySelector<HTMLElement>('.main-content')
  loginObserver = new IntersectionObserver((entries) => {
    entries.forEach((entry) => entry.target.classList.toggle('is-visible', entry.isIntersecting))
  }, { root, threshold: 0.12, rootMargin: '0px 0px -8% 0px' })
  sections.forEach((section) => loginObserver?.observe(section))
}

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

watch([authLoading, isAuthenticated], async ([loading, authenticated]) => {
  if (!loading && !authenticated) {
    await nextTick()
    setupLoginObserver()
  } else if (authenticated) {
    loginObserver?.disconnect()
    loginObserver = null
  }
})

onBeforeUnmount(() => {
  loginObserver?.disconnect()
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
  proxyCreationPreparation.setContext(selectedOrganization.value, environment)
  if (selectedOrganization.value && environment) {
    void proxies.load(selectedOrganization.value, selectedEnvironment.value)
  }
})

watch(selectedOrganization, (organization) => {
  proxyCreationPreparation.setContext(organization, selectedEnvironment.value)
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

type PolicyValidationError = { field: string; message: string }

function validatePolicyStage(stage: Record<string, any>, stageLabel: string): PolicyValidationError[] {
  const errors: PolicyValidationError[] = []
  for (const lane of ['request', 'response'] as const) {
    const policies = Array.isArray(stage[lane]) ? stage[lane] : []
    policies.forEach((policy: Record<string, any>, index: number) => {
      const prefix = `${stageLabel} / ${lane} / policy ${index + 1}`
      if (policy.type === 'security_jwt') {
        if (!String(policy.issuer || '').trim()) errors.push({ field: `${prefix} / issuer`, message: 'JWT issuer is required.' })
        if (!String(policy.audience || '').trim()) errors.push({ field: `${prefix} / audience`, message: 'JWT audience is required.' })
      }
      if (policy.type === 'quota' && (!Number(policy.allow) || !Number(policy.interval))) errors.push({ field: prefix, message: 'Quota allow and interval must be greater than zero.' })
      if (policy.type === 'spike_arrest' && !Number(policy.rate)) errors.push({ field: `${prefix} / rate`, message: 'Spike arrest rate must be greater than zero.' })
      if (policy.type === 'cors' && (!Array.isArray(policy.allow_origins) || policy.allow_origins.length === 0)) errors.push({ field: `${prefix} / origins`, message: 'At least one CORS origin is required.' })
    })
  }
  return errors
}

const flowValidationErrors = computed<PolicyValidationError[]>(() => {
  const flow = templateEditor.current.value?.data.flow as Record<string, any> | undefined
  const errors: PolicyValidationError[] = []
  for (const stage of ['pre_flow', 'post_flow'] as const) {
    if (!flow?.[stage] || !Array.isArray(flow[stage].request) || !Array.isArray(flow[stage].response)) {
      errors.push({ field: `flow.${stage}`, message: 'Request and response policy lists are required.' })
    }
  }
  if (!Array.isArray(flow?.conditional_flows)) {
    errors.push({ field: 'flow.conditional_flows', message: 'Conditional flows must be a list.' })
  } else {
    flow.conditional_flows.forEach((conditional: Record<string, any>, index: number) => {
      if (!Object.prototype.hasOwnProperty.call(conditional, 'condition')) errors.push({ field: `flow.conditional_flows.${index}.condition`, message: 'Conditional flow condition is required.' })
      if (!Array.isArray(conditional.request) || !Array.isArray(conditional.response)) errors.push({ field: `flow.conditional_flows.${index}`, message: 'Request and response policy lists are required.' })
    })
  }
  return errors
})

const policyValidationErrors = computed(() => [
  ...flowValidationErrors.value,
  ...validatePolicyStage(flowDraft.value.pre_flow, 'PreFlow'),
  ...flowDraft.value.conditional_flows.flatMap((flow, index) => validatePolicyStage(flow, `Conditional Flow ${index + 1}`)),
  ...validatePolicyStage(flowDraft.value.post_flow, 'PostFlow'),
])
const templateValid = computed(() => metadataValid.value && policyValidationErrors.value.length === 0)
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

function policyIconPath(policy: Record<string, any>): string {
  const icons: Record<string, string> = {
    security_api_key: 'M8 7V5a4 4 0 0 1 8 0v2M6 7h12v12H6z',
    security_oauth2: 'M12 3a5 5 0 0 0-5 5v2a5 5 0 0 0 10 0V8a5 5 0 0 0-5-5zM9 19h6',
    security_jwt: 'M12 3l7 3v5c0 4.5-3 8-7 10-4-2-7-5.5-7-10V6z',
    quota: 'M4 12h16M12 4v16M7 7l10 10M17 7L7 17',
    spike_arrest: 'M4 17l4-5 3 3 5-8 4 5',
    cors: 'M6 7h12M6 12h12M6 17h12',
    transform: 'M5 8h14M5 16h14M8 5l-3 3 3 3M16 13l3 3-3 3',
  }
  return icons[String(policy.type)] || 'M12 5v14M5 12h14'
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

function metadataPayload(draft: TemplateMetadataDraft): Record<string, unknown> {
  const metadata: Record<string, unknown> = {
    name: draft.name,
    owner: draft.owner,
    naming_convention: draft.naming_convention,
  }
  if (draft.description?.trim()) metadata.description = draft.description
  if (draft.target_environment) metadata.target_environment = draft.target_environment
  return metadata
}

function updateMetadata(field: 'name' | 'description' | 'owner' | 'target_environment', value: string) {
  const current = templateEditor.current.value
  if (!current) return
  const draft = { ...metadataDraft.value, [field]: value }
  templateEditor.updateDraft({ ...current, data: { ...current.data, metadata: metadataPayload(draft) } })
}

function updatePrefix(value: string) {
  const current = templateEditor.current.value
  if (!current) return
  const draft = { ...metadataDraft.value, naming_convention: { ...metadataDraft.value.naming_convention, prefix: value } }
  templateEditor.updateDraft({ ...current, data: { ...current.data, metadata: metadataPayload(draft) } })
}

function updateNamingCase(value: string) {
  const current = templateEditor.current.value
  if (!current) return
  const draft = { ...metadataDraft.value, naming_convention: { ...metadataDraft.value.naming_convention, case: value } }
  templateEditor.updateDraft({ ...current, data: { ...current.data, metadata: metadataPayload(draft) } })
}

function normalizeCurrentTemplate() {
  const current = templateEditor.current.value
  if (!current) return
  const flow = current.data.flow as Record<string, any> | undefined
  const metadata = metadataPayload(metadataDraft.value)
  const normalizeStage = (stage: Record<string, any> | undefined) => ({ request: Array.isArray(stage?.request) ? stage.request : [], response: Array.isArray(stage?.response) ? stage.response : [] })
  templateEditor.updateDraft({ ...current, data: { ...current.data, metadata, flow: { ...flow, pre_flow: normalizeStage(flow?.pre_flow), post_flow: normalizeStage(flow?.post_flow), conditional_flows: Array.isArray(flow?.conditional_flows) ? flow.conditional_flows.map((item) => ({ ...item, ...normalizeStage(item) })) : [] } } })
}

async function saveTemplate() {
  normalizeCurrentTemplate()
  if (!templateValid.value) return false
  const saved = await templateEditor.save()
  if (!saved) return false
  await loadTemplates()
  editorStep.value = 1
  templateView.value = 'catalogue'
  return true
}

function openTemplateDrawer(template: TemplateDto) {
  selectedTemplate.value = template
}

function closeTemplateDrawer() {
  selectedTemplate.value = null
}

async function editTemplate(name: string) {
  closeTemplateDrawer()
  if (await templateEditor.load(name)) {
    editorStep.value = 1
    templateView.value = 'editor'
  }
}

function selectProxyCreationTemplate(name: string) {
  const template = templateList.value.find((candidate) => candidate.name === name)
  if (template) proxyCreationPreparation.selectTemplate(template)
}

function prepareProxyCreation(name: string) {
  const template = templateList.value.find((candidate) => candidate.name === name)
  if (!template) return
  closeTemplateDrawer()
  proxyCreationMode.value = true
  proxyCreationPreparation.selectTemplate(template)
  proxyCreationPreparation.setContext(selectedOrganization.value, selectedEnvironment.value)
  activeView.value = 'Proxies'
  resetContentScroll()
}

async function uploadProxyCreation() {
  if (await proxyCreationPreparation.upload()) {
    if (selectedOrganization.value && selectedEnvironment.value) {
      await proxies.load(selectedOrganization.value, selectedEnvironment.value)
    }
  }
}

function cancelProxyCreationPreparation() {
  proxyCreationPreparation.clear()
  proxyCreationMode.value = false
  activeView.value = 'Proxies'
  resetContentScroll()
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

async function nextEditorStep() {
  normalizeCurrentTemplate()
  const valid = editorStep.value === 1 ? metadataValid.value : editorStep.value === 2 ? flowValidationErrors.value.length === 0 : templateValid.value
  if (!valid) return
  if (editorStep.value < 4) editorStep.value = (editorStep.value + 1) as 1 | 2 | 3 | 4
  if (editorStep.value === 4 && !(await templateEditor.validate())) return
}

async function continueToReview() {
  await nextEditorStep()
  if (editorStep.value === 4 && templateValid.value) templateView.value = 'review'
}

function previousEditorStep() {
  if (editorStep.value > 1) editorStep.value = (editorStep.value - 1) as 1 | 2 | 3 | 4
}
async function newTemplate() {
  activeView.value = 'Templates'
  resetContentScroll()
  if (templateEditor.dirty.value) {
    await askConfirmation('Start a new template?', 'Your unsaved changes will be discarded.', () => {
      templateEditor.startNew({ metadata: { name: '', owner: '', naming_convention: { prefix: '', case: 'kebab-case' } }, flow: { pre_flow: { request: [], response: [] }, conditional_flows: [], post_flow: { request: [], response: [] } } })
      editorStep.value = 1
      templateView.value = 'editor'
    })
    return
  }
  templateEditor.startNew({ metadata: { name: '', owner: '', naming_convention: { prefix: '', case: 'kebab-case' } }, flow: { pre_flow: { request: [], response: [] }, conditional_flows: [], post_flow: { request: [], response: [] } } })
  editorStep.value = 1
  templateView.value = 'editor'
}

async function deleteTemplate(name: string) {
  await askConfirmation('Delete template?', `Delete "${name}" from local storage?`, async () => {
    await performDeleteTemplate(name)
  }, 'danger')
}

async function performDeleteTemplate(name: string) {
  closeTemplateDrawer()
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

function resetContentScroll() {
  void nextTick(() => document.querySelector<HTMLElement>('.main-content')?.scrollTo({ top: 0, behavior: 'smooth' }))
}

function openCreateProxy() {
  proxyCreationMode.value = true
  proxyCreationPreparation.clear()
  proxyCreationPreparation.setContext(selectedOrganization.value, selectedEnvironment.value)
  void loadTemplates()
  activeView.value = 'Proxies'
  resetContentScroll()
}

function retryProxies() {
  if (selectedOrganization.value && selectedEnvironment.value) {
    void proxies.load(selectedOrganization.value, selectedEnvironment.value)
  }
}

function closeProxyDrawer() {
  selectedProxy.value = null
  selectedRevision.value = null
  revisionDetail.value = null
  revisionDetailError.value = null
}

function openProxy(proxy: ProxyDto) {
  deployment.reset()
  selectedProxy.value = proxy
  selectedRevision.value = null
  deploymentRevision.value = null
  deploymentReviewConfirmed.value = false
  deploymentReviewError.value = null
  revisionDetail.value = null
  activeView.value = 'Proxies'
}

function selectDeploymentCandidate(proxy: ProxyDto, revision: ProxyRevisionDto) {
  deployment.reset()
  selectedProxy.value = proxy
  selectedRevision.value = null
  deploymentRevision.value = revision.number
  deploymentReviewConfirmed.value = false
  deploymentReviewError.value = null
  revisionDetail.value = null
}

function closeDeploymentDetails() {
  deployment.stopPolling()
  selectedProxy.value = null
  deploymentRevision.value = null
  deploymentReviewConfirmed.value = false
  deploymentReviewError.value = null
  deployment.reset()
}

function reviewDeploymentRevision(revision: ProxyRevisionDto) {
  if (!selectedProxy.value) return
  if (revision.status === 'Succeeded') {
    deploymentReviewError.value = 'This revision is already deployed. Explicit replacement will be handled in M8-05.'
    return
  }
  if (revision.status !== 'NotDeployed') {
    deploymentReviewError.value = 'Only an existing revision that is not deployed can be reviewed here.'
    return
  }
  deployment.reset()
  deploymentRevision.value = revision.number
  deploymentReviewConfirmed.value = false
  deploymentReviewError.value = null
  activeView.value = 'Deployments'
  resetContentScroll()
}

async function confirmDeploymentReview() {
  if (!selectedProxy.value || !selectedDeploymentRevision.value) return
  await askConfirmation(
    'Confirm revision deployment?',
    `Review target ${selectedOrganization.value} / ${selectedEnvironment.value} for proxy ${selectedProxy.value.name}, revision ${selectedDeploymentRevision.value.number}.`,
    () => { deploymentReviewConfirmed.value = true },
  )
}

async function executeDeployment() {
  if (!deploymentReviewConfirmed.value || !selectedProxy.value || !selectedDeploymentRevision.value) return
  await deployment.deploy({
    organization: selectedOrganization.value,
    environment: selectedEnvironment.value,
    proxyName: selectedProxy.value.name,
    revision: selectedDeploymentRevision.value.number,
  }, false)
  if (selectedOrganization.value && selectedEnvironment.value) {
    await proxies.load(selectedOrganization.value, selectedEnvironment.value)
    const refreshedProxy = proxyList.value.find((proxy) => proxy.name === selectedProxy.value?.name)
    if (refreshedProxy) selectedProxy.value = refreshedProxy
  }
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

async function loadRevisionDetail(revision: number) {
  if (!selectedProxy.value) return
  selectedRevision.value = revision
  revisionDetail.value = null
  revisionDetailError.value = null
  revisionDetailLoading.value = true
  try {
    revisionDetail.value = await invoke<RevisionDetailDto>('get_revision_detail', {
      organization: selectedOrganization.value,
      proxyName: selectedProxy.value.name,
      revision,
    })
  } catch (caught) {
    const message = typeof caught === 'object' && caught !== null && 'message' in caught
      ? (caught as { message?: unknown }).message
      : typeof caught === 'string' ? caught : null
    revisionDetailError.value = typeof message === 'string' && message.length > 0
      ? message
      : 'Revision details could not be loaded. Check the selected proxy, revision and permissions, then retry.'
  } finally {
    revisionDetailLoading.value = false
  }
}

async function toggleRevision(revision: number) {
  if (!selectedProxy.value || selectedRevision.value === revision) {
    selectedRevision.value = null
    revisionDetail.value = null
    revisionDetailError.value = null
    return
  }
  await loadRevisionDetail(revision)
}

const visibleProxies = computed(() => {
  const query = proxySearch.value.trim().toLowerCase()
  return proxyList.value.filter((proxy) => {
    const matchesSearch = !query || proxy.name.toLowerCase().includes(query)
    const matchesFilter = proxyFilter.value === 'all' || proxy.revisions.some((revision) => proxyFilter.value === 'deployed'
      ? revision.status === 'Succeeded'
      : revision.status === 'NotDeployed')
    return matchesSearch && matchesFilter
  })
})

const selectedDeploymentRevision = computed<ProxyRevisionDto | null>(() => {
  if (!selectedProxy.value || deploymentRevision.value === null) return null
  return selectedProxy.value.revisions.find((revision) => revision.number === deploymentRevision.value) || null
})

const availableDeploymentRevisions = computed(() => proxyList.value.flatMap((proxy) => proxy.revisions
  .filter((revision) => revision.status === 'NotDeployed')
  .map((revision) => ({ proxy, revision }))))

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
  branch: 'feature/m9-design-polish',
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
            <label class="workspace-selector">
              <span>Organization</span>
              <BaseSelect
                v-model="selectedOrganization"
                label="Organization"
                placeholder="Select an organization"
                :disabled="organizationsLoading"
                :options="organizationList.map((organization) => ({ value: organization.id, label: organization.id, description: organization.project_id }))"
              />
            </label>
            <span class="workspace-selector__separator" aria-hidden="true">/</span>
            <label class="workspace-selector">
              <span>Environment</span>
              <BaseSelect
                v-if="selectedOrganization"
                v-model="selectedEnvironment"
                label="Environment"
                placeholder="Select an environment"
                :disabled="organizationsLoading || !environmentList.length"
                :options="environmentList.map((environment) => ({ value: environment.name, label: environment.name, description: 'Apigee environment' }))"
              />
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
        <div v-if="isAuthenticated" class="page-heading">
          <div>
            <p class="page-heading__eyebrow">{{ activeView }}</p>
          </div>
          <span class="page-heading__status">Workspace connected</span>
        </div>

        <template v-if="authLoading && !auth.context">
          <section class="login-loading" aria-live="polite">
            <div class="login-loading__mark" aria-hidden="true"><span></span><span></span><span></span></div>
            <p class="login-eyebrow">Apigee Forge</p>
            <h1>{{ authState === 'restoring' ? 'Restoring your workspace' : 'Preparing your workspace' }}</h1>
            <p>{{ authState === 'restoring' ? 'Checking your saved Google session securely…' : 'Getting things ready…' }}</p>
          </section>
        </template>

        <template v-else-if="!isAuthenticated">
          <section class="login-experience" aria-labelledby="login-title">
            <header class="login-experience__nav">
              <a class="login-brand" href="#login-title" aria-label="Apigee Forge home"><span class="login-brand__mark">AF</span><span>Apigee Forge</span></a>
              <label class="mode-switcher">
                <span>Workspace</span>
                <select v-model="selectedMode" @change="changeMode(selectedMode as AppMode)">
                  <option value="cloud">Live</option>
                  <option value="demo">Demo</option>
                </select>
              </label>
            </header>

            <div class="login-hero reveal-on-scroll">
              <div class="login-hero__copy">
                <p class="login-eyebrow">API delivery, made calm.</p>
                <h1 id="login-title">Shape your APIs.<br /><span>Ship with confidence.</span></h1>
                <p class="login-hero__lead">Apigee Forge brings templates, governance and deployments together in one thoughtful workspace for your APIs.</p>
                <div class="login-hero__actions">
                  <button class="primary-action login-hero__button" type="button" :disabled="authLoading" @click="auth.login">
                    <span class="google-g">G</span>{{ authLoading ? 'Connecting securely…' : 'Sign in with Google' }}
                  </button>
                  <a href="#how-it-works" class="login-text-link">See how it works <span aria-hidden="true">↓</span></a>
                </div>
                <p class="login-hero__note">Your Google session is restored automatically when it is still valid.</p>
              </div>
              <div class="login-hero__visual" aria-label="Apigee Forge workflow illustration" role="img">
                <div class="workflow-orbit workflow-orbit--outer"></div>
                <div class="workflow-orbit workflow-orbit--inner"></div>
                <div class="workflow-card workflow-card--main"><span class="workflow-card__icon">✦</span><span><b>Proxy workflow</b><small>Ready to deploy</small></span><strong>✓</strong></div>
                <div class="workflow-card workflow-card--template"><span class="workflow-dot workflow-dot--blue"></span><span><b>Template</b><small>Governance</small></span></div>
                <div class="workflow-card workflow-card--deploy"><span class="workflow-dot workflow-dot--green"></span><span><b>Revision 03</b><small>Live · eval</small></span></div>
                <svg class="workflow-lines" viewBox="0 0 420 360" aria-hidden="true"><path d="M88 114C120 60 178 49 213 105M260 166C303 173 316 208 322 238" /><circle cx="88" cy="114" r="4" /><circle cx="322" cy="238" r="4" /></svg>
              </div>
            </div>

            <div id="how-it-works" class="login-story">
              <div class="login-story__intro reveal-on-scroll"><p class="login-eyebrow">A clear path from idea to production</p><h2>Everything your API team needs.<br /><span>Nothing in the way.</span></h2></div>
              <div class="login-feature-grid">
                <article class="login-feature reveal-on-scroll"><div class="feature-visual feature-visual--flow"><span class="feature-node feature-node--active">01</span><i></i><span class="feature-node">02</span><i></i><span class="feature-node">03</span></div><p class="login-eyebrow">01 · Compose</p><h3>Make standards reusable.</h3><p>Turn your governance rules into visual templates that stay readable, versionable and ready for every team.</p></article>
                <article class="login-feature reveal-on-scroll"><div class="feature-visual feature-visual--layers"><span></span><span></span><span></span><b>OpenAPI</b></div><p class="login-eyebrow">02 · Prepare</p><h3>See the whole picture.</h3><p>Bring an OpenAPI specification and a template together, review the target and generate a bundle locally before anything changes.</p></article>
                <article class="login-feature reveal-on-scroll"><div class="feature-visual feature-visual--signal"><span></span><span></span><span></span><span></span><b>Live status</b></div><p class="login-eyebrow">03 · Deliver</p><h3>Deploy deliberately.</h3><p>Review the exact revision and environment, confirm once, then follow the deployment until Apigee is done.</p></article>
              </div>
            </div>

            <section class="login-capabilities" aria-labelledby="capabilities-title">
              <div class="login-story__intro reveal-on-scroll"><p class="login-eyebrow">One product, two ways to work</p><h2 id="capabilities-title">From a local idea<br /><span>to a governed API.</span></h2><p class="login-section-lead">Forge gives platform teams a shared language for API delivery, whether they prefer a visual desktop workflow or an automated pipeline.</p></div>
              <div class="capability-row capability-row--reverse reveal-on-scroll">
                <div class="capability-copy"><p class="login-eyebrow">The visual editor</p><h3>Design your proxy flow at a glance.</h3><p>Place security, traffic control and transformation policies exactly where they belong. The visual flow makes each request and response step easy to understand before you save the template.</p><div class="capability-points"><span>PreFlow and PostFlow</span><span>Conditional flows</span><span>Guided policies</span></div></div>
                <div class="capability-visual capability-visual--core capability-visual--editor" aria-hidden="true"><div class="core-ring core-ring--one"></div><div class="core-ring core-ring--two"></div><div class="core-core">FLOW</div><span class="core-label core-label--top">PreFlow</span><span class="core-label core-label--left">Request</span><span class="core-label core-label--right">Response</span><span class="core-label core-label--bottom">PostFlow</span></div>
              </div>
              <div class="capability-row reveal-on-scroll">
                <div class="capability-copy"><p class="login-eyebrow">The desktop workspace</p><h3>A calmer way to manage Apigee.</h3><p>Connect with your Google identity, select an organization and environment, then see proxies, revisions and deployment status in one focused workspace.</p><div class="capability-points"><span>Live and Demo modes</span><span>Template editor</span><span>Deployment review</span></div></div>
                <div class="capability-visual capability-visual--workspace" aria-hidden="true"><div class="workspace-window"><span class="workspace-window__top"><i></i><i></i><i></i></span><span class="workspace-window__line workspace-window__line--long"></span><span class="workspace-window__line"></span><span class="workspace-window__line workspace-window__line--short"></span><span class="workspace-window__badge">Ready</span></div></div>
              </div>
              <div class="capability-row capability-row--reverse reveal-on-scroll">
                <div class="capability-copy"><p class="login-eyebrow">The command line</p><h3>Automation when you need it.</h3><p>Use the same core from a terminal or CI/CD pipeline. Generate bundles, import revisions, deploy to an environment and read machine-friendly JSON results without a GUI.</p><div class="terminal-snippet" aria-label="Example Apigee Forge CLI commands"><code><span>$</span> apigee-forge generate</code><code><span>$</span> apigee-forge deploy --json</code><code><span>✓</span> pipeline ready</code></div></div>
                <div class="capability-visual capability-visual--pipeline" aria-hidden="true"><span class="pipeline-node">CLI</span><i></i><span class="pipeline-node pipeline-node--active">CI/CD</span><i></i><span class="pipeline-node">Apigee</span><div class="pipeline-track"></div></div>
              </div>
            </section>

            <section class="login-journey reveal-on-scroll" aria-labelledby="journey-title"><div class="login-journey__header"><p class="login-eyebrow">A workflow with intention</p><h2 id="journey-title">Every step has a place.</h2><p>Nothing is hidden behind a button. Forge separates local preparation from remote mutations, so teams can move quickly without losing trust.</p></div><div class="journey-steps"><article><span>01</span><b>Compose</b><p>Create a reusable template with visual policies.</p></article><article><span>02</span><b>Prepare</b><p>Combine OpenAPI and standards into a preview.</p></article><article><span>03</span><b>Generate</b><p>Render and package a safe local bundle.</p></article><article><span>04</span><b>Deliver</b><p>Upload, review the revision and deploy deliberately.</p></article></div></section>

            <section class="login-security reveal-on-scroll" aria-labelledby="security-title"><div><p class="login-eyebrow">Safe by default</p><h2 id="security-title">Confidence is a feature.</h2><p>Google OAuth, OS credential storage, SQLCipher local state, strict validation and explicit confirmations protect the path from source to production.</p></div><div class="security-list"><span><b>✓</b> Google identity and IAM permissions</span><span><b>✓</b> No credentials in the interface</span><span><b>✓</b> Demo mode without network access</span><span><b>✓</b> Explicit review before mutations</span></div></section>

            <div class="login-trust reveal-on-scroll"><span>Built around your existing tools</span><b>Google Cloud</b><b>OpenAPI</b><b>Apigee</b><b>Rust</b></div>

            <BaseErrorState v-if="authError" class="login-error" @retry="auth.refresh">
              <template #title>Authentication is not configured</template>
              <template #hint>{{ authError }} Set APIGEE_FORGE_OAUTH_CLIENT_ID before starting the GUI; the optional keyring alias defaults to desktop.</template>
            </BaseErrorState>
            <BaseCard v-if="isDemo" class="login-demo-card" eyebrow="Demo workspace">
              <BaseEmptyState>
                <template #title>Offline workspace ready</template>
                <template #hint>The GUI is intentionally usable without a provisioned Apigee organization.</template>
              </BaseEmptyState>
            </BaseCard>
          </section>
        </template>



        <template v-else-if="activeView === 'Deployments'">
          <BaseCard eyebrow="Revisions awaiting deployment">
            <p class="deployment-catalogue__intro">Review an existing Apigee proxy revision before deploying it to the selected environment.</p>
            <BaseSpinner v-if="proxiesLoading" />
            <BaseErrorState v-else-if="proxiesError" @retry="retryProxies">
              <template #title>Revisions unavailable</template>
              <template #hint>{{ proxiesError }}</template>
            </BaseErrorState>
            <BaseEmptyState v-else-if="!availableDeploymentRevisions.length">
              <template #title>No revisions awaiting deployment</template>
              <template #hint>Upload a proxy revision or return later when a revision is available for deployment.</template>
            </BaseEmptyState>
            <ul v-else class="deployment-revision-list">
              <li v-for="candidate in availableDeploymentRevisions" :key="`${candidate.proxy.name}-${candidate.revision.number}`" :class="{ 'deployment-revision-list__item--selected': selectedProxy?.name === candidate.proxy.name && selectedDeploymentRevision?.number === candidate.revision.number }">
                <button type="button" class="deployment-revision-list__button" @click="selectDeploymentCandidate(candidate.proxy, candidate.revision)">
                  <span>{{ candidate.proxy.name }}</span>
                  <span class="deployment-revision-list__meta">Revision {{ candidate.revision.number }} <BaseChip label="Not deployed" tone="neutral" /></span>
                </button>
              </li>
            </ul>
          </BaseCard>
          <DeploymentDetailsDrawer
            v-if="selectedProxy && selectedDeploymentRevision"
            :open="true"
            :proxy="selectedProxy"
            :revision="selectedDeploymentRevision"
            :organization="selectedOrganization"
            :environment="selectedEnvironment"
            :demo="isDemo"
            :confirmed="deploymentReviewConfirmed"
            :deployment="deploymentResult"
            :status="deploymentStatus"
            :last-updated="deploymentLastUpdated"
            :error="deploymentReviewError || deploymentError"
            @close="closeDeploymentDetails"
            @confirm="confirmDeploymentReview"
            @deploy="executeDeployment"
            @stop="deployment.stopPolling()"
            @retry="executeDeployment"
          />
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
        <template v-else>
          <template v-if="activeView === 'Dashboard'">
            <section class="dashboard-welcome dashboard-reveal" aria-labelledby="dashboard-title">
              <div class="dashboard-welcome__copy">
                <p class="dashboard-eyebrow">{{ isDemo ? 'Demo workspace' : 'Your Apigee workspace' }}</p>
                <h1 id="dashboard-title">{{ profileName ? `Welcome back, ${profileName}.` : 'Welcome back.' }}</h1>
                <p>Everything is in place to shape, review and deliver your APIs.</p>
              </div>
              <div class="dashboard-welcome__context">
                <span class="dashboard-status-dot" aria-hidden="true"></span>
                <div><span>Current workspace</span><strong>{{ selectedOrganization || 'Select an organization' }}</strong><small>{{ selectedEnvironment || 'Select an environment' }}</small></div>
                <BaseChip :label="isDemo ? 'Demo' : 'Live'" :tone="isDemo ? 'warning' : 'success'" />
              </div>
            </section>

            <BaseErrorState v-if="organizationsError" class="dashboard-reveal" @retry="retryContext">
              <template #title>Workspace context unavailable</template>
              <template #hint>{{ organizationsError }}</template>
            </BaseErrorState>
            <BaseErrorState v-else-if="!isDemo && !organizationList.length && !organizationsLoading" class="dashboard-reveal">
              <template #title>No Apigee organization linked</template>
              <template #hint>Google authentication succeeded, but this account has no accessible Apigee organization or project.</template>
            </BaseErrorState>
            <BaseCard v-else-if="isDemo && !organizationList.length && !organizationsLoading" class="dashboard-reveal" eyebrow="Demo workspace">
              <BaseEmptyState>
                <template #title>No Demo data loaded</template>
                <template #hint>The Demo dataset is intentionally deferred until the post-MVP tutorial.</template>
              </BaseEmptyState>
            </BaseCard>

            <section class="dashboard-actions dashboard-reveal" aria-labelledby="quick-actions-title">
              <div class="dashboard-section-heading"><div><p class="dashboard-eyebrow">Start here</p><h2 id="quick-actions-title">What would you like to do?</h2></div><span class="dashboard-section-heading__hint">Two simple ways to begin</span></div>
              <div class="dashboard-action-grid">
                <button type="button" class="dashboard-action-card" @click="newTemplate">
                  <span class="dashboard-action-card__icon" aria-hidden="true">+</span>
                  <span><strong>Create template</strong><small>Build a reusable template for the CLI and future proxies.</small></span>
                  <span class="dashboard-action-card__arrow">→</span>
                </button>
                <button type="button" class="dashboard-action-card dashboard-action-card--accent" @click="openCreateProxy">
                  <span class="dashboard-action-card__icon" aria-hidden="true">+</span>
                  <span><strong>Create proxy</strong><small>Start the guided Apigee proxy workflow.</small></span>
                  <span class="dashboard-action-card__arrow">→</span>
                </button>
              </div>
            </section>

            <section class="dashboard-metrics dashboard-reveal" aria-labelledby="summary-title">
              <div class="dashboard-section-heading dashboard-section-heading--compact"><div><p class="dashboard-eyebrow">At a glance</p><h2 id="summary-title">Workspace summary</h2></div><span v-if="organizationsLoading || proxiesLoading" class="dashboard-syncing"><BaseSpinner /> Syncing</span></div>
              <div class="dashboard-metric-grid">
                <BaseCard class="dashboard-metric-card"><span class="dashboard-metric-card__icon dashboard-metric-card__icon--blue">◇</span><span class="dashboard-metric-card__label">API proxies</span><strong class="metric-card__value">{{ dashboardMetrics.proxies }}</strong><span class="metric-card__hint">Visible in this environment</span></BaseCard>
                <BaseCard class="dashboard-metric-card"><span class="dashboard-metric-card__icon dashboard-metric-card__icon--teal">↗</span><span class="dashboard-metric-card__label">Revisions</span><strong class="metric-card__value">{{ dashboardMetrics.revisions }}</strong><span class="metric-card__hint">Available revisions</span></BaseCard>
                <BaseCard class="dashboard-metric-card"><span class="dashboard-metric-card__icon dashboard-metric-card__icon--green">✓</span><span class="dashboard-metric-card__label">Deployed proxies</span><strong class="metric-card__value">{{ dashboardMetrics.deployedProxies }}</strong><span class="metric-card__hint">{{ dashboardMetrics.deployedRevisions }} deployed revisions</span></BaseCard>
              </div>
            </section>

            <BaseCard class="dashboard-proxy-card dashboard-reveal" eyebrow="Workspace activity">
              <div class="dashboard-card-heading"><div><p class="dashboard-eyebrow">Your APIs</p><h2>Recent proxies</h2></div><button type="button" class="dashboard-link" @click="activeView = 'Proxies'">View all <span aria-hidden="true">→</span></button></div>
              <div v-if="proxiesLoading" class="dashboard-proxy-loading" aria-live="polite"><span v-for="item in 3" :key="item" class="dashboard-skeleton-row"><i></i><i></i></span><span class="dashboard-loading-label"><BaseSpinner /> Loading proxies…</span></div>
              <BaseErrorState v-else-if="proxiesError" @retry="retryProxies"><template #title>Proxies unavailable</template><template #hint>{{ proxiesError }}</template></BaseErrorState>
              <BaseEmptyState v-else-if="!selectedEnvironment || !proxyList.length"><template #title>{{ selectedEnvironment ? 'No proxies found' : 'Select an environment' }}</template><template #hint>{{ selectedEnvironment ? 'This organization has no visible proxies.' : 'Choose an organization and environment to load proxies.' }}</template></BaseEmptyState>
              <ul v-else class="proxy-list dashboard-proxy-list">
                <li v-for="proxy in visibleProxies.slice(0, 5)" :key="proxy.name"><button type="button" class="proxy-list__button" @click="openProxy(proxy)"><span><strong>{{ proxy.name }}</strong><small>revision {{ proxy.revisions.at(-1)?.number || '—' }}</small></span><span class="proxy-list__meta"><BaseChip :label="proxy.revisions.some((revision) => revision.status === 'Succeeded') ? 'Deployed' : 'Not deployed'" :tone="proxy.revisions.some((revision) => revision.status === 'Succeeded') ? 'success' : 'neutral'" /><span class="proxy-list__arrow">→</span></span></button></li>
              </ul>
            </BaseCard>
          </template>
        <template v-else-if="activeView === 'Proxies'">
          <BaseCard v-if="proxyCreationMode" eyebrow="Proxy creation">
            <ProxyCreationPreparation
              :templates="templateList"
              :selected-template-name="proxyCreationTemplate?.name || null"
              :open-api-source="proxyCreationOpenApiSource"
              :proxy-name="proxyCreationProxyName"
              :errors="proxyCreationErrors"
              :ready="proxyCreationReady"
              :preview="proxyCreationPreview"
              :logical-target-environment="proxyCreationLogicalTarget"
              :status="proxyCreationStatus"
              :generation="proxyCreationGeneration"
              :created-revision="proxyCreationCreatedRevision"
              :error="proxyCreationError"
              @select-template="selectProxyCreationTemplate"
              @update-open-api-display-name="proxyCreationPreparation.setOpenApiSource({ display_name: $event })"
              @update-open-api-content="proxyCreationPreparation.setOpenApiSource({ content: $event })"
              @update-proxy-name="proxyCreationPreparation.setProxyName($event)"
              @generate="proxyCreationPreparation.generate()"
              @upload="uploadProxyCreation"
              @cancel="cancelProxyCreationPreparation"
            />
          </BaseCard>
          <BaseCard v-else eyebrow="Proxy catalogue">
            <div class="proxy-catalogue__header">
              <div>
                <p class="proxy-catalogue__intro">Manage proxies in the selected Apigee organization and environment.</p>
                <input v-model="proxySearch" class="proxy-search" type="search" placeholder="Search proxies" aria-label="Search proxies" />
              </div>
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
                <button type="button" class="proxy-list__button" @click="openProxy(proxy)">
                  <span>{{ proxy.name }}</span>
                  <BaseChip :label="proxy.revisions.some((revision) => revision.status === 'Succeeded') ? 'Deployed' : 'Not deployed'" :tone="proxy.revisions.some((revision) => revision.status === 'Succeeded') ? 'success' : 'neutral'" />
                </button>
              </li>
            </ul>
          </BaseCard>
          <ProxyDetailsDrawer
            :open="Boolean(selectedProxy)"
            :proxy="selectedProxy"
            :organization="selectedOrganization"
            :environment="selectedEnvironment"
            :selected-revision="selectedRevision"
            :revision-detail="revisionDetail"
            :revision-detail-loading="revisionDetailLoading"
            :revision-detail-error="revisionDetailError"
            @close="closeProxyDrawer"
            @toggle-revision="toggleRevision"
            @retry-revision="loadRevisionDetail"
            @review-deployment="reviewDeploymentRevision"
          />
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
            <ul v-else class="proxy-list template-list">
              <li v-for="template in visibleTemplates" :key="template.name" :class="{ 'template-list__item--selected': currentTemplate?.name === template.name }">
                <button type="button" class="proxy-list__button template-list__select" @click="openTemplateDrawer(template)">
                  <span>{{ template.name || 'Untitled template' }}</span>
                  <span class="template-list__owner">{{ templateOwner(template) }}</span>
                </button>
              </li>
            </ul>
          </BaseCard>
          <TemplateDetailsDrawer
            :open="Boolean(selectedTemplate)"
            :template="selectedTemplate"
            :delete-pending="selectedTemplate ? templateDeletePending === selectedTemplate.name : false"
            @close="closeTemplateDrawer"
            @edit="selectedTemplate && editTemplate(selectedTemplate.name)"
            @prepare-proxy="selectedTemplate && prepareProxyCreation(selectedTemplate.name)"
            @delete="selectedTemplate && deleteTemplate(selectedTemplate.name)"
          />
          </template>
          <template v-if="templateView === 'editor'">
          <TemplateEditorShell :title="metadataDraft.name" :step="editorStep" :next-label="editorStep === 1 ? 'Continue to flow' : editorStep === 2 ? 'Continue to policies' : 'Continue to summary'" :next-disabled="editorStep === 1 ? !metadataValid : editorStep === 2 ? flowValidationErrors.length > 0 : !templateValid" :show-next="editorStep !== 4" @back="editorStep === 1 ? closeTemplateEditor() : previousEditorStep()" @next="editorStep === 4 ? continueToReview() : nextEditorStep()">
            <BaseCard v-if="currentTemplate && editorStep === 1" eyebrow="1 · Details">
            <div class="metadata-form">
              <label for="template-name"><span>Name</span><input id="template-name" :value="metadataDraft.name" :aria-invalid="Boolean(metadataErrors.name)" aria-describedby="template-name-error" @input="updateMetadata('name', ($event.target as HTMLInputElement).value)" /><small id="template-name-error" v-if="metadataErrors.name">{{ metadataErrors.name }}</small></label>
              <label for="template-description"><span>Description</span><textarea id="template-description" :value="metadataDraft.description" rows="2" @input="updateMetadata('description', ($event.target as HTMLTextAreaElement).value)" /></label>
              <label for="template-owner"><span>Owner</span><input id="template-owner" :value="metadataDraft.owner" :aria-invalid="Boolean(metadataErrors.owner)" aria-describedby="template-owner-error" @input="updateMetadata('owner', ($event.target as HTMLInputElement).value)" /><small id="template-owner-error" v-if="metadataErrors.owner">{{ metadataErrors.owner }}</small></label>
              <label for="template-target-environment"><span>Target environment</span><select id="template-target-environment" :value="metadataDraft.target_environment" @change="updateMetadata('target_environment', ($event.target as HTMLSelectElement).value)"><option value="">None</option><option value="dev">dev</option><option value="test">test</option><option value="prod">prod</option></select></label>
              <label for="template-prefix"><span>Name prefix</span><input id="template-prefix" :value="metadataDraft.naming_convention.prefix" :aria-invalid="Boolean(metadataErrors.prefix)" aria-describedby="template-prefix-error" @input="updatePrefix(($event.target as HTMLInputElement).value)" /><small id="template-prefix-error" v-if="metadataErrors.prefix">{{ metadataErrors.prefix }}</small></label>
              <label for="template-name-case"><span>Name case</span><select id="template-name-case" :value="metadataDraft.naming_convention.case" @change="updateNamingCase(($event.target as HTMLSelectElement).value)"><option value="kebab-case">kebab-case</option><option value="snake_case">snake_case</option><option value="camelCase">camelCase</option></select></label>
            </div>
          </BaseCard>
          <BaseCard v-if="currentTemplate && (editorStep === 2 || editorStep === 3)" :eyebrow="editorStep === 2 ? '2 · Flow' : '3 · Policies'">
            <template v-if="editorStep === 2">
            <p class="editor-section__intro">Choose the flow stage where policies will run.</p>
            <FlowDiagram
              :flow="flowDraft"
              :selected-flow="selectedFlow"
              @select-stage="selectedFlow = $event"
              @update-condition="updateConditionalCondition"
              @remove-condition="removeConditionalFlow"
              @add-condition="addConditionalFlow"
            />
            </template>
            <template v-else>
            <p class="editor-section__intro">Add policies to the selected request or response lane.</p>
            <div class="policy-editor">
              <div class="policy-editor__toolbar">
                <div class="policy-lanes"><button type="button" :class="{ 'policy-lane--active': selectedLane === 'request' }" @click="selectedLane = 'request'">Request ({{ selectedStage.request?.length || 0 }})</button><button type="button" :class="{ 'policy-lane--active': selectedLane === 'response' }" @click="selectedLane = 'response'">Response ({{ selectedStage.response?.length || 0 }})</button></div>
                <div class="policy-add"><select v-model="selectedPolicyType" aria-label="Policy type"><option v-for="[value, label] in policyTypes" :key="value" :value="value">{{ label }}</option></select><button type="button" @click="addPolicy">Add policy</button></div>
              </div>
              <ol class="policy-list">
                <li v-for="(policy, index) in selectedPolicies" :key="`${policy.type}-${index}`" class="policy-item">
                  <div class="policy-item__header"><span class="policy-item__identity"><svg class="policy-item__icon" viewBox="0 0 24 24" aria-hidden="true"><path :d="policyIconPath(policy)" /></svg><strong>{{ policyLabel(policy) }}</strong></span><div><button type="button" :aria-label="`Move policy ${index + 1} up`" :disabled="index === 0" @click="movePolicy(index, -1)">↑</button><button type="button" :aria-label="`Move policy ${index + 1} down`" :disabled="index === selectedPolicies.length - 1" @click="movePolicy(index, 1)">↓</button><button type="button" :aria-label="`Remove policy ${index + 1}`" @click="removePolicy(index)">Remove</button></div></div>
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
            </template>
          </BaseCard>
          <BaseCard v-if="currentTemplate && editorStep === 4" eyebrow="4 · Ready to save">
            <div v-if="templateValid && !currentTemplateValidationErrors.length" class="wizard-ready-state"><div class="wizard-ready-state__icon">✓</div><h2>Congratulations, your template is ready.</h2><p>All details and policies are valid. Review the summary before saving this template locally.</p><button type="button" class="primary-action" @click="continueToReview">Continue to review</button></div>
            <div v-else class="wizard-error-state" role="alert"><div class="wizard-error-state__icon">!</div><h2>Something needs your attention.</h2><p>Fix the validation errors below before continuing to review.</p></div>
            <div class="wizard-navigation"><button type="button" @click="previousEditorStep">Back to policies</button></div>
          </BaseCard>
          <div v-if="currentTemplateValidationErrors.length || policyValidationErrors.length" class="template-validation-errors" role="alert" aria-live="assertive">
            <strong>Template validation</strong>
            <button v-for="validationError in currentTemplateValidationErrors" :key="`${validationError.code}-${validationError.field}`" type="button">{{ validationError.field || 'Template' }}: {{ validationError.message }}</button>
            <button v-for="policyError in policyValidationErrors" :key="`${policyError.field}-${policyError.message}`" type="button">{{ policyError.field }}: {{ policyError.message }}</button>
          </div>
          </TemplateEditorShell>
          </template>

          <template v-if="templateView === 'review'">
            <BaseCard eyebrow="Review and save">
              <div class="review-header"><div><h2>Ready to save</h2><p>Check the template summary before writing it to local storage.</p></div><BaseChip :label="currentTemplateDirty ? 'Unsaved changes' : 'Saved'" :tone="currentTemplateDirty ? 'warning' : 'success'" /></div>
              <div class="review-grid"><div><span>Name</span><strong>{{ metadataDraft.name || 'Missing' }}</strong></div><div><span>Owner</span><strong>{{ metadataDraft.owner || 'Missing' }}</strong></div><div><span>Target</span><strong>{{ metadataDraft.target_environment || 'None' }}</strong></div><div><span>Policies</span><strong>{{ totalPolicyCount }}</strong></div></div>
              <div class="review-actions"><button type="button" @click="templateView = 'editor'">Back to editor</button><button type="button" class="primary-action" :disabled="!templateValid || !currentTemplateDirty || currentTemplateStatus === 'saving'" @click="saveTemplate">{{ currentTemplateStatus === 'saving' ? 'Saving…' : 'Save & finish' }}</button></div>
            </BaseCard>
          </template>
          </template>
        </template>
        <BaseModal v-if="modal" :open="true" :title="modal.title" :message="modal.message" :confirm-label="modal.confirmLabel" :tone="modal.tone" @close="modal = null" @confirm="modalAction?.()" />
      </main>
    </div>
  </div>
</template>
