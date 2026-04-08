import { z } from 'zod'

export const operaceSchema = z.object({
  nazev: z.string(),
  datumProvedeni: z.string(),
})
export type Operace = z.infer<typeof operaceSchema>

export const proceedingChangeSchema = z.object({
  detected_on: z.string(),
  old_stav: z.string(),
  new_stav: z.string(),
  new_operations: z.array(operaceSchema).optional().default([]),
  stav_uhrady_changed: z.boolean().optional().default(false),
  old_stav_uhrady: z.string().nullable().optional(),
  new_stav_uhrady: z.string().nullable().optional(),
  confirmed_on: z.string().nullable().optional(),
})
export type ProceedingChange = z.infer<typeof proceedingChangeSchema>

export const proceedingSummarySchema = z.object({
  uuid: z.string(),
  cislo_rizeni: z.string(),
  typ_rizeni: z.string(),
  label: z.string().optional().default(''),
  stav: z.string(),
  stav_uhrady: z.string().nullable().optional(),
  stav_uhrady_label: z.string().nullable().optional(),
  datum_prijeti: z.string(),
  added_on: z.string(),
  last_check_at: z.string(),
  last_op_date: z.string().nullable().optional(),
  operace_count: z.number(),
  changes_count: z.number(),
  unconfirmed_count: z.number().optional().default(0),
  provedene_operace: z.array(operaceSchema),
})
export type ProceedingSummary = z.infer<typeof proceedingSummarySchema>

export const proceedingDetailSchema = proceedingSummarySchema.extend({
  changes: z.array(proceedingChangeSchema).optional().default([]),
})
export type ProceedingDetail = z.infer<typeof proceedingDetailSchema>

export const overviewResponseSchema = z.object({
  mode: z.literal('overview'),
  generated_at: z.string(),
  proceedings: z.array(proceedingSummarySchema),
  api_key_configured: z.boolean().optional().default(false),
  api_key_masked: z.string().optional().default(''),
})
export type OverviewResponse = z.infer<typeof overviewResponseSchema>

export const detailResponseSchema = z.object({
  mode: z.literal('detail'),
  generated_at: z.string(),
  proceeding: proceedingDetailSchema,
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
      cislo_rizeni: z.string(),
      error: z.string(),
    }),
  ),
})
export type CheckAllResponse = z.infer<typeof checkAllResponseSchema>

export const settingsStatusSchema = z.object({
  configured: z.boolean(),
  maskedKey: z.string().optional(),
})
export type SettingsStatus = z.infer<typeof settingsStatusSchema>
