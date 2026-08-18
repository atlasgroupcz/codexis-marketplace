import type { z } from 'zod'
import type {
  ActionResponse,
  CheckAllResponse,
  DetailResponse,
  DphResponse,
  NotifyResponse,
  NotifyValues,
  OverviewResponse,
  SaveNotifyResponse,
} from './schemas'
import {
  actionResponseSchema,
  checkAllResponseSchema,
  detailResponseSchema,
  dphResponseSchema,
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

/** VAT-payer reliability + published accounts, by IČO/DIČ (MF registry). */
export function fetchDph(dic: string): Promise<DphResponse> {
  return getJson(dphResponseSchema, { dph: dic })
}

export function fetchNotify(): Promise<NotifyResponse> {
  return getJson(notifyResponseSchema, { view: 'notify' })
}

// ─── writes ────────────────────────────────────────────────────────────────

export function addCompany(ico: string, label = ''): Promise<ActionResponse> {
  return postJson('add_company', actionResponseSchema, { ico, label })
}

export function addPerson(
  prijmeni: string,
  jmeno: string,
  datum_narozeni: string,
  label = '',
): Promise<ActionResponse> {
  return postJson('add_person', actionResponseSchema, {
    prijmeni,
    jmeno,
    datum_narozeni,
    label,
  })
}

export function checkAll(): Promise<CheckAllResponse> {
  return postJson('check_all', checkAllResponseSchema)
}

export function checkSubject(uuid: string): Promise<ActionResponse> {
  return postJson('check_subject', actionResponseSchema, { uuid })
}

export function removeSubject(uuid: string): Promise<ActionResponse> {
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

export function saveNotify(values: NotifyValues): Promise<SaveNotifyResponse> {
  return postJson('save_notify_settings', saveNotifyResponseSchema, { ...values })
}
