import { useCallback, useEffect, useRef } from 'react'

interface BackoffConfig {
  readonly initialIntervalMs: number
  readonly maxIntervalMs: number
  readonly multiplier: number
}

type PollingStrategy =
  | {
      readonly mode: 'apollo-polling'
      readonly startPolling: (ms: number) => void
      readonly stopPolling: () => void
    }
  | {
      readonly mode: 'refetch'
      readonly refetch: () => Promise<unknown>
    }

interface UseInProgressPollingOptions {
  readonly isInProgress: boolean
  readonly polling: PollingStrategy
  readonly backoff?: Partial<BackoffConfig>
}

const DEFAULT_BACKOFF: BackoffConfig = {
  initialIntervalMs: 1000,
  maxIntervalMs: 60000,
  multiplier: 2,
}

/**
 * Polls for fresh data while an entity is in-progress, using exponential backoff.
 * Runs in parallel with WebSocket subscriptions as a safety net — subscriptions
 * provide instant updates, polling ensures data freshness even when events are missed.
 */
export function useInProgressPolling(options: UseInProgressPollingOptions): void {
  const { isInProgress, polling, backoff: backoffOverrides } = options
  const backoff: BackoffConfig = { ...DEFAULT_BACKOFF, ...backoffOverrides }

  const isInProgressRef = useRef(isInProgress)
  isInProgressRef.current = isInProgress

  const pollingRef = useRef(polling)
  pollingRef.current = polling

  const attemptRef = useRef(0)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const clearTimer = useCallback(() => {
    if (timerRef.current !== null) {
      clearTimeout(timerRef.current)
      timerRef.current = null
    }
  }, [])

  // Apollo-polling mode: bump interval via successive startPolling calls
  useEffect(() => {
    if (!isInProgress) {
      clearTimer()
      attemptRef.current = 0
      const currentPolling = pollingRef.current
      if (currentPolling.mode === 'apollo-polling') {
        currentPolling.stopPolling()
      }
      return
    }

    const currentPolling = pollingRef.current
    if (currentPolling.mode !== 'apollo-polling') {
      return
    }

    const initialMs = backoff.initialIntervalMs
    currentPolling.startPolling(initialMs)
    attemptRef.current = 0

    const scheduleBump = () => {
      attemptRef.current += 1

      const currentMs = Math.min(
        backoff.initialIntervalMs * Math.pow(backoff.multiplier, attemptRef.current - 1),
        backoff.maxIntervalMs,
      )

      // Already at max, no more bumps needed
      if (currentMs >= backoff.maxIntervalMs) {
        return
      }

      const nextMs = Math.min(
        backoff.initialIntervalMs * Math.pow(backoff.multiplier, attemptRef.current),
        backoff.maxIntervalMs,
      )

      timerRef.current = setTimeout(() => {
        if (!isInProgressRef.current) return
        const p = pollingRef.current
        if (p.mode === 'apollo-polling') {
          p.startPolling(nextMs)
        }
        scheduleBump()
      }, currentMs)
    }

    scheduleBump()

    return () => {
      clearTimer()
      const p = pollingRef.current
      if (p.mode === 'apollo-polling') {
        p.stopPolling()
      }
    }
  }, [isInProgress, backoff.initialIntervalMs, backoff.maxIntervalMs, backoff.multiplier, clearTimer])

  // Refetch mode: setTimeout chain with escalating delays
  useEffect(() => {
    if (!isInProgress) {
      clearTimer()
      attemptRef.current = 0
      return
    }

    const currentPolling = pollingRef.current
    if (currentPolling.mode !== 'refetch') {
      return
    }

    attemptRef.current = 0

    const scheduleRefetch = () => {
      const intervalMs = Math.min(
        backoff.initialIntervalMs * Math.pow(backoff.multiplier, attemptRef.current),
        backoff.maxIntervalMs,
      )

      timerRef.current = setTimeout(async () => {
        if (!isInProgressRef.current) return
        const p = pollingRef.current
        if (p.mode === 'refetch') {
          try {
            await p.refetch()
          } catch {
            // Swallow refetch errors — next interval will retry
          }
        }
        attemptRef.current += 1
        scheduleRefetch()
      }, intervalMs)
    }

    scheduleRefetch()

    return () => {
      clearTimer()
    }
  }, [isInProgress, backoff.initialIntervalMs, backoff.maxIntervalMs, backoff.multiplier, clearTimer])
}
