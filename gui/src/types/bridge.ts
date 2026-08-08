export type AuthDto = {
  authenticated: boolean
  mode: 'desktop' | 'headless' | null
  identity: string | null
  project_id: string | null
  selected_organization: string | null
}

export type OrganizationDto = {
  id: string
  project_id: string
  location: string | null
}

export type EnvironmentDto = {
  name: string
}

export type ProxyRevisionDto = {
  number: number
  deployed: boolean
}

export type ProxyDto = {
  name: string
  revisions: ProxyRevisionDto[]
}

export type GuiCommandError = {
  code: string
  message: string
}
