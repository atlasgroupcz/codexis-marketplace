import { createApiClient } from '@workspace/ui/lib/api-client'
import { z } from 'zod'

export const settingsSchema = z.object({
  configured: z.boolean(),
  maskedKey: z.string(),
  keySource: z.string(),
  settingsFile: z.string(),
  apiUrl: z.string(),
  generateUrl: z.string(),
})
export type Settings = z.infer<typeof settingsSchema>

const actionSchema = z.object({
  ok: z.boolean(),
  error: z.string().optional(),
})

const { getJson, postJson } = createApiClient()

export const fetchSettings = (): Promise<Settings> => getJson(settingsSchema)

export const saveApiKey = (apiKey: string): Promise<unknown> =>
  postJson('save_api_key', actionSchema, { apiKey })

export const deleteApiKey = (): Promise<unknown> => postJson('delete_api_key', actionSchema)
