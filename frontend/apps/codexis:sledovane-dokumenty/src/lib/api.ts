import {
  actionResponseSchema,
  browseResponseSchema,
  detailResponseSchema,
  folderResponseSchema,
  foldersResponseSchema,
  notifyResponseSchema,
  overviewResponseSchema,
} from './schemas'
import type {
  ActionResponse,
  BrowseResponse,
  DetailResponse,
  FolderResponse,
  FoldersResponse,
  NotifyResponse,
  NotifySettings,
  OverviewResponse,
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

export async function postAction(action: string, uuid: string, changeIndex?: number): Promise<ActionResponse> {
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

// ── watched folders ─────────────────────────────────────────────────────────

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

export async function fetchFolders(): Promise<FoldersResponse> {
  return foldersResponseSchema.parse(await getJson(withParams({ view: 'folders' })))
}

export async function fetchNotify(): Promise<NotifyResponse> {
  return notifyResponseSchema.parse(await getJson(withParams({ view: 'notify' })))
}

export async function saveNotify(values: NotifySettings): Promise<ActionResponse> {
  return postFolderAction('save_notify_settings', values)
}

export async function fetchFolder(root: string): Promise<FolderResponse> {
  return folderResponseSchema.parse(await getJson(withParams({ view: 'folder', root })))
}

export async function browseFolders(path?: string): Promise<BrowseResponse> {
  const params: Record<string, string> = { view: 'browse' }
  if (path) {
    params.path = path
  }
  return browseResponseSchema.parse(await getJson(withParams(params)))
}

export type FolderAction =
  | 'folder_add'
  | 'folder_remove'
  | 'folder_refresh'
  | 'folder_check'
  | 'folder_confirm'
  | 'start_extraction'
  | 'save_notify_settings'

export async function postFolderAction(
  action: FolderAction,
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
