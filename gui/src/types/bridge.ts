export type AppMode = 'demo' | 'cloud'

export type SessionStatus = 'authentication_required' | 'organization_required' | 'ready' | 'error'

export type SessionDto = {
  mode: AppMode
  status: SessionStatus
  identity: string | null
  organization: string | null
  environment: string | null
  error: string | null
}

export type AuthStorageDto = {
  refresh_token_stored: boolean
}

export type AuthDto = {
  authenticated: boolean
  mode: 'desktop' | 'headless' | null
  identity: string | null
  given_name: string | null
  family_name: string | null
  name: string | null
  picture: string | null
  project_id: string | null
  selected_organization: string | null
}

export type OrganizationDto = {
  source: AppMode
  id: string
  project_id: string
  location: string | null
}

export type EnvironmentDto = {
  source: AppMode
  name: string
}

export type ProxyRevisionDto = {
  number: number
  deployed: boolean
  status: 'Pending' | 'InProgress' | 'Succeeded' | 'Failed' | 'NotDeployed'
}

export type ProxyDto = {
  source: AppMode
  name: string
  revisions: ProxyRevisionDto[]
}

export type RoleDto = {
  name: string
  source: AppMode
}

export type RevisionDetailDto = {
  source: AppMode
  organization: string
  proxy_name: string
  revision: number
  data: Record<string, unknown>
}

export type TemplateDto = {
  name: string
  data: Record<string, unknown>
}

export type TemplateValidationErrorDto = {
  code: string
  message: string
  field: string | null
}

export type GuiCommandError = {
  code: string
  message: string
}

export type OpenApiSourceDto = {
  display_name: string
  content: string
}

export type DeploymentJobInputDto = {
  template_name: string
  openapi_source: OpenApiSourceDto
  organization: string
  environment: string
  proxy_name: string
  override_existing: boolean
}

export type DeploymentPreviewDto = {
  template_name: string
  spec_display_name: string
  organization: string
  environment: string
  proxy_name: string
  logical_target_environment: string | null
  logical_target_matches: boolean | null
  policy_count: number
}
