import { computed, ref } from 'vue'
import type { DeploymentJobInputDto, DeploymentPreviewDto, OpenApiSourceDto, TemplateDto } from '../types/bridge'

export type DeploymentPreparationField = 'template' | 'spec' | 'organization' | 'environment' | 'proxy'
export type DeploymentPreparationErrors = Partial<Record<DeploymentPreparationField, string>>

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function cloneTemplate(template: TemplateDto | null): TemplateDto | null {
  return template ? JSON.parse(JSON.stringify(template)) as TemplateDto : null
}

function templateMetadata(template: TemplateDto | null): Record<string, unknown> | null {
  const metadata = template?.data.metadata
  return isRecord(metadata) ? metadata : null
}

function templateName(template: TemplateDto | null): string {
  const metadataName = templateMetadata(template)?.name
  return typeof metadataName === 'string' ? metadataName.trim() : ''
}

function namingConvention(template: TemplateDto | null): { prefix: string; case: string } {
  const value = templateMetadata(template)?.naming_convention
  if (!isRecord(value)) return { prefix: '', case: 'kebab-case' }
  return {
    prefix: typeof value.prefix === 'string' ? value.prefix.trim() : '',
    case: typeof value.case === 'string' ? value.case : 'kebab-case',
  }
}

function normalizeProxyName(value: string, namingCase: string): string {
  const words = value
    .trim()
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .split(/[^a-zA-Z0-9]+/)
    .filter(Boolean)
    .map((word) => word.toLowerCase())
  if (namingCase === 'snake_case') return words.join('_')
  if (namingCase === 'camelCase') return words.map((word, index) => index === 0 ? word : `${word[0].toUpperCase()}${word.slice(1)}`).join('')
  return words.join('-')
}

export function deriveProxyName(template: TemplateDto | null): string {
  const metadata = namingConvention(template)
  return normalizeProxyName(`${metadata.prefix} ${templateName(template)}`, metadata.case)
}

function hasOpenApiDocument(content: string): boolean {
  const trimmed = content.trim()
  if (!trimmed) return false
  try {
    const parsed = JSON.parse(trimmed) as unknown
    return isRecord(parsed) && typeof parsed.openapi === 'string' && parsed.openapi.trim().length > 0
  } catch {
    return /^\s*openapi\s*:\s*['"]?\d+\.\d+/m.test(content)
  }
}

function countPolicies(template: TemplateDto | null): number {
  const flow = template?.data.flow
  if (!isRecord(flow)) return 0
  const stages = [flow.pre_flow, flow.post_flow, ...(Array.isArray(flow.conditional_flows) ? flow.conditional_flows : [])]
  return stages.reduce((total, stage) => {
    if (!isRecord(stage)) return total
    const request = Array.isArray(stage.request) ? stage.request.length : 0
    const response = Array.isArray(stage.response) ? stage.response.length : 0
    return total + request + response
  }, 0)
}

function validateTemplate(template: TemplateDto | null): string | undefined {
  if (!template) return 'Select a template before preparing a deployment.'
  if (!templateName(template)) return 'The selected template must have a name.'
  const metadata = templateMetadata(template)
  if (!metadata || typeof metadata.owner !== 'string' || !metadata.owner.trim()) return 'The selected template is missing a valid owner.'
  const flow = template.data.flow
  if (!isRecord(flow)) return 'The selected template has no valid flow.'
  if (!isRecord(flow.pre_flow) || !isRecord(flow.post_flow)) return 'The selected template has incomplete flow stages.'
  return undefined
}

export function useDeploymentPreparation() {
  const selectedTemplate = ref<TemplateDto | null>(null)
  const openApiSource = ref<OpenApiSourceDto>({ display_name: '', content: '' })
  const organization = ref('')
  const environment = ref('')
  const proxyName = ref('')

  const errors = computed<DeploymentPreparationErrors>(() => {
    const next: DeploymentPreparationErrors = {}
    const templateError = validateTemplate(selectedTemplate.value)
    if (templateError) next.template = templateError
    if (!openApiSource.value.display_name.trim()) next.spec = 'Provide a name for the OpenAPI specification.'
    else if (!hasOpenApiDocument(openApiSource.value.content)) next.spec = 'The specification must contain a valid OpenAPI document.'
    if (!organization.value.trim()) next.organization = 'Select an Apigee organization.'
    if (!environment.value.trim()) next.environment = 'Select an Apigee environment.'
    if (!proxyName.value.trim()) next.proxy = 'A proxy name is required.'
    return next
  })

  const ready = computed(() => Object.keys(errors.value).length === 0)
  const logicalTargetEnvironment = computed(() => {
    const target = templateMetadata(selectedTemplate.value)?.target_environment
    return typeof target === 'string' && target.trim() ? target.trim() : null
  })
  const preview = computed<DeploymentPreviewDto | null>(() => {
    if (!selectedTemplate.value) return null
    return {
      template_name: templateName(selectedTemplate.value),
      spec_display_name: openApiSource.value.display_name.trim(),
      organization: organization.value.trim(),
      environment: environment.value.trim(),
      proxy_name: proxyName.value.trim(),
      logical_target_environment: logicalTargetEnvironment.value,
      logical_target_matches: logicalTargetEnvironment.value
        ? logicalTargetEnvironment.value === environment.value.trim()
        : null,
      policy_count: countPolicies(selectedTemplate.value),
    }
  })

  function selectTemplate(template: TemplateDto) {
    selectedTemplate.value = cloneTemplate(template)
    proxyName.value = deriveProxyName(template)
  }

  function setOpenApiSource(source: Partial<OpenApiSourceDto>) {
    openApiSource.value = { ...openApiSource.value, ...source }
  }

  function setContext(nextOrganization: string, nextEnvironment: string) {
    organization.value = nextOrganization
    environment.value = nextEnvironment
  }

  function clear() {
    selectedTemplate.value = null
    openApiSource.value = { display_name: '', content: '' }
    organization.value = ''
    environment.value = ''
    proxyName.value = ''
  }

  function jobInput(): DeploymentJobInputDto | null {
    if (!ready.value || !selectedTemplate.value) return null
    return {
      template_name: templateName(selectedTemplate.value),
      openapi_source: { ...openApiSource.value },
      organization: organization.value.trim(),
      environment: environment.value.trim(),
      proxy_name: proxyName.value.trim(),
      override_existing: false,
    }
  }

  return {
    selectedTemplate,
    openApiSource,
    organization,
    environment,
    proxyName,
    logicalTargetEnvironment,
    errors,
    ready,
    preview,
    selectTemplate,
    setOpenApiSource,
    setContext,
    clear,
    jobInput,
  }
}
