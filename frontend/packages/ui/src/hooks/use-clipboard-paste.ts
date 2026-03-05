import { useCallback, useEffect } from 'react'
import type { RefObject } from 'react'

interface UseClipboardPasteOptions {
  onPaste: (files: Array<File>) => void
  enabled?: boolean
}

export function useClipboardPaste(
  containerRef: RefObject<HTMLElement | null>,
  { onPaste, enabled = true }: UseClipboardPasteOptions
): void {
  const handlePaste = useCallback(
    (event: ClipboardEvent) => {
      // Skip if target is an input/textarea (they have their own paste handling)
      const target = event.target as HTMLElement
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable
      ) {
        return
      }

      const items = event.clipboardData?.items
      if (!items) return

      const files: Array<File> = []
      for (const item of items) {
        if (item.kind === 'file') {
          const file = item.getAsFile()
          if (file) files.push(file)
        }
      }

      if (files.length > 0) {
        event.preventDefault()
        onPaste(files)
      }
    },
    [onPaste]
  )

  useEffect(() => {
    const container = containerRef.current
    if (!container || !enabled) return

    container.addEventListener('paste', handlePaste)
    return () => container.removeEventListener('paste', handlePaste)
  }, [containerRef, handlePaste, enabled])
}
