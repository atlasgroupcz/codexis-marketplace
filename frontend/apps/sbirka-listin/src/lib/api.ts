import { createApiClient } from '@workspace/ui/lib/api-client'
import type {
  ActionResponse,
  CheckAllResponse,
  DetailResponse,
  DownloadResponse,
  NotifyResponse,
  NotifySettingsValues,
  OverviewResponse,
} from './schemas'
import {
  actionResponseSchema,
  checkAllResponseSchema,
  detailResponseSchema,
  downloadResponseSchema,
  notifyResponseSchema,
  overviewResponseSchema,
} from './schemas'

// Shared client: DEV → `/api/` (Vite-proxied), prod → `./api` (component route).
// Both GET and POST surface the CGI's `{ error }` body, so a rejected promise
// carries the daemon's message (e.g. "IČO už je sledované").
const { getJson, postJson } = createApiClient()

// ─── reads ─────────────────────────────────────────────────────────────────

export function fetchOverview(): Promise<OverviewResponse> {
  return getJson(overviewResponseSchema)
}

export function fetchDetail(ico: string): Promise<DetailResponse> {
  return getJson(detailResponseSchema, { ico })
}

export function fetchNotify(): Promise<NotifyResponse> {
  return getJson(notifyResponseSchema, { view: 'notify' })
}

// ─── writes ────────────────────────────────────────────────────────────────

export function addSubjekt(ico: string): Promise<ActionResponse> {
  return postJson('add', actionResponseSchema, { ico })
}

export function removeSubjekt(ico: string): Promise<ActionResponse> {
  return postJson('remove', actionResponseSchema, { ico })
}

export function checkAll(): Promise<CheckAllResponse> {
  return postJson('check_all', checkAllResponseSchema)
}

export function confirmChange(ico: string, changeIndex?: number): Promise<ActionResponse> {
  const payload: Record<string, unknown> = { ico }
  if (changeIndex !== undefined) payload.change_index = changeIndex
  return postJson('confirm_change', actionResponseSchema, payload)
}

export function downloadDokument(ico: string, cislo: string): Promise<DownloadResponse> {
  return postJson('download', downloadResponseSchema, { ico, cislo })
}

export function saveNotify(values: NotifySettingsValues): Promise<ActionResponse> {
  return postJson('save_notify_settings', actionResponseSchema, { ...values })
}
