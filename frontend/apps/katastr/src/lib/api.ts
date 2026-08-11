import { createApiClient } from '@workspace/ui/lib/api-client'
import type {
  ActionResponse,
  CheckAllResponse,
  DetailResponse,
  NotifyResponse,
  NotifySettingsValues,
  OverviewResponse,
} from './schemas'
import {
  actionResponseSchema,
  checkAllResponseSchema,
  detailResponseSchema,
  notifyResponseSchema,
  overviewResponseSchema,
} from './schemas'

// Shared client: same DEV/prod base and the same "reject on { ok: false }"
// contract the local copy had, plus it surfaces the server's `error` field on a
// non-2xx GET (the old local getJson threw a bare HTTP status).
const { getJson, postJson } = createApiClient()

// ─── reads ─────────────────────────────────────────────────────────────────

export function fetchOverview(): Promise<OverviewResponse> {
  return getJson(overviewResponseSchema)
}

export function fetchDetail(cisloRizeni: string): Promise<DetailResponse> {
  return getJson(detailResponseSchema, { cislo_rizeni: cisloRizeni })
}

export function fetchNotify(): Promise<NotifyResponse> {
  return getJson(notifyResponseSchema, { view: 'notify' })
}

// ─── writes ────────────────────────────────────────────────────────────────

export function addProceeding(cislo_rizeni: string, label = ''): Promise<ActionResponse> {
  return postJson('add_proceeding', actionResponseSchema, { cislo_rizeni, label })
}

export function checkAll(): Promise<CheckAllResponse> {
  return postJson('check_all', checkAllResponseSchema)
}

export function removeProceeding(cisloRizeni: string): Promise<ActionResponse> {
  return postJson('remove', actionResponseSchema, { cislo_rizeni: cisloRizeni })
}

export function confirmChange(cisloRizeni: string, changeIndex?: number): Promise<ActionResponse> {
  const payload: Record<string, unknown> = { cislo_rizeni: cisloRizeni }
  if (changeIndex !== undefined) payload.change_index = changeIndex
  return postJson('confirm_change', actionResponseSchema, payload)
}

export function saveApiKey(apiKey: string): Promise<ActionResponse> {
  return postJson('save_api_key', actionResponseSchema, { apiKey })
}

export function deleteApiKey(): Promise<ActionResponse> {
  return postJson('delete_api_key', actionResponseSchema)
}

export function setLabel(cisloRizeni: string, label: string): Promise<ActionResponse> {
  return postJson('set_label', actionResponseSchema, { cislo_rizeni: cisloRizeni, label })
}

export function saveNotify(values: NotifySettingsValues): Promise<ActionResponse> {
  return postJson('save_notify_settings', actionResponseSchema, { ...values })
}
