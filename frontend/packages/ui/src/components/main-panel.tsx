import { cn } from '@workspace/ui/lib/utils'
import type { HTMLAttributes } from 'react'

export type MainPanelProps = HTMLAttributes<HTMLDivElement>

export function MainPanel({ className, children, ...props }: MainPanelProps) {
  return (
    <div
      className={cn(
        'flex h-full flex-col overflow-hidden bg-background',
        className,
      )}
      {...props}
    >
      {children}
    </div>
  )
}

export type MainPanelHeaderProps = HTMLAttributes<HTMLDivElement>

export function MainPanelHeader({
  className,
  children,
  ...props
}: MainPanelHeaderProps) {
  return (
    <header
      className={cn(
        'flex h-10 shrink-0 items-center gap-2 px-3 border-b',
        className,
      )}
      {...props}
    >
      {children}
    </header>
  )
}

export type MainPanelContentProps = HTMLAttributes<HTMLDivElement>

export function MainPanelContent({
  className,
  children,
  ...props
}: MainPanelContentProps) {
  return (
    <main
      className={cn('flex-1 overflow-hidden min-h-0', className)}
      {...props}
    >
      {children}
    </main>
  )
}
