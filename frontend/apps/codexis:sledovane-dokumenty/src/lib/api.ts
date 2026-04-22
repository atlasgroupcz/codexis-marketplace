import { actionResponseSchema, detailResponseSchema, overviewResponseSchema } from './schemas'
import type { ActionResponse, DetailResponse, OverviewResponse } from './schemas'

function getApiBase(): string {
  if (import.meta.env.DEV) {
    return '/api/'
  }
  return './'
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
