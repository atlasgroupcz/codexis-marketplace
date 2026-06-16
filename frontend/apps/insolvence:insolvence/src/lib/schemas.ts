import { z } from 'zod'

export const udalostSchema = z.object({
  id: z.string().optional(),
  datum: z.string().optional().default(''),
  popis: z.string().optional().default(''),
  oddil: z.string().optional(),
})
export type Udalost = z.infer<typeof udalostSchema>

export const rizeniSchema = z.object({
  spisova_znacka: z.string(),
  soud: z.string().optional().default(''),
  druh: z.string().optional().default('neurceno'),
  druh_raw: z.string().optional().default(''),
  stav: z.string().optional().default(''),
  datum_zahajeni: z.string().optional().default(''),
  datum_ukonceni: z.string().optional().default(''),
  aktivni: z.boolean().optional().default(false),
  url_detail: z.string().optional().default(''),
  udalosti: z.array(udalostSchema).optional().default([]),
})
export type Rizeni = z.infer<typeof rizeniSchema>

export const changeSchema = z.object({
  detected_on: z.string(),
  typ: z.string(),
  spisova_znacka: z.string().optional().default(''),
  popis: z.string().optional().default(''),
  druh: z.string().optional(),
  old_stav: z.string().optional(),
  new_stav: z.string().optional(),
  udalost: udalostSchema.optional(),
  confirmed_on: z.string().nullable().optional(),
})
export type SubjectChange = z.infer<typeof changeSchema>

export const subjectSummarySchema = z.object({
  uuid: z.string(),
  kind: z.enum(['company', 'person']),
  label: z.string().optional().default(''),
  display_name: z.string(),
  ico: z.string().optional().default(''),
  nazev: z.string().optional().default(''),
  prijmeni: z.string().optional().default(''),
  jmeno: z.string().optional().default(''),
  datum_narozeni: z.string().optional().default(''),
  ma_aktivni_insolvenci: z.boolean(),
  rizeni_count: z.number(),
  primary_druh: z.string().nullable().optional(),
  primary_druh_label: z.string().nullable().optional(),
  primary_stav: z.string().nullable().optional(),
  added_on: z.string().optional().default(''),
  last_check_at: z.string().optional().default(''),
  changes_count: z.number().optional().default(0),
  unconfirmed_count: z.number().optional().default(0),
})
export type SubjectSummary = z.infer<typeof subjectSummarySchema>

export const subjectDetailSchema = subjectSummarySchema.extend({
  rizeni: z.array(rizeniSchema).optional().default([]),
  changes: z.array(changeSchema).optional().default([]),
})
export type SubjectDetail = z.infer<typeof subjectDetailSchema>

export const overviewResponseSchema = z.object({
  mode: z.literal('overview'),
  generated_at: z.string(),
  subjects: z.array(subjectSummarySchema),
})
export type OverviewResponse = z.infer<typeof overviewResponseSchema>

export const detailResponseSchema = z.object({
  mode: z.literal('detail'),
  generated_at: z.string(),
  subject: subjectDetailSchema,
})
export type DetailResponse = z.infer<typeof detailResponseSchema>

export const actionResponseSchema = z.object({
  ok: z.boolean(),
  error: z.string().optional(),
  subject: subjectSummarySchema.optional(),
  marked: z.number().optional(),
})
export type ActionResponse = z.infer<typeof actionResponseSchema>

export const checkAllResponseSchema = z.object({
  ok: z.boolean(),
  checked: z.number(),
  new_changes: z.number(),
  errors: z.array(
    z.object({
      display_name: z.string(),
      error: z.string(),
    }),
  ),
})
export type CheckAllResponse = z.infer<typeof checkAllResponseSchema>

export const dphAccountSchema = z.object({
  ucet: z.string(),
  iban: z.boolean().optional().default(false),
  datum_zverejneni: z.string().optional().default(''),
})
export type DphAccount = z.infer<typeof dphAccountSchema>

export const dphResponseSchema = z.object({
  mode: z.literal('dph'),
  dic: z.string().optional().default(''),
  found: z.boolean().optional().default(false),
  je_platce: z.boolean().optional().default(false),
  nespolehlivy: z.boolean().nullable().optional().default(null),
  nespolehlivy_od: z.string().optional().default(''),
  cislo_fu: z.string().optional().default(''),
  ucty: z.array(dphAccountSchema).optional().default([]),
})
export type DphResponse = z.infer<typeof dphResponseSchema>
