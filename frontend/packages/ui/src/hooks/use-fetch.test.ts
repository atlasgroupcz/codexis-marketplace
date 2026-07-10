import { describe, expect, it, vi } from 'vitest'
import { act, renderHook, waitFor } from '@testing-library/react'
import { useFetch } from './use-fetch'

describe('useFetch', () => {
  it('resolves data and clears loading', async () => {
    const { result } = renderHook(() => useFetch(() => Promise.resolve(42)))
    expect(result.current.loading).toBe(true)
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.data).toBe(42)
    expect(result.current.error).toBeNull()
  })

  it('captures errors', async () => {
    const { result } = renderHook(() => useFetch(() => Promise.reject(new Error('boom'))))
    await waitFor(() => expect(result.current.error?.message).toBe('boom'))
  })

  it('refetch re-runs the fetcher', async () => {
    const fetcher = vi.fn(() => Promise.resolve('x'))
    const { result } = renderHook(() => useFetch(fetcher))
    await waitFor(() => expect(result.current.loading).toBe(false))
    act(() => result.current.refetch())
    await waitFor(() => expect(fetcher).toHaveBeenCalledTimes(2))
  })
})
