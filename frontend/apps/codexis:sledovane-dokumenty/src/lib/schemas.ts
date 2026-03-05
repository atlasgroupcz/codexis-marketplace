import { z } from 'zod'

export const sourceDocumentSchema = z.object({
  codexisId: z.string(),
  name: z.string(),
})
export type SourceDocument = z.infer<typeof sourceDocumentSchema>

export const changeTypeSchema = z.enum(['document_change', 'related_change'])
export type ChangeType = z.infer<typeof changeTypeSchema>

export const trackingTypeSchema = z.enum([
  'all',
  'document_changes',
  'related_documents_changes',
])
export type TrackingType = z.infer<typeof trackingTypeSchema>

export const documentPartSchema = z.object({
  partId: z.string(),
  label: z.string(),
})
export type DocumentPart = z.infer<typeof documentPartSchema>

export const changeSchema = z.object({
  source_documents: z.array(sourceDocumentSchema),
  detected_on: z.string(),
  effective_on: z.string(),
  change_type: changeTypeSchema,
  description_md: z.string(),
  confirmed_on: z.string().nullable(),
})
export type Change = z.infer<typeof changeSchema>

export const trackedDocumentSummarySchema = z.object({
  uuid: z.string(),
  codexisId: z.string(),
  name: z.string(),
  added_on: z.string(),
  tracking_type: trackingTypeSchema,
  unconfirmed_changes: z.number(),
  total_changes: z.number(),
})
export type TrackedDocumentSummary = z.infer<typeof trackedDocumentSummarySchema>

export const overviewResponseSchema = z.object({
  mode: z.literal('overview'),
  generated_at: z.string(),
  tracked_documents: z.array(trackedDocumentSummarySchema),
})
export type OverviewResponse = z.infer<typeof overviewResponseSchema>

export const documentDetailSchema = z.object({
  uuid: z.string(),
  codexisId: z.string(),
  name: z.string(),
  added_on: z.string(),
  tracking_type: trackingTypeSchema,
  parts: z.array(documentPartSchema),
  changes: z.array(changeSchema),
  total_changes: z.number(),
  unconfirmed_changes: z.number(),
})
export type DocumentDetail = z.infer<typeof documentDetailSchema>

export const detailResponseSchema = z.object({
  mode: z.literal('detail'),
  generated_at: z.string(),
  document: documentDetailSchema,
})
export type DetailResponse = z.infer<typeof detailResponseSchema>
