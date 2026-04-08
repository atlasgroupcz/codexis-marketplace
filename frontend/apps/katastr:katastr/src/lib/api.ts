import type { z } from 'zod'
import {
  actionResponseSchema,
  checkAllResponseSchema,
  detailResponseSchema,
  overviewResponseSchema,
} from './schemas'
import type {
  ActionResponse,
  CheckAllResponse,
  DetailResponse,
  OverviewResponse,
} from './schemas'

function getApiBase(): string {
  if (import.meta.env.DEV) {
    return '/api/'
  }
  return './'
}

function buildUrl(params?: Record<string, string>): string {
  const url = new URL(getApiBase(), window.location.href)
  if (params) {
    for (const [k, v] of Object.entries(params)) url.searchParams.set(k, v)
  }
  return url.toString()
}

async function getJson<T>(
  schema: z.ZodType<T>,
  params?: Record<string, string>,
): Promise<T> {
  const response = await fetch(buildUrl(params), {
    headers: { Accept: 'application/json' },
    cache: 'no-store',
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  return schema.parse(await response.json())
}

async function postJson<T>(
  action: string,
  schema: z.ZodType<T>,
  payload: Record<string, unknown> = {},
): Promise<T> {
  const response = await fetch(buildUrl(), {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Accept: 'application/json',
    },
    body: JSON.stringify({ action, ...payload }),
  })
  const json = (await response.json()) as unknown
  if (!response.ok) {
    const message =
      typeof json === 'object' && json !== null && 'error' in json
        ? String((json as { error: unknown }).error)
        : `HTTP ${response.status}: ${response.statusText}`
    throw new Error(message)
  }
  return schema.parse(json)
}

// ─── reads ─────────────────────────────────────────────────────────────────

export function fetchOverview(): Promise<OverviewResponse> {
  return getJson(overviewResponseSchema)
}

export function fetchDetail(uuid: string): Promise<DetailResponse> {
  return getJson(detailResponseSchema, { uuid })
}

// ─── writes ────────────────────────────────────────────────────────────────

export function addProceeding(
  cislo_rizeni: string,
  label = '',
): Promise<ActionResponse> {
  return postJson('add_proceeding', actionResponseSchema, { cislo_rizeni, label })
}

export function checkAll(): Promise<CheckAllResponse> {
  return postJson('check_all', checkAllResponseSchema)
}

export function removeProceeding(uuid: string): Promise<ActionResponse> {
  return postJson('remove', actionResponseSchema, { uuid })
}

export function confirmChange(
  uuid: string,
  changeIndex?: number,
): Promise<ActionResponse> {
  const payload: Record<string, unknown> = { uuid }
  if (changeIndex !== undefined) payload.change_index = changeIndex
  return postJson('confirm_change', actionResponseSchema, payload)
}

export function saveApiKey(apiKey: string): Promise<ActionResponse> {
  return postJson('save_api_key', actionResponseSchema, { apiKey })
}

export function deleteApiKey(): Promise<ActionResponse> {
  return postJson('delete_api_key', actionResponseSchema)
}

export function setLabel(uuid: string, label: string): Promise<ActionResponse> {
  return postJson('set_label', actionResponseSchema, { uuid, label })
}
