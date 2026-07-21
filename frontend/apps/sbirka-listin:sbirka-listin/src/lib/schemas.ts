import { z } from 'zod'

/** One filed document as stored in the watcher baseline. */
export const listinaSchema = z.object({
  cislo: z.string(),
  typ: z.string(),
  zalozeno: z.string().nullable().optional(),
  // Present when the document is already downloaded to the daemon → the dashboard
  // shows "Otevřít" instead of "Stáhnout". (Absent on a change's nove_listiny.)
  path: z.string().nullable().optional(),
})
export type Listina = z.infer<typeof listinaSchema>

/** One check run that found something (new and/or vanished listiny). A newly
 * filed document has the same shape as a stored one, so it reuses `listinaSchema`. */
export const changeSchema = z.object({
  detected_on: z.string(),
  nove_listiny: z.array(listinaSchema).optional().default([]),
  zmizele_listiny: z.array(z.string()).optional().default([]),
  confirmed_on: z.string().nullable().optional(),
})
export type Change = z.infer<typeof changeSchema>

/** One tracked subject, as shown in the overview list. */
export const subjektSummarySchema = z.object({
  ico: z.string(),
  nazev: z.string(),
  spisovaznacka: z.string().nullable().optional(),
  sidlo: z.string().nullable().optional(),
  listin: z.number(),
  nepotvrzenych_zmen: z.number().optional().default(0),
  last_check_at: z.string().nullable().optional(),
})
export type SubjektSummary = z.infer<typeof subjektSummarySchema>

/** One tracked subject with its full listiny + change history. */
export const subjektDetailSchema = z.object({
  ico: z.string(),
  nazev: z.string(),
  spisovaznacka: z.string().nullable().optional(),
  sidlo: z.string().nullable().optional(),
  listiny: z.array(listinaSchema).optional().default([]),
  changes: z.array(changeSchema).optional().default([]),
  added_on: z.string(),
  last_check_at: z.string().nullable().optional(),
})
export type SubjektDetail = z.infer<typeof subjektDetailSchema>

export const overviewResponseSchema = z.object({
  mode: z.literal('overview'),
  generated_at: z.string(),
  subjekty: z.array(subjektSummarySchema),
})
export type OverviewResponse = z.infer<typeof overviewResponseSchema>

export const detailResponseSchema = z.object({
  mode: z.literal('detail'),
  generated_at: z.string(),
  subjekt: subjektDetailSchema,
})
export type DetailResponse = z.infer<typeof detailResponseSchema>

export const actionResponseSchema = z.object({
  ok: z.boolean(),
  error: z.string().optional(),
})
export type ActionResponse = z.infer<typeof actionResponseSchema>

export const checkAllResponseSchema = z.object({
  ok: z.boolean(),
  checked: z.number(),
  new_changes: z.number(),
  errors: z.array(
    z.object({
      ico: z.string(),
      error: z.string(),
    }),
  ),
})
export type CheckAllResponse = z.infer<typeof checkAllResponseSchema>

export const downloadResponseSchema = z.object({
  ok: z.boolean(),
  ico: z.string(),
  cislo: z.string(),
  path: z.string(),
  url: z.string().nullable().optional(),
  reused: z.boolean(),
})
export type DownloadResponse = z.infer<typeof downloadResponseSchema>

export const notifySettingsSchema = z.object({
  email: z.boolean(),
  inApp: z.boolean(),
  recipients: z.array(z.string()),
})
export type NotifySettingsValues = z.infer<typeof notifySettingsSchema>

export const notifyResponseSchema = z.object({
  mode: z.literal('notify'),
  settings: notifySettingsSchema,
  userEmail: z.string(),
})
export type NotifyResponse = z.infer<typeof notifyResponseSchema>
