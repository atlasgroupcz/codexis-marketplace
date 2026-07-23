import { z } from 'zod'

export const notifySettingsSchema = z.object({
  email: z.boolean(),
  inApp: z.boolean(),
  recipients: z.array(z.string()),
})

export const statusResponseSchema = z.object({
  ok: z.literal(true),
  configured: z.boolean(),
  baseUrl: z.string(),
  start: z.string().optional(),
  /** Signed-in user's address, used by the recipients editor. */
  userEmail: z.string().optional(),
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
