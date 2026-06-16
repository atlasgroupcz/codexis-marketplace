import { z } from 'zod'

export const markKindSchema = z.enum(['word', 'figurative', 'combined'])
export type MarkKind = z.infer<typeof markKindSchema>

export const tierSchema = z.enum(['high', 'medium', 'low'])
export type Tier = z.infer<typeof tierSchema>

export const scoresSchema = z.object({
  text_orthographic: z.number(),
  text_phonetic: z.number(),
  text_combined: z.number(),
  image_phash: z.number().nullable(),
  image_color: z.number().nullable(),
  vienna_overlap: z.number(),
  class_overlap: z.number(),
  sign_similarity: z.number(),
  overall: z.number(),
})
export type Scores = z.infer<typeof scoresSchema>

export const aiAssessmentSchema = z.object({
  risk: z.enum(['high', 'medium', 'low', 'none']),
  visual: z.string().optional().default(''),
  aural: z.string().optional().default(''),
  conceptual: z.string().optional().default(''),
  goods_similarity: z.string().optional().default(''),
  summary: z.string().optional().default(''),
  assessed_on: z.string().optional().default(''),
})
export type AiAssessment = z.infer<typeof aiAssessmentSchema>

export const collisionSchema = z.object({
  detected_on: z.string(),
  source: z.string().optional().default(''),
  source_id: z.string().optional().default(''),
  mark_text: z.string().optional().default(''),
  mark_kind: z.string().optional().default(''),
  applicant: z.string().optional().default(''),
  status: z.string().optional().default(''),
  filing_date: z.string().optional().default(''),
  nice_classes: z.array(z.number()).optional().default([]),
  vienna_codes: z.array(z.string()).optional().default([]),
  territory: z.string().optional().default(''),
  office: z.string().optional().default(''),
  url_detail: z.string().optional().default(''),
  image_url: z.string().optional().default(''),
  image_path: z.string().nullable().optional().default(null),
  scores: scoresSchema,
  tier: tierSchema,
  ai_assessment: aiAssessmentSchema.nullable().optional().default(null),
  confirmed_on: z.string().nullable().optional().default(null),
})
export type Collision = z.infer<typeof collisionSchema>

export const markSummarySchema = z.object({
  uuid: z.string(),
  kind: markKindSchema,
  label: z.string().optional().default(''),
  display_name: z.string(),
  text: z.string().optional().default(''),
  has_logo: z.boolean().optional().default(false),
  nice_classes: z.array(z.number()).optional().default([]),
  territories: z.array(z.string()).optional().default([]),
  owner_name: z.string().optional().default(''),
  added_on: z.string().optional().default(''),
  last_check_at: z.string().optional().default(''),
  collisions_count: z.number().optional().default(0),
  unconfirmed_count: z.number().optional().default(0),
  worst_tier: tierSchema.nullable().optional().default(null),
})
export type MarkSummary = z.infer<typeof markSummarySchema>

export const markDetailSchema = markSummarySchema.extend({
  threshold: z.number().optional().default(0.6),
  vienna_codes: z.array(z.string()).optional().default([]),
  collisions: z.array(collisionSchema).optional().default([]),
})
export type MarkDetail = z.infer<typeof markDetailSchema>

export const overviewResponseSchema = z.object({
  mode: z.literal('overview'),
  generated_at: z.string(),
  marks: z.array(markSummarySchema),
})
export type OverviewResponse = z.infer<typeof overviewResponseSchema>

export const detailResponseSchema = z.object({
  mode: z.literal('detail'),
  generated_at: z.string(),
  mark: markDetailSchema,
})
export type DetailResponse = z.infer<typeof detailResponseSchema>

export const actionResponseSchema = z.object({
  ok: z.boolean(),
  error: z.string().optional(),
  mark: markSummarySchema.optional(),
  marked: z.number().optional(),
  new_collisions: z.number().optional(),
})
export type ActionResponse = z.infer<typeof actionResponseSchema>

export const checkAllResponseSchema = z.object({
  ok: z.boolean(),
  checked: z.number(),
  new_collisions: z.number(),
  errors: z.array(z.object({ display_name: z.string(), error: z.string() })),
})
export type CheckAllResponse = z.infer<typeof checkAllResponseSchema>
