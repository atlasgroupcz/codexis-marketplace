import type {
  ActionResponse,
  DetailResponse,
  NotifySettingsValues,
  OverviewResponse,
  ReportResponse,
} from './schemas'
import {
  actionResponseSchema,
  detailResponseSchema,
  notifyResponseSchema,
  overviewResponseSchema,
  reportResponseSchema,
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

export async function fetchReport(uuid: string, reportId: string): Promise<ReportResponse> {
  const url = new URL(getApiBase(), window.location.href)
  url.searchParams.set('uuid', uuid)
  url.searchParams.set('report', reportId)
  const response = await fetch(url.toString(), {
    headers: { Accept: 'application/json' },
    cache: 'no-store',
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  const json: unknown = await response.json()
  return reportResponseSchema.parse(json)
}

export async function fetchNotify(): Promise<{
  settings: NotifySettingsValues
  userEmail: string
}> {
  const url = new URL(getApiBase(), window.location.href)
  url.searchParams.set('view', 'notify')
  const response = await fetch(url.toString(), {
    headers: { Accept: 'application/json' },
    cache: 'no-store',
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  const json: unknown = await response.json()
  const parsed = notifyResponseSchema.parse(json)
  return { settings: parsed.settings, userEmail: parsed.userEmail }
}

export async function saveNotify(values: NotifySettingsValues): Promise<ActionResponse> {
  return postAction({
    action: 'save_notify_settings',
    email: values.email,
    inApp: values.inApp,
    recipients: values.recipients,
  })
}

export async function postAction(body: Record<string, unknown>): Promise<ActionResponse> {
  const url = new URL(getApiBase(), window.location.href).toString()
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
