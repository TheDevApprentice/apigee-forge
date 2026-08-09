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

export type AuthDto = {
  authenticated: boolean
  mode: 'desktop' | 'headless' | null
  identity: string | null
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

export type GuiCommandError = {
  code: string
  message: string
}
