import { z } from 'zod'

/**
 * The bell is the only channel. SID mails its own users (per-event
 * templates, a link to the document, per-tenant gating), so the plugin
 * deliberately has no e-mail switch to offer.
 */
export const notifySettingsSchema = z.object({
  inApp: z.boolean(),
})

export const statusResponseSchema = z.object({
  ok: z.literal(true),
  configured: z.boolean(),
  baseUrl: z.string(),
  start: z.string().optional(),
  notify: notifySettingsSchema,
})

/** Every write action answers with `ok`, plus an error message when it failed. */
export const actionResponseSchema = z.object({
  ok: z.boolean(),
  error: z.string().optional(),
  verified: z.boolean().optional(),
  start: z.string().optional(),
  notify: notifySettingsSchema.optional(),
})

export type NotifySettings = z.infer<typeof notifySettingsSchema>
export type StatusResponse = z.infer<typeof statusResponseSchema>
export type ActionResponse = z.infer<typeof actionResponseSchema>
