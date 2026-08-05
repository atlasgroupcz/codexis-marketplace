import { useCallback, useEffect, useState } from 'react'
import type { ReportSummary, TopicSummary } from '@/lib/schemas'
import { fetchDetail } from '@/lib/api'

export interface FeedReport extends ReportSummary {
  topicUuid: string
  topicName: string
}

interface UseReportsFeedResult {
  reports: Array<FeedReport>
  loading: boolean
  error: Error | null
  refetch: () => void
}

/**
 * The overview endpoint only carries report counts, so the cross-topic feed of
 * new reports is assembled client-side: fetch the detail of every topic that
 * has reports, then flatten and sort the report summaries (newest first).
 */
export function useReportsFeed(topics: Array<TopicSummary>): UseReportsFeedResult {
  const [reports, setReports] = useState<Array<FeedReport>>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<Error | null>(null)

  // Stable string key so the effect only re-runs when the relevant set changes.
  const key = topics
    .filter((topic) => topic.total_reports > 0)
    .map((topic) => topic.uuid)
    .join(',')

  const load = useCallback(() => {
    const ids = key ? key.split(',') : []
    if (ids.length === 0) {
      setReports([])
      setError(null)
      setLoading(false)
      return
    }
    setLoading(true)
    setError(null)
    void Promise.allSettled(ids.map((id) => fetchDetail(id)))
      .then((results) => {
        const flat: Array<FeedReport> = []
        let firstError: Error | null = null
        let anyOk = false
        for (const result of results) {
          if (result.status === 'fulfilled') {
            anyOk = true
            const { topic, reports: topicReports } = result.value
            for (const report of topicReports) {
              flat.push({ ...report, topicUuid: topic.uuid, topicName: topic.name })
            }
          } else if (!firstError) {
            firstError =
              result.reason instanceof Error ? result.reason : new Error(String(result.reason))
          }
        }
        if (!anyOk && firstError) {
          setError(firstError)
          return
        }
        flat.sort((a, b) => {
          const aKey = a.checked_at ?? ''
          const bKey = b.checked_at ?? ''
          return aKey < bKey ? 1 : -1
        })
        setReports(flat)
      })
      .finally(() => setLoading(false))
  }, [key])

  useEffect(() => {
    load()
  }, [load])

  return { reports, loading, error, refetch: load }
}
