import { Loader2Icon } from 'lucide-react'
import { cn } from '@workspace/ui/lib/utils'

interface SpinnerProps {
  className?: string
  size?: 'sm' | 'md' | 'lg'
}

const sizeClasses = {
  sm: 'size-4',
  md: 'size-6',
  lg: 'size-8',
}

export function Spinner({ className, size = 'md' }: SpinnerProps) {
  return (
    <div className={cn('flex items-center justify-center', className)}>
      <Loader2Icon
        className={cn('animate-spin text-muted-foreground', sizeClasses[size])}
      />
    </div>
  )
}
