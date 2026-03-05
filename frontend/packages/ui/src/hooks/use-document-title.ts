import { useEffect, useRef } from 'react'

const APP_NAME = 'CDX'

/**
 * Hook to update the document title dynamically.
 * Automatically appends the app name and restores the previous title on unmount.
 *
 * @param title - The page-specific title (e.g., "homework.docx" or "My Chat")
 * @param options - Configuration options
 */
export function useDocumentTitle(
  title: string | null | undefined,
  options?: { restoreOnUnmount?: boolean },
) {
  const { restoreOnUnmount = true } = options ?? {}
  const previousTitleRef = useRef<string | null>(null)

  useEffect(() => {
    // Store the previous title on first run
    if (previousTitleRef.current === null) {
      previousTitleRef.current = document.title
    }

    // Set new title
    document.title = title ? `${title} - ${APP_NAME}` : APP_NAME

    // Restore previous title on unmount if requested
    return () => {
      if (restoreOnUnmount && previousTitleRef.current !== null) {
        document.title = previousTitleRef.current
      }
    }
  }, [title, restoreOnUnmount])
}
