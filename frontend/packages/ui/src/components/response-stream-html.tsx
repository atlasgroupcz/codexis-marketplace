"use client"

import React, {
  
  useCallback,
  useEffect,
  useRef,
  useState
} from "react"
import type {ReactNode} from "react";

export type HTMLResponseStreamMode = "fade" // Currently only "fade" is supported

export type UseHTMLTextStreamOptions = {
  htmlStream: string | AsyncIterable<string>
  speed?: number // 1-100, influences fadeDuration and segmentDelay
  mode?: HTMLResponseStreamMode // Defaults to "fade"
  onComplete?: () => void
  fadeDuration?: number // Custom fade duration in ms (overrides speed calculation)
  segmentDelay?: number // Custom delay between segments in ms (overrides speed calculation)
  onError?: (error: unknown) => void
}

export type UseHTMLTextStreamResult = {
  renderedContent: ReactNode
  isComplete: boolean
  reset: () => void
  startStreaming: () => void
  // Pause/Resume are not implemented for HTML fade mode as it's CSS-driven once initiated
}

function useHTMLTextStream({
  htmlStream,
  speed = 20,
  onComplete,
  fadeDuration,
  segmentDelay,
  onError,
}: UseHTMLTextStreamOptions): UseHTMLTextStreamResult {
  const [renderedContent, setRenderedContent] = useState<ReactNode>(null)
  const [isComplete, setIsComplete] = useState(false)

  const speedRef = useRef(speed)
  const fadeDurationRef = useRef(fadeDuration)
  const segmentDelayRef = useRef(segmentDelay)
  const streamAbortControllerRef = useRef<AbortController | null>(null)
  const onCompleteRef = useRef(onComplete)
  const onErrorRef = useRef(onError)
  const currentHtmlRef = useRef<string>("")

  useEffect(() => {
    speedRef.current = speed
    fadeDurationRef.current = fadeDuration
    segmentDelayRef.current = segmentDelay
  }, [speed, fadeDuration, segmentDelay])

  useEffect(() => {
    onCompleteRef.current = onComplete
  }, [onComplete])

  useEffect(() => {
    onErrorRef.current = onError
  }, [onError])

  const getCalculatedFadeDuration = useCallback(() => {
    if (typeof fadeDurationRef.current === "number")
      return Math.max(10, fadeDurationRef.current)
    const normalizedSpeed = Math.min(100, Math.max(1, speedRef.current))
    // Slower speed means longer fade. Example: speed 1 -> 1000ms, speed 100 -> 100ms
    return Math.max(100, Math.round(1000 / Math.sqrt(normalizedSpeed)))
  }, [])

  const getCalculatedSegmentDelay = useCallback(() => {
    if (typeof segmentDelayRef.current === "number")
      return Math.max(0, segmentDelayRef.current)
    const normalizedSpeed = Math.min(100, Math.max(1, speedRef.current))
    // Slower speed means longer delay. Example: speed 1 -> 100ms, speed 100 -> 10ms
    return Math.max(10, Math.round(100 / Math.sqrt(normalizedSpeed)))
  }, [])

  const transformNodeToReact = useCallback(
    (
      node: Node,
      path: string // Base path for generating unique keys
    ): ReactNode | Array<ReactNode> | null => {
      if (node.nodeType === Node.ELEMENT_NODE) {
        const element = node as Element
        const children: Array<ReactNode> = []
        element.childNodes.forEach((child, index) => {
          const childPath = `${path}/${element.tagName.toLowerCase()}[${index}]`
          const transformedChild = transformNodeToReact(child, childPath)
          if (transformedChild) {
            if (Array.isArray(transformedChild)) {
              children.push(...transformedChild)
            } else {
              children.push(transformedChild)
            }
          }
        })

        const props: Record<string, unknown> = { key: path }
        const attrNameMap: { [key: string]: string } = {
          class: "className",
          for: "htmlFor",
          // Common HTML attributes that have different names in React
          tabindex: "tabIndex",
          readonly: "readOnly",
          maxlength: "maxLength",
          cellspacing: "cellSpacing",
          cellpadding: "cellPadding",
          rowspan: "rowSpan",
          colspan: "colSpan",
          usemap: "useMap",
          formaction: "formAction",
          formenctype: "formEncType",
          formmethod: "formMethod",
          formnovalidate: "formNoValidate",
          formtarget: "formTarget",
        }

        for (const attr of Array.from(element.attributes)) {

          const propName =
            attrNameMap[attr.name.toLowerCase()] || attr.name

          // Handle boolean attributes correctly
          if (typeof props[propName] === 'boolean') {
            props[propName] = true;
          } else if (propName === 'style' && typeof attr.value === 'string') {
             // Basic style string to object conversion (can be improved for full CSS parsing)
            const styleObject: { [key: string]: string } = {};
            attr.value.split(';').forEach(styleRule => {
              const [key, value] = styleRule.split(':');
              if (key && value) {
                const camelCaseKey = key.trim().replace(/-([a-z])/g, (_, char: string) => char.toUpperCase());
                styleObject[camelCaseKey] = value.trim();
              }
            });
            props[propName] = styleObject;
          }
          else {
            props[propName] = attr.value
          }
        }
        
        return React.createElement(
          element.tagName.toLowerCase(),
          props,
          children.length > 0 ? children : undefined
        )
      } else if (node.nodeType === Node.TEXT_NODE) {
        const textContent = node.textContent || ""
        if (textContent.trim() === "") {
          return textContent // Preserve whitespace nodes as is
        }

        // Split into words, keeping spaces with the word before them or as separate segments if they are significant
        const words = textContent.split(/(\s+)/).filter(Boolean) // "Hello world" -> ["Hello", " ", "world"]

        return words.map((wordOrSpace, wordIndexInTextNode) => {
          if (wordOrSpace.trim() === "") {
            // Preserve spaces without animation, ensure they have keys
            return React.createElement("span", {key: `${path}-space-${wordIndexInTextNode}`}, wordOrSpace);
          }
          const segmentKey = `${path}-word-${wordIndexInTextNode}`
          return React.createElement(
            "span",
            {
              key: segmentKey,
              className: "html-fade-segment", // For potential base styling via CSS class
              style: {
                display: "inline", // Ensure words flow like text
                opacity: 0,
                animationName: "htmlFadeIn",
                animationDuration: `${getCalculatedFadeDuration()}ms`,
                animationTimingFunction: "ease-out",
                animationFillMode: "forwards",
                animationDelay: `${
                  wordIndexInTextNode * getCalculatedSegmentDelay()
                }ms`,
              },
            },
            wordOrSpace
          )
        })
      } else if (node.nodeType === Node.COMMENT_NODE) {
        return null // Comments are not rendered
      }
      return null
    },
    [getCalculatedFadeDuration, getCalculatedSegmentDelay]
  )

  const processHtmlString = useCallback(
    (html: string) => {
      if (typeof DOMParser === "undefined") {
        // SSR or environment without DOMParser
        setRenderedContent(html) // Fallback to raw HTML
        onErrorRef.current?.(new Error("DOMParser not available."))
        return
      }
      try {
        const parser = new DOMParser()
        const doc = parser.parseFromString(html, "text/html")
        // segmentCounterRef.current = 0 removed
        // Transform only the body's children, or the whole doc if no body (e.g. fragment)
        const rootNodeToTransform = doc.body
        const transformedContent = Array.from(rootNodeToTransform.childNodes)
          .map((child, index) =>
            transformNodeToReact(child, `root[${index}]`)
          )
          .filter(Boolean) // Filter out nulls (e.g. comments)
          .flat() // flatten arrays of spans from text nodes
        
        setRenderedContent(transformedContent)

      } catch (error) {
        console.error("Error processing HTML string:", error)
        setRenderedContent(html) // Fallback to raw HTML on error
        onErrorRef.current?.(error)
      }
    },
    [transformNodeToReact]
  )

  const reset = useCallback(() => {
    if (streamAbortControllerRef.current) {
      streamAbortControllerRef.current.abort()
      streamAbortControllerRef.current = null
    }
    currentHtmlRef.current = ""
    setRenderedContent(null)
    setIsComplete(false)
    // segmentCounterRef.current = 0 removed
  }, [])

  const startStreaming = useCallback(() => {
    reset()

    if (typeof htmlStream === "string") {
      currentHtmlRef.current = htmlStream
      processHtmlString(htmlStream)
      setIsComplete(true)
      onCompleteRef.current?.()
    } else if (typeof htmlStream[Symbol.asyncIterator] === 'function') {
      const processAsync = async () => {
        streamAbortControllerRef.current = new AbortController()
        const signal = streamAbortControllerRef.current.signal
        try {
          for await (const chunk of htmlStream) {
            if (signal.aborted) return
            currentHtmlRef.current += chunk
            processHtmlString(currentHtmlRef.current)
          }
          if (!signal.aborted) {
            setIsComplete(true)
            onCompleteRef.current?.()
          }
        } catch (error) {
          if (!signal.aborted) {
            console.error("Error processing HTML stream:", error)
            setIsComplete(true) // Mark as complete even on error to stop processing
            onErrorRef.current?.(error)
            onCompleteRef.current?.() // Call onComplete even on error
          }
        } finally {
            if (streamAbortControllerRef.current.signal === signal) {
              streamAbortControllerRef.current = null
            }
        }
      }
      processAsync()
    } else {
        setIsComplete(true); // No valid stream
        onCompleteRef.current?.();
    }
  }, [htmlStream, reset, processHtmlString])

  useEffect(() => {
    startStreaming()
    return () => {
      reset() // Cleanup on unmount
    }
  }, [startStreaming, reset]) // htmlStream is a dependency of startStreaming

  return {
    renderedContent,
    isComplete,
    reset,
    startStreaming,
  }
}

export type ResponseStreamHTMLProps = {
  htmlStream: string | AsyncIterable<string>
  mode?: HTMLResponseStreamMode
  speed?: number
  className?: string
  onComplete?: () => void
  as?: keyof React.JSX.IntrinsicElements | React.ComponentType<{
    className?: string
    children?: ReactNode
  }>
  fadeDuration?: number
  segmentDelay?: number
  onError?: (error: unknown) => void
}

function ResponseStreamHTML({
  htmlStream,
  mode = "fade",
  speed = 10,
  className = "",
  onComplete,
  as = "div",
  fadeDuration,
  segmentDelay = 0,
  onError,
}: ResponseStreamHTMLProps) {
  const { renderedContent } = useHTMLTextStream({
      htmlStream,
      speed,
      mode,
      onComplete,
      fadeDuration,
      segmentDelay,
      onError,
    })

  // CSS keyframes for the fade-in animation
  // Using a <style> tag is one way to ensure keyframes are available.
  // Alternatively, this could be in a global CSS file.
  const fadeAnimationStyle = `
    @keyframes htmlFadeIn {
      from { opacity: 0; }
      to { opacity: 1; }
    }
  `

  const Container = as as React.ElementType

  return (
    <>
      <style>{fadeAnimationStyle}</style>
      <Container className={className}>{renderedContent}</Container>
    </>
  )
}

export { useHTMLTextStream, ResponseStreamHTML } 
