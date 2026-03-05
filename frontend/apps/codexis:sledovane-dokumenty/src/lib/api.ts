import { detailResponseSchema, overviewResponseSchema } from './schemas'
import type { DetailResponse, OverviewResponse } from './schemas'

type RawChange = {
  confirmed_on?: string | null
}

type RawTrackedDocument = {
  uuid?: string
  codexisId?: string
  name?: string
  added_on?: string
  tracking_type?: 'all' | 'document_changes' | 'related_documents_changes'
  parts?: Array<unknown>
  changes?: Array<RawChange>
}

type SampleDataResponse = {
  trackedDocuments?: Array<RawTrackedDocument>
}

function getApiBase(): string {
  if (import.meta.env.DEV) {
    return '/api/'
  }
  return './'
}

function buildSampleDataUrl(): string {
  return new URL('./assets/sample-data.json', window.location.href).toString()
}

function countChanges(changes: Array<RawChange> | undefined): {
  total: number
  unconfirmed: number
} {
  if (!Array.isArray(changes)) {
    return { total: 0, unconfirmed: 0 }
  }

  const total = changes.length
  const unconfirmed = changes.filter((change) => !change.confirmed_on).length
  return { total, unconfirmed }
}

async function fetchSampleData(): Promise<Array<RawTrackedDocument>> {
  const response = await fetch(buildSampleDataUrl(), {
    headers: { Accept: 'application/json' },
    cache: 'no-store',
  })

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }

  const json = (await response.json()) as SampleDataResponse
  return Array.isArray(json.trackedDocuments) ? json.trackedDocuments : []
}

function shouldUseSampleDataFallback(error: unknown): boolean {
  if (!(error instanceof Error)) {
    return false
  }

  return /HTTP 5\d\d/.test(error.message) || /Failed to fetch/.test(error.message)
}

async function fetchOverviewFromSampleData(): Promise<OverviewResponse> {
  const trackedDocuments = await fetchSampleData()
  const tracked_documents = trackedDocuments.map((document) => {
    const { total, unconfirmed } = countChanges(document.changes)

    return {
      uuid: document.uuid ?? '',
      codexisId: document.codexisId ?? '',
      name: document.name ?? '',
      added_on: document.added_on ?? new Date().toISOString(),
      tracking_type: document.tracking_type ?? 'all',
      unconfirmed_changes: unconfirmed,
      total_changes: total,
    }
  })

  return overviewResponseSchema.parse({
    mode: 'overview',
    generated_at: new Date().toISOString(),
    tracked_documents,
  })
}

async function fetchDetailFromSampleData(uuid: string): Promise<DetailResponse> {
  const trackedDocuments = await fetchSampleData()
  const document = trackedDocuments.find((item) => item.uuid === uuid)

  if (!document) {
    throw new Error('HTTP 404: Not Found')
  }

  const { total, unconfirmed } = countChanges(document.changes)

  return detailResponseSchema.parse({
    mode: 'detail',
    generated_at: new Date().toISOString(),
    document: {
      uuid: document.uuid ?? '',
      codexisId: document.codexisId ?? '',
      name: document.name ?? '',
      added_on: document.added_on ?? new Date().toISOString(),
      tracking_type: document.tracking_type ?? 'all',
      parts: Array.isArray(document.parts) ? document.parts : [],
      changes: Array.isArray(document.changes) ? document.changes : [],
      total_changes: total,
      unconfirmed_changes: unconfirmed,
    },
  })
}

export async function fetchOverview(): Promise<OverviewResponse> {
  try {
    const url = new URL(getApiBase(), window.location.href).toString()
    const response = await fetch(url, {
      headers: { Accept: 'application/json' },
      cache: 'no-store',
    })
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }
    const json: unknown = await response.json()
    return overviewResponseSchema.parse(json)
  } catch (error) {
    if (shouldUseSampleDataFallback(error)) {
      return fetchOverviewFromSampleData()
    }
    throw error
  }
}

export async function fetchDetail(uuid: string): Promise<DetailResponse> {
  try {
    const url = new URL(getApiBase(), window.location.href)
    url.searchParams.set('uuid', uuid)
    const response = await fetch(url.toString(), {
      headers: { Accept: 'application/json' },
      cache: 'no-store',
    })
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }
    const json: unknown = await response.json()
    return detailResponseSchema.parse(json)
  } catch (error) {
    if (shouldUseSampleDataFallback(error)) {
      return fetchDetailFromSampleData(uuid)
    }
    throw error
  }
}
