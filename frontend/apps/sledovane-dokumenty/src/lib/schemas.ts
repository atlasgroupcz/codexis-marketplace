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

export const amendmentSchema = z.object({
  id: z.string(),
  name: z.string(),
})
export type Amendment = z.infer<typeof amendmentSchema>

export const changeSchema = z.object({
  source_documents: z.array(sourceDocumentSchema),
  detected_on: z.string(),
  // related_change has no legal effective date - omitted by backend.
  effective_on: z.string().optional(),
  change_type: changeTypeSchema,
  description_md: z.string(),
  confirmed_on: z.string().nullable(),
  compare_url: z.string().optional(),
  amendments: z.array(amendmentSchema).optional().default([]),
})
export type Change = z.infer<typeof changeSchema>

export const groupRefSchema = z.object({
  id: z.string(),
  name: z.string(),
})
export type GroupRef = z.infer<typeof groupRefSchema>

export const groupSchema = z.object({
  id: z.string(),
  name: z.string(),
  members: z.array(z.string()),
})
export type Group = z.infer<typeof groupSchema>

export const trackedDocumentSummarySchema = z.object({
  uuid: z.string(),
  codexisId: z.string(),
  name: z.string(),
  added_on: z.string(),
  tracking_type: trackingTypeSchema,
  unconfirmed_changes: z.number(),
  total_changes: z.number(),
  groups: z.array(groupRefSchema).optional().default([]),
})
export type TrackedDocumentSummary = z.infer<typeof trackedDocumentSummarySchema>

export const overviewResponseSchema = z.object({
  mode: z.literal('overview'),
  generated_at: z.string(),
  tracked_documents: z.array(trackedDocumentSummarySchema),
  groups: z.array(groupSchema).optional().default([]),
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
  groups: z.array(groupRefSchema).optional().default([]),
  user_notes: z.array(z.string()).optional().default([]),
})
export type DocumentDetail = z.infer<typeof documentDetailSchema>

export const actionResponseSchema = z.object({
  ok: z.boolean(),
  error: z.string().optional(),
})
export type ActionResponse = z.infer<typeof actionResponseSchema>

export const detailResponseSchema = z.object({
  mode: z.literal('detail'),
  generated_at: z.string(),
  document: documentDetailSchema,
  groups: z.array(groupSchema).optional().default([]),
})
export type DetailResponse = z.infer<typeof detailResponseSchema>

// ── watched folders ─────────────────────────────────────────────────────────

export const notifySettingsSchema = z.object({
  email: z.boolean(),
  inApp: z.boolean(),
  recipients: z.array(z.string()).optional().default([]),
})
export type NotifySettings = z.infer<typeof notifySettingsSchema>

export const notifyResponseSchema = z.object({
  mode: z.literal('notify'),
  settings: notifySettingsSchema,
  userEmail: z.string(),
})
export type NotifyResponse = z.infer<typeof notifyResponseSchema>

export const folderChangeSchema = z.object({
  detected_on: z.string(),
  effective_on: z.string().nullable().optional(),
  old_version_id: z.string().nullable().optional(),
  new_version_id: z.string().nullable().optional(),
  amendments: z.array(amendmentSchema).optional().default([]),
  compare_url: z.string().optional(),
  confirmed_on: z.string().nullable().optional(),
})
export type FolderChange = z.infer<typeof folderChangeSchema>

export const legislationRefSchema = z.object({
  uri: z.string().nullable().optional(),
  text: z.string(),
  codexisId: z.string().nullable().optional(),
})
export type LegislationRef = z.infer<typeof legislationRefSchema>

export const trackedLegislationSchema = legislationRefSchema.extend({
  tracked: z.boolean().optional().default(false),
  baselineVersionId: z.string().nullable().optional(),
  lastKnownVersionId: z.string().nullable().optional(),
  changes: z.array(folderChangeSchema).optional().default([]),
  unconfirmed_changes: z.number().optional().default(0),
})
export type TrackedLegislation = z.infer<typeof trackedLegislationSchema>

export const watchedDocumentSchema = z.object({
  path: z.string(),
  sha256: z.string(),
  discoveredAt: z.string().nullable().optional(),
  extractedAt: z.string().nullable(),
  updatedAt: z.string().nullable().optional(),
  legislation: z.array(legislationRefSchema).optional().default([]),
})
export type WatchedDocument = z.infer<typeof watchedDocumentSchema>

export const folderSummarySchema = z.object({
  uuid: z.string().optional(),
  root: z.string(),
  name: z.string(),
  added_on: z.string().nullable().optional(),
  documents: z.number(),
  pending_harvest: z.number(),
  legislation: z.number(),
  unconfirmed_changes: z.number(),
  last_check_at: z.string().nullable().optional(),
})
export type FolderSummary = z.infer<typeof folderSummarySchema>

export const foldersResponseSchema = z.object({
  mode: z.literal('folders'),
  generated_at: z.string(),
  folders: z.array(folderSummarySchema),
  settings: notifySettingsSchema,
})
export type FoldersResponse = z.infer<typeof foldersResponseSchema>

export const folderDetailSchema = z.object({
  root: z.string(),
  name: z.string().nullable(),
  added_on: z.string().nullable().optional(),
  documents: z.array(watchedDocumentSchema),
  documents_count: z.number(),
  pending_harvest: z.number(),
  legislation: z.array(trackedLegislationSchema),
  last_check_at: z.string().nullable().optional(),
})
export type FolderDetail = z.infer<typeof folderDetailSchema>

export const folderResponseSchema = z.object({
  mode: z.literal('folder'),
  generated_at: z.string(),
  folder: folderDetailSchema,
})
export type FolderResponse = z.infer<typeof folderResponseSchema>

export const browseEntrySchema = z.object({
  name: z.string(),
  path: z.string(),
  isDir: z.boolean(),
})
export type BrowseEntry = z.infer<typeof browseEntrySchema>

export const browseResponseSchema = z.object({
  mode: z.literal('browse'),
  generated_at: z.string(),
  path: z.string(),
  parent: z.string().nullable(),
  home: z.string(),
  entries: z.array(browseEntrySchema),
})
export type BrowseResponse = z.infer<typeof browseResponseSchema>
