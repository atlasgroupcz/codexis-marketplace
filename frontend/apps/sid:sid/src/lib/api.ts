import { actionResponseSchema, statusResponseSchema } from './schemas'
import type { ActionResponse, StatusResponse } from './schemas'

/**
 * In dev the Vite proxy maps /api to the component endpoint; in the VM the
 * panel is served from the component directory, so `./api` lands on the CGI
 * binary (any path that is not a file falls through to it).
 */
function getApiBase(): string {
  if (import.meta.env.DEV) {
    return '/api/'
  }
  return './api'
}

async function post(body: Record<string, unknown>): Promise<ActionResponse> {
  const response = await fetch(new URL(getApiBase(), window.location.href), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    cache: 'no-store',
    body: JSON.stringify(body),
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  return actionResponseSchema.parse(await response.json())
}

export async function fetchStatus(): Promise<StatusResponse> {
  const response = await fetch(new URL(getApiBase(), window.location.href), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    cache: 'no-store',
    body: JSON.stringify({ action: 'get_status' }),
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  return statusResponseSchema.parse(await response.json())
}

export function saveApiKey(apiKey: string): Promise<ActionResponse> {
  return post({ action: 'save_config', api_key: apiKey })
}

export function deleteApiKey(): Promise<ActionResponse> {
  return post({ action: 'delete_config' })
}

export function saveNotifySettings(values: {
  email: boolean
  inApp: boolean
  recipients: Array<string>
}): Promise<ActionResponse> {
  return post({ action: 'save_notify_settings', ...values })
}
