import type {
  ActionResponse,
  AvailableProjectsResponse,
  DetailResponse,
  NotifyResponse,
  NotifySettings,
  OverviewResponse,
  ProjectResponse,
  ProjectsResponse,
} from './schemas'
import {
  actionResponseSchema,
  availableProjectsResponseSchema,
  detailResponseSchema,
  notifyResponseSchema,
  overviewResponseSchema,
  projectResponseSchema,
  projectsResponseSchema,
} from './schemas'

function getApiBase(): string {
  if (import.meta.env.DEV) {
    return '/api/'
  }
  return './api'
}

export async function fetchOverview(): Promise<OverviewResponse> {
  const url = new URL(getApiBase(), window.location.href).toString()
  const response = await fetch(url, {
    headers: { Accept: 'application/json' },
    cache: 'no-store',
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  const json: unknown = await response.json()
  return overviewResponseSchema.parse(json)
}

export async function fetchDetail(uuid: string): Promise<DetailResponse> {
  const url = new URL(getApiBase(), window.location.href)
  url.searchParams.set('uuid', uuid)
  const response = await fetch(url.toString(), {
    headers: { Accept: 'application/json' },
    cache: 'no-store',
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  const json: unknown = await response.json()
  return detailResponseSchema.parse(json)
}

export async function postAction(
  action: string,
  uuid: string,
  changeIndex?: number,
): Promise<ActionResponse> {
  const url = new URL(getApiBase(), window.location.href).toString()
  const body: Record<string, unknown> = { action, uuid }
  if (changeIndex !== undefined) {
    body.changeIndex = changeIndex
  }
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Accept: 'application/json',
    },
    body: JSON.stringify(body),
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  const json: unknown = await response.json()
  return actionResponseSchema.parse(json)
}

export async function postNoteAction(
  action: 'note_add' | 'note_remove',
  uuid: string,
  payload: Record<string, unknown>,
): Promise<ActionResponse> {
  const url = new URL(getApiBase(), window.location.href).toString()
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Accept: 'application/json',
    },
    body: JSON.stringify({ action, uuid, ...payload }),
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  const json: unknown = await response.json()
  return actionResponseSchema.parse(json)
}

export async function postGroupAction(
  action: 'group_add' | 'group_remove' | 'group_delete' | 'group_rename',
  payload: Record<string, unknown>,
): Promise<ActionResponse> {
  const url = new URL(getApiBase(), window.location.href).toString()
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Accept: 'application/json',
    },
    body: JSON.stringify({ action, ...payload }),
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  const json: unknown = await response.json()
  return actionResponseSchema.parse(json)
}

// ── watched projects ────────────────────────────────────────────────────────

function withParams(params: Record<string, string>): string {
  const url = new URL(getApiBase(), window.location.href)
  for (const [k, v] of Object.entries(params)) {
    url.searchParams.set(k, v)
  }
  return url.toString()
}

async function getJson(url: string): Promise<unknown> {
  const response = await fetch(url, {
    headers: { Accept: 'application/json' },
    cache: 'no-store',
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  return response.json()
}

export async function fetchProjects(): Promise<ProjectsResponse> {
  return projectsResponseSchema.parse(await getJson(withParams({ view: 'projects' })))
}

export async function fetchNotify(): Promise<NotifyResponse> {
  return notifyResponseSchema.parse(await getJson(withParams({ view: 'notify' })))
}

export async function saveNotify(values: NotifySettings): Promise<ActionResponse> {
  return postProjectAction('save_notify_settings', values)
}

export async function fetchProject(project: string): Promise<ProjectResponse> {
  return projectResponseSchema.parse(await getJson(withParams({ view: 'project', project })))
}

export async function fetchAvailableProjects(): Promise<AvailableProjectsResponse> {
  return availableProjectsResponseSchema.parse(await getJson(withParams({ view: 'available' })))
}

export type ProjectAction =
  | 'project_add'
  | 'project_remove'
  | 'project_refresh'
  | 'project_check'
  | 'project_confirm'
  | 'start_extraction'
  | 'save_notify_settings'

export async function postProjectAction(
  action: ProjectAction,
  payload: Record<string, unknown>,
): Promise<ActionResponse> {
  const url = new URL(getApiBase(), window.location.href).toString()
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Accept: 'application/json',
    },
    body: JSON.stringify({ action, ...payload }),
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  return actionResponseSchema.parse(await response.json())
}
