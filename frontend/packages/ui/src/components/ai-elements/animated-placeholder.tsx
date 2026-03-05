'use client'

import { AnimatePresence, motion } from 'motion/react'
import { cn } from '@workspace/ui/lib/utils'

export interface AnimatedPlaceholderProps {
  /** The placeholder text to display */
  text: string
  /** Unique key for triggering animations on text change */
  animationKey: number | string
  /** Whether the placeholder should be visible */
  visible: boolean
  /** Additional CSS classes */
  className?: string
}

/**
 * An animated placeholder component that fades in/out with smooth transitions.
 * Designed to overlay a textarea and provide animated placeholder text.
 *
 * Features:
 * - Smooth fade and slide transitions using framer-motion
 * - Truncates long text with ellipsis
 * - Pointer-events: none to allow clicking through to the textarea
 */
export function AnimatedPlaceholder({
  text,
  animationKey,
  visible,
  className,
}: AnimatedPlaceholderProps) {
  return (
    <div
      className={cn(
        'pointer-events-none absolute inset-0 flex items-start overflow-hidden',
        className,
      )}
      aria-hidden="true"
    >
      <AnimatePresence mode="wait">
        {visible && (
          <motion.span
            key={animationKey}
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{
              duration: 0.35,
              ease: [0.4, 0, 0.2, 1],
            }}
            className="line-clamp-2 text-base md:text-sm text-muted-foreground/60"
          >
            {text}
          </motion.span>
        )}
      </AnimatePresence>
    </div>
  )
}
