import type { z } from 'zod'
import type {
  ActionResponse,
  CheckAllResponse,
  ConfigResponse,
  DetailResponse,
  NotifyResponse,
  NotifySettingsValues,
  OverviewResponse,
  SaveNotifyResponse,
} from './schemas'
import {
  actionResponseSchema,
  checkAllResponseSchema,
  configResponseSchema,
  detailResponseSchema,
  notifyResponseSchema,
  overviewResponseSchema,
  saveNotifyResponseSchema,
} from './schemas'

function getApiBase(): string {
  if (import.meta.env.DEV) {
    return '/api/'
  }
  return './api'
}

function buildUrl(params?: Record<string, string>): string {
  const url = new URL(getApiBase(), window.location.href)
  if (params) {
    for (const [k, v] of Object.entries(params)) url.searchParams.set(k, v)
  }
  return url.toString()
}

async function getJson<T>(schema: z.ZodType<T>, params?: Record<string, string>): Promise<T> {
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
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
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

// ─── writes ──────────────────────────────────────────────────────────────────

export interface AddTextInput {
  text: string
  nice_classes: Array<number>
  territories: Array<string>
  owner_name?: string
  label?: string
}

export function addText(input: AddTextInput): Promise<ActionResponse> {
  return postJson('add_text', actionResponseSchema, {
    owner_name: '',
    label: '',
    ...input,
  })
}

export interface AddLogoInput {
  logo_base64: string
  mime: string
  text?: string
  nice_classes: Array<number>
  territories: Array<string>
  vienna_codes?: Array<string>
  owner_name?: string
  label?: string
}

export function addLogo(input: AddLogoInput): Promise<ActionResponse> {
  return postJson('add_logo', actionResponseSchema, {
    text: '',
    vienna_codes: [],
    owner_name: '',
    label: '',
    ...input,
  })
}

export function checkAll(): Promise<CheckAllResponse> {
  return postJson('check_all', checkAllResponseSchema)
}

export function checkSubject(uuid: string): Promise<ActionResponse> {
  return postJson('check_subject', actionResponseSchema, { uuid })
}

export function removeMark(uuid: string): Promise<ActionResponse> {
  return postJson('remove', actionResponseSchema, { uuid })
}

export function confirmChange(uuid: string, changeIndex?: number): Promise<ActionResponse> {
  const payload: Record<string, unknown> = { uuid }
  if (changeIndex !== undefined) payload.change_index = changeIndex
  return postJson('confirm_change', actionResponseSchema, payload)
}

export function setLabel(uuid: string, label: string): Promise<ActionResponse> {
  return postJson('set_label', actionResponseSchema, { uuid, label })
}

export interface UpdateMarkInput {
  text?: string
  nice_classes?: Array<number>
  territories?: Array<string>
  owner_name?: string
  label?: string
  threshold?: number
}

export function updateMark(uuid: string, fields: UpdateMarkInput): Promise<ActionResponse> {
  return postJson('update_mark', actionResponseSchema, { uuid, ...fields })
}

// ─── EUIPO credentials config ─────────────────────────────────────────────────

export function fetchConfig(): Promise<ConfigResponse> {
  return getJson(configResponseSchema, { settings: '1' })
}

export function saveConfig(clientId: string, clientSecret: string): Promise<ActionResponse> {
  return postJson('save_config', actionResponseSchema, {
    client_id: clientId,
    client_secret: clientSecret,
  })
}

export function deleteConfig(): Promise<ActionResponse> {
  return postJson('delete_config', actionResponseSchema)
}

// ─── notification settings ────────────────────────────────────────────────────

export function fetchNotify(): Promise<NotifyResponse> {
  return getJson(notifyResponseSchema, { view: 'notify' })
}

export function saveNotify(values: NotifySettingsValues): Promise<SaveNotifyResponse> {
  return postJson('save_notify_settings', saveNotifyResponseSchema, values)
}
