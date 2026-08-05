import { actionResponseSchema, statusResponseSchema } from './schemas'
import type { ActionResponse, NotifySettings, StatusResponse } from './schemas'
import type { z } from 'zod'

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

/** Every call is a POST carrying an `action`; only the response shape differs. */
async function request<T>(schema: z.ZodType<T>, body: Record<string, unknown>): Promise<T> {
  const response = await fetch(new URL(getApiBase(), window.location.href), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    cache: 'no-store',
    body: JSON.stringify(body),
  })
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  return schema.parse(await response.json())
}

export function fetchStatus(): Promise<StatusResponse> {
  return request(statusResponseSchema, { action: 'get_status' })
}

export function saveApiKey(apiKey: string): Promise<ActionResponse> {
  return request(actionResponseSchema, { action: 'save_config', api_key: apiKey })
}

export function deleteApiKey(): Promise<ActionResponse> {
  return request(actionResponseSchema, { action: 'delete_config' })
}

export function saveNotifySettings(values: NotifySettings): Promise<ActionResponse> {
  return request(actionResponseSchema, { action: 'save_notify_settings', ...values })
}
