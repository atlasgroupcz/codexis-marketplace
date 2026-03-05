import { useCallback } from 'react'
import { toast } from 'sonner'

export interface OperationToastOptions {
  loadingMessage: string
  successMessage: string
  errorMessage?: string
}

export interface UseOperationToastReturn {
  withToast: <T>(
    options: OperationToastOptions,
    operation: () => Promise<T>,
  ) => Promise<T | undefined>
}

/**
 * Hook for wrapping async operations with toast notifications.
 * Shows loading state, then success or error message.
 */
export function useOperationToast(): UseOperationToastReturn {
  const withToast = useCallback(
    async <T>(
      options: OperationToastOptions,
      operation: () => Promise<T>,
    ): Promise<T | undefined> => {
      const { loadingMessage, successMessage, errorMessage } = options
      const toastId = toast.loading(loadingMessage)

      try {
        const result = await operation()
        toast.success(successMessage, { id: toastId })
        return result
      } catch (error) {
        const message =
          error instanceof Error
            ? error.message
            : errorMessage || 'Operation failed'
        toast.error(message, { id: toastId })
        return undefined
      }
    },
    [],
  )

  return { withToast }
}
