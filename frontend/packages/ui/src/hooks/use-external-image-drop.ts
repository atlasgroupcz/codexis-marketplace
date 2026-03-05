import { useCallback, useEffect, useRef, useState } from 'react'
import type { RefObject } from 'react'

interface UseExternalImageDropOptions {
  onDrop: (files: Array<File>) => void
  onFallback: (url: string) => void
  enabled?: boolean
}

interface UseExternalImageDropReturn {
  isDragOver: boolean
}

// Image extensions we'll try to fetch
const IMAGE_EXTENSIONS = ['.jpg', '.jpeg', '.png', '.gif', '.webp', '.svg', '.bmp', '.ico']

// Check if URL looks like an image
function isImageUrl(url: string): boolean {
  try {
    const urlObj = new URL(url)
    const pathname = urlObj.pathname.toLowerCase()
    return IMAGE_EXTENSIONS.some((ext) => pathname.endsWith(ext))
  } catch {
    return false
  }
}

// Extract filename from URL or generate one
function extractFilename(url: string, contentType: string): string {
  try {
    const urlObj = new URL(url)
    const pathname = urlObj.pathname
    const segments = pathname.split('/').filter(Boolean)
    const lastSegment = segments[segments.length - 1]

    // Check if last segment looks like a real filename with extension
    // (not just the extension itself like ".jpg" or "a.jpg" where "a" is too short)
    if (lastSegment && IMAGE_EXTENSIONS.some((ext) => lastSegment.toLowerCase().endsWith(ext))) {
      // Get the name part without extension
      const extMatch = IMAGE_EXTENSIONS.find((ext) => lastSegment.toLowerCase().endsWith(ext))
      const nameWithoutExt = extMatch ? lastSegment.slice(0, -extMatch.length) : lastSegment

      // Only use the filename if the name part has at least 2 characters
      if (nameWithoutExt.length >= 2) {
        return lastSegment
      }
    }
  } catch {
    // URL parsing failed, fall through to generated name
  }

  // Generate filename from content type
  const ext = contentType.split('/')[1]?.split(';')[0] || 'jpg'
  const normalizedExt = ext === 'jpeg' ? 'jpg' : ext
  return `image-${Date.now()}.${normalizedExt}`
}

// Get URL from drag event data
function getUrlFromDragEvent(event: DragEvent): string | null {
  const dataTransfer = event.dataTransfer
  if (!dataTransfer) return null

  // Try text/uri-list first (standard for URL drops)
  let url = dataTransfer.getData('text/uri-list')
  if (url) {
    // uri-list can have multiple URLs separated by newlines, take the first
    const firstUrl = url.split('\n')[0] ?? ''
    url = firstUrl.trim()
    if (url && !url.startsWith('#')) return url
  }

  // Fallback to text/plain
  url = dataTransfer.getData('text/plain').trim()
  if (url && (url.startsWith('http://') || url.startsWith('https://'))) {
    return url
  }

  return null
}

export function useExternalImageDrop(
  containerRef: RefObject<HTMLElement | null>,
  { onDrop, onFallback, enabled = true }: UseExternalImageDropOptions
): UseExternalImageDropReturn {
  const [isDragOver, setIsDragOver] = useState(false)
  const dragCounterRef = useRef(0)

  const handleDragEnter = useCallback((event: DragEvent) => {
    event.preventDefault()
    dragCounterRef.current += 1

    const url = getUrlFromDragEvent(event)
    if (url && isImageUrl(url)) {
      setIsDragOver(true)
    }
  }, [])

  const handleDragLeave = useCallback((event: DragEvent) => {
    event.preventDefault()
    dragCounterRef.current -= 1
    if (dragCounterRef.current === 0) {
      setIsDragOver(false)
    }
  }, [])

  const handleDragOver = useCallback((event: DragEvent) => {
    event.preventDefault()
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy'
    }
  }, [])

  const handleDrop = useCallback(
    async (event: DragEvent) => {
      event.preventDefault()
      setIsDragOver(false)
      dragCounterRef.current = 0

      const url = getUrlFromDragEvent(event)
      if (!url || !isImageUrl(url)) return

      try {
        const response = await fetch(url)
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`)
        }

        const blob = await response.blob()
        const contentType = response.headers.get('content-type') || 'image/jpeg'
        const filename = extractFilename(url, contentType)
        const file = new File([blob], filename, { type: contentType })

        onDrop([file])
      } catch {
        // Fetch failed (likely CORS), use fallback
        onFallback(url)
      }
    },
    [onDrop, onFallback]
  )

  useEffect(() => {
    const container = containerRef.current
    if (!container || !enabled) return

    container.addEventListener('dragenter', handleDragEnter)
    container.addEventListener('dragleave', handleDragLeave)
    container.addEventListener('dragover', handleDragOver)
    container.addEventListener('drop', handleDrop)

    return () => {
      container.removeEventListener('dragenter', handleDragEnter)
      container.removeEventListener('dragleave', handleDragLeave)
      container.removeEventListener('dragover', handleDragOver)
      container.removeEventListener('drop', handleDrop)
    }
  }, [containerRef, handleDragEnter, handleDragLeave, handleDragOver, handleDrop, enabled])

  return { isDragOver }
}
