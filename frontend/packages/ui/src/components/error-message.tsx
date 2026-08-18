import { Button } from '@workspace/ui/components/button'
import { cn } from '@workspace/ui/lib/utils'
import { AlertCircle } from 'lucide-react'
import type { ComponentType } from 'react'

export interface ErrorMessageProps {
  message: string
  onRetry?: () => void
  retryLabel?: string
  variant?: 'plain' | 'card'
  icon?: ComponentType<{ className?: string }>
  className?: string
}

export function ErrorMessage({
  message,
  onRetry,
  retryLabel = 'Retry',
  variant = 'plain',
  icon: Icon = AlertCircle,
  className,
}: ErrorMessageProps) {
  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center gap-4 p-8 text-center',
        variant === 'card' &&
          'rounded-lg border border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-950/40',
        className,
      )}
    >
      <Icon
        className={cn(
          'size-10',
          variant === 'card' ? 'text-red-600 dark:text-red-300' : 'text-destructive',
        )}
      />
      <p className="text-muted-foreground text-sm">{message}</p>
      {onRetry && (
        <Button variant="outline" size="sm" onClick={onRetry}>
          {retryLabel}
        </Button>
      )}
    </div>
  )
}
