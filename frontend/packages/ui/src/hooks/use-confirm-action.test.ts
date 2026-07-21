import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { act, renderHook } from '@testing-library/react'
import { useConfirmAction } from './use-confirm-action'

describe('useConfirmAction', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.runOnlyPendingTimers()
    vi.useRealTimers()
  })

  it('arms on the first trigger without running the action', () => {
    const action = vi.fn(() => Promise.resolve())
    const { result } = renderHook(() => useConfirmAction(action))

    expect(result.current.confirming).toBe(false)
    act(() => result.current.trigger())
    expect(result.current.confirming).toBe(true)
    expect(action).not.toHaveBeenCalled()
  })

  it('runs the action on the second trigger and tracks pending', async () => {
    let resolve: () => void = () => {}
    const action = vi.fn(() => new Promise<void>((r) => { resolve = r }))
    const { result } = renderHook(() => useConfirmAction(action))

    act(() => result.current.trigger())
    act(() => result.current.trigger())
    expect(action).toHaveBeenCalledTimes(1)
    expect(result.current.pending).toBe(true)

    await act(async () => {
      resolve()
      // Let the action's .finally microtask (setPending(false)) settle inside act.
      await Promise.resolve()
    })
    expect(result.current.pending).toBe(false)
  })

  it('auto-disarms after the timeout window', () => {
    const action = vi.fn(() => Promise.resolve())
    const { result } = renderHook(() => useConfirmAction(action, { timeoutMs: 3000 }))

    act(() => result.current.trigger())
    expect(result.current.confirming).toBe(true)

    act(() => vi.advanceTimersByTime(3000))
    expect(result.current.confirming).toBe(false)
    expect(action).not.toHaveBeenCalled()
  })

  it('cancel disarms without running the action', () => {
    const action = vi.fn(() => Promise.resolve())
    const { result } = renderHook(() => useConfirmAction(action))

    act(() => result.current.trigger())
    expect(result.current.confirming).toBe(true)

    act(() => result.current.cancel())
    expect(result.current.confirming).toBe(false)

    // The (now cleared) timer must not re-fire any state change.
    act(() => vi.advanceTimersByTime(3000))
    expect(result.current.confirming).toBe(false)
    expect(action).not.toHaveBeenCalled()
  })
})
