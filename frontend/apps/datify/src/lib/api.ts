import { createApiClient } from '@workspace/ui/lib/api-client'
import { z } from 'zod'

/**
 * What the settings CGI reports. The token itself is never returned - only a
 * mask and which layer supplied it, so the page can tell stored state from a
 * host-injected override the user cannot edit here.
 */
export const settingsSchema = z.object({
  configured: z.boolean(),
  maskedKey: z.string(),
  /** An env var name, or the path of the settings file. */
  keySource: z.string(),
  settingsFile: z.string(),
  generateUrl: z.string(),
})
export type Settings = z.infer<typeof settingsSchema>

const actionSchema = z.object({
  ok: z.boolean(),
  error: z.string().optional(),
})

// Shared client: the same DEV/prod base and the same "reject on { ok: false }"
// contract a local copy would have, plus it surfaces the server's `error` field
// on a non-2xx. That matters here - the CGI answers a rejected or missing token
// with a message naming what to do about it, which a bare "HTTP 412" throws away.
const { getJson, postJson } = createApiClient()

export const fetchSettings = (): Promise<Settings> => getJson(settingsSchema)

/**
 * Rejects with the reason Datify gave. The CGI verifies the token before storing
 * it, so a rejection here means nothing was written.
 */
export const saveToken = (apiKey: string): Promise<unknown> =>
  postJson('save_api_key', actionSchema, { apiKey })

export const deleteToken = (): Promise<unknown> => postJson('delete_api_key', actionSchema)
