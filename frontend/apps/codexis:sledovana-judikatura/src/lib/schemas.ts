import { z } from 'zod'

export const documentSchema = z.object({
  codexisId: z.string().optional().default(''),
  title: z.string().optional().default(''),
  spZn: z.string().optional().default(''),
  court: z.string().optional().default(''),
  doc_type: z.string().optional().default(''),
  issued_on: z.string().optional().default(''),
  legal_sentence: z.string().nullable().optional().default(''),
  summary: z.string().optional().default(''),
  changes_established_view: z.boolean().optional().default(false),
  practical_conclusion: z.string().optional().default(''),
})
export type Document = z.infer<typeof documentSchema>

export const reportAreaSchema = z.object({
  name: z.string(),
  documents: z.array(documentSchema).optional().default([]),
  area_summary: z.string().optional().default(''),
})
export type ReportArea = z.infer<typeof reportAreaSchema>

export const reportSchema = z.object({
  report_id: z.string(),
  checked_at: z.string().optional().default(''),
  period_from: z.string().nullable().optional(),
  period_to: z.string().nullable().optional(),
  found_count: z.number().optional().default(0),
  areas: z.array(reportAreaSchema).optional().default([]),
  overall_summary: z.string().optional().default(''),
  summary_for_lawyer: z.string().nullable().optional(),
  summary_for_hr: z.string().nullable().optional(),
  summary_en: z.string().nullable().optional(),
  confirmed_on: z.string().nullable().optional(),
})
export type Report = z.infer<typeof reportSchema>

export const topicAreaSchema = z.object({
  name: z.string(),
  baseline_summary: z.string().nullable().optional(),
  added_at: z.string().optional(),
  baseline_set_at: z.string().optional(),
})
export type TopicArea = z.infer<typeof topicAreaSchema>

export const topicSchema = z.object({
  uuid: z.string(),
  name: z.string(),
  notes: z.array(z.string()).optional().default([]),
  areas: z.array(topicAreaSchema).optional().default([]),
  filters: z.record(z.string(), z.unknown()).optional().default({}),
  created_at: z.string().optional().default(''),
  last_check_at: z.string().nullable().optional(),
})
export type Topic = z.infer<typeof topicSchema>

export const topicSummarySchema = z.object({
  uuid: z.string(),
  name: z.string(),
  areas: z.number(),
  notes: z.number(),
  created_at: z.string().nullable().optional(),
  last_check_at: z.string().nullable().optional(),
  total_reports: z.number(),
  unconfirmed_reports: z.number(),
})
export type TopicSummary = z.infer<typeof topicSummarySchema>

export const reportSummarySchema = z.object({
  report_id: z.string(),
  checked_at: z.string().optional(),
  period_from: z.string().nullable().optional(),
  period_to: z.string().nullable().optional(),
  found_count: z.number().optional().default(0),
  confirmed_on: z.string().nullable().optional(),
})
export type ReportSummary = z.infer<typeof reportSummarySchema>

export const overviewResponseSchema = z.object({
  mode: z.literal('overview'),
  generated_at: z.string(),
  topics: z.array(topicSummarySchema),
})
export type OverviewResponse = z.infer<typeof overviewResponseSchema>

export const detailResponseSchema = z.object({
  mode: z.literal('detail'),
  generated_at: z.string(),
  topic: topicSchema,
  reports: z.array(reportSummarySchema),
})
export type DetailResponse = z.infer<typeof detailResponseSchema>

export const reportResponseSchema = z.object({
  mode: z.literal('report'),
  generated_at: z.string(),
  topic: z.object({
    uuid: z.string(),
    name: z.string(),
  }),
  report: reportSchema,
})
export type ReportResponse = z.infer<typeof reportResponseSchema>

export const actionResponseSchema = z.object({
  ok: z.boolean(),
  error: z.string().optional(),
})
export type ActionResponse = z.infer<typeof actionResponseSchema>
