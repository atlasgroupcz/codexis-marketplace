'use client'

import { useLayoutEffect, useMemo, useRef, useState } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import { CheckIcon, CopyIcon } from 'lucide-react'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import {
  oneDark,
  oneLight,
} from 'react-syntax-highlighter/dist/esm/styles/prism'
import { Button } from '@workspace/ui/components/button'
import { cn } from '@workspace/ui/lib/utils'

export interface VirtualizedCodeBlockProps {
  code: string
  language: string
  className?: string
}

/**
 * Virtualized code block component for large files.
 * Only renders visible lines + overscan for optimal performance.
 * Uses react-syntax-highlighter with oneDark/oneLight themes.
 * Horizontal scrolling is synchronized across all lines via the parent container.
 */
export function VirtualizedCodeBlock({
  code,
  language,
  className,
}: VirtualizedCodeBlockProps) {
  const lines = useMemo(() => code.split('\n'), [code])
  const parentRef = useRef<HTMLDivElement>(null)
  const contentRef = useRef<HTMLDivElement>(null)
  const [contentWidth, setContentWidth] = useState<number | undefined>(undefined)

  const virtualizer = useVirtualizer({
    count: lines.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 20, // ~20px per line
    overscan: 30, // Render 30 extra lines above/below viewport
  })

  // Calculate line number width based on total lines
  const lineNumberWidth = useMemo(() => {
    const digits = String(lines.length).length
    return Math.max(digits * 0.6 + 1.5, 3) // Min 3rem
  }, [lines.length])
  const virtualItems = virtualizer.getVirtualItems()

  // Measure the maximum content width after render to enable synchronized horizontal scroll
  useLayoutEffect(() => {
    if (!contentRef.current) return

    const measureWidth = () => {
      const rows = contentRef.current?.querySelectorAll('[data-index]')
      if (!rows) return

      let maxWidth = 0
      rows.forEach((row) => {
        const codeElement = row.querySelector('code, span')
        if (codeElement) {
          maxWidth = Math.max(maxWidth, codeElement.scrollWidth)
        }
      })

      if (maxWidth > 0) {
        // Add line number width (in pixels) + some padding
        const lineNumPx = lineNumberWidth * 16 // rem to px (assuming 16px base)
        setContentWidth(maxWidth + lineNumPx + 32) // 32px extra padding
      }
    }

    // Measure after a small delay to ensure rendering is complete
    const timeoutId = setTimeout(measureWidth, 50)
    return () => clearTimeout(timeoutId)
  }, [virtualItems, lineNumberWidth])

  // Sync vertical scroll between line numbers and code content
  const lineNumbersRef = useRef<HTMLDivElement>(null)
  const codeContentRef = useRef<HTMLDivElement>(null)

  const handleScroll = (source: 'lineNumbers' | 'codeContent') => {
    const lineNumbers = lineNumbersRef.current
    const codeContent = codeContentRef.current
    if (!lineNumbers || !codeContent) return

    if (source === 'codeContent') {
      lineNumbers.scrollTop = codeContent.scrollTop
    } else {
      codeContent.scrollTop = lineNumbers.scrollTop
    }
  }

  return (
    <div className={cn('relative h-full w-full', className)}>
      {/* Copy button - positioned absolutely */}
      <div className="absolute top-2 right-4 z-10">
        <CopyButton content={code} />
      </div>

      {/* Two-column layout: fixed line numbers + scrollable code */}
      <div className="flex h-full w-full bg-background font-mono text-sm">
        {/* Line numbers column - only vertical scroll */}
        <div
          ref={lineNumbersRef}
          className="shrink-0 overflow-hidden border-r border-border bg-muted/30"
          style={{ width: `${lineNumberWidth}rem` }}
          onScroll={() => handleScroll('lineNumbers')}
        >
          <div
            style={{
              height: virtualizer.getTotalSize(),
              position: 'relative',
            }}
          >
            {virtualizer.getVirtualItems().map((virtualRow) => (
              <div
                key={virtualRow.key}
                className="px-2 text-right text-muted-foreground select-none"
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  transform: `translateY(${virtualRow.start}px)`,
                }}
              >
                {virtualRow.index + 1}
              </div>
            ))}
          </div>
        </div>

        {/* Code content column - both vertical and horizontal scroll */}
        <div
          ref={(el) => {
            // Combine refs for virtualizer scroll element and our sync ref
            parentRef.current = el
            codeContentRef.current = el
          }}
          className="flex-1 overflow-auto"
          onScroll={() => handleScroll('codeContent')}
        >
          <div
            ref={contentRef}
            style={{
              height: virtualizer.getTotalSize(),
              minWidth: contentWidth ?? '100%',
              position: 'relative',
            }}
          >
            {virtualizer.getVirtualItems().map((virtualRow) => (
              <div
                key={virtualRow.key}
                data-index={virtualRow.index}
                ref={virtualizer.measureElement}
                className="hover:bg-muted/50"
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  minWidth: '100%',
                  transform: `translateY(${virtualRow.start}px)`,
                }}
              >
                <HighlightedLine
                  content={lines[virtualRow.index] ?? ''}
                  language={language}
                />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

/**
 * Per-line syntax highlighting component.
 * Uses react-syntax-highlighter with oneDark/oneLight themes.
 */
function HighlightedLine({
  content,
  language,
}: {
  content: string
  language: string
}) {
  if (!content) {
    // Empty line - render non-breaking space to maintain height
    return <code className="flex-1 px-4 whitespace-pre">&nbsp;</code>
  }

  return (
    <div className="flex-1 whitespace-pre">
      {/* Light theme version */}
      <SyntaxHighlighter
        className="dark:hidden"
        language={language}
        style={oneLight}
        customStyle={{
          margin: 0,
          padding: '0 1rem',
          background: 'transparent',
          fontSize: 'inherit',
          lineHeight: 'inherit',
          whiteSpace: 'pre',
        }}
        codeTagProps={{
          style: {
            fontFamily: 'inherit',
            fontSize: 'inherit',
            whiteSpace: 'pre',
          },
        }}
        PreTag="span"
        CodeTag="code"
      >
        {content}
      </SyntaxHighlighter>
      {/* Dark theme version */}
      <SyntaxHighlighter
        className="hidden dark:block"
        language={language}
        style={oneDark}
        customStyle={{
          margin: 0,
          padding: '0 1rem',
          background: 'transparent',
          fontSize: 'inherit',
          lineHeight: 'inherit',
          whiteSpace: 'pre',
        }}
        codeTagProps={{
          style: {
            fontFamily: 'inherit',
            fontSize: 'inherit',
            whiteSpace: 'pre',
          },
        }}
        PreTag="span"
        CodeTag="code"
      >
        {content}
      </SyntaxHighlighter>
    </div>
  )
}

/**
 * Copy button component for copying entire file content.
 */
function CopyButton({ content }: { content: string }) {
  const [isCopied, setIsCopied] = useState(false)

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(content)
      setIsCopied(true)
      setTimeout(() => setIsCopied(false), 2000)
    } catch {
      // Silently fail if clipboard is not available
    }
  }

  const Icon = isCopied ? CheckIcon : CopyIcon

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={handleCopy}
      className={cn(
        'size-8 bg-background/80 backdrop-blur-sm border shadow-sm',
        isCopied && 'text-green-500',
      )}
    >
      <Icon className="size-4" />
    </Button>
  )
}
