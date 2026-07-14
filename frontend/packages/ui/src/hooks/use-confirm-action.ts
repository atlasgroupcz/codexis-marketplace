import { useCallback, useEffect, useRef, useState } from 'react'

export interface UseConfirmActionResult {
  /**
   * True once the first `trigger()` armed the action, until it is confirmed,
   * cancelled, or the auto-disarm timer elapses. Callers typically swap their
   * button to a destructive variant while this is true.
   */
  confirming: boolean
  /** True while the confirmed action is in flight. */
  pending: boolean
  /**
   * Two-step trigger:
   * - the first call arms confirmation and starts the auto-disarm timer
   * - a second call within `timeoutMs` runs `action`
   */
  trigger: () => void
  /** Manually disarm without running the action. */
  cancel: () => void
}

/**
 * "Click twice to confirm" helper shared by the watchdog detail panes.
 *
 * The first `trigger()` arms the action and starts a timer that auto-disarms
 * after `timeoutMs`. A second `trigger()` within that window runs `action` and
 * tracks it via `pending`. The timer is cleared on confirm and on unmount so it
 * never fires against an unmounted component.
 */
export function useConfirmAction(
  action: () => void | Promise<unknown>,
  { timeoutMs = 3000 }: { timeoutMs?: number } = {},
): UseConfirmActionResult {
  const [confirming, setConfirming] = useState(false)
  const [pending, setPending] = useState(false)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const clearTimer = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current)
      timerRef.current = null
    }
  }, [])

  useEffect(() => clearTimer, [clearTimer])

  const cancel = useCallback(() => {
    clearTimer()
    setConfirming(false)
  }, [clearTimer])

  const trigger = useCallback(() => {
    if (!confirming) {
      setConfirming(true)
      timerRef.current = setTimeout(() => {
        timerRef.current = null
        setConfirming(false)
      }, timeoutMs)
      return
    }
    clearTimer()
    setPending(true)
    Promise.resolve(action()).finally(() => setPending(false))
  }, [confirming, action, timeoutMs, clearTimer])

  return { confirming, pending, trigger, cancel }
}
