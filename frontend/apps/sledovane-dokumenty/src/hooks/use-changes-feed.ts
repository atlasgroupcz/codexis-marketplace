import { useCallback, useEffect, useState } from 'react'
import type { Change, TrackedDocumentSummary } from '@/lib/schemas'
import { fetchDetail } from '@/lib/api'

export interface FeedChange extends Change {
  documentUuid: string
  documentName: string
}

interface UseChangesFeedResult {
  changes: Array<FeedChange>
  loading: boolean
  error: Error | null
  refetch: () => void
}

/**
 * The overview endpoint only carries change counts, so the cross-document
 * activity feed is assembled client-side: fetch the detail of every document
 * that has history, then flatten and sort its changes (newest first).
 */
export function useChangesFeed(documents: Array<TrackedDocumentSummary>): UseChangesFeedResult {
  const [changes, setChanges] = useState<Array<FeedChange>>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  // Stable string key so the effect only re-runs when the relevant set changes.
  const key = documents
    .filter((doc) => doc.total_changes > 0)
    .map((doc) => doc.uuid)
    .join(',')

  const load = useCallback(() => {
    const ids = key ? key.split(',') : []
    if (ids.length === 0) {
      setChanges([])
      setError(null)
      setLoading(false)
      return
    }
    setLoading(true)
    setError(null)
    void Promise.allSettled(ids.map((id) => fetchDetail(id)))
      .then((results) => {
        const flat: Array<FeedChange> = []
        let firstError: Error | null = null
        let anyOk = false
        for (const result of results) {
          if (result.status === 'fulfilled') {
            anyOk = true
            const { document } = result.value
            for (const change of document.changes) {
              flat.push({ ...change, documentUuid: document.uuid, documentName: document.name })
            }
          } else if (!firstError) {
            firstError = result.reason instanceof Error ? result.reason : new Error(String(result.reason))
          }
        }
        if (!anyOk && firstError) {
          setError(firstError)
          return
        }
        flat.sort((a, b) => (a.detected_on < b.detected_on ? 1 : -1))
        setChanges(flat)
      })
      .finally(() => setLoading(false))
  }, [key])

  useEffect(() => {
    load()
  }, [load])

  return { changes, loading, error, refetch: load }
}
