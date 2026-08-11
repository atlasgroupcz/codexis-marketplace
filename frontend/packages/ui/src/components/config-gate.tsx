import { Button } from '@workspace/ui/components/button'
import { ExternalLink } from 'lucide-react'
import type { ComponentType } from 'react'

export interface ConfigGateProps {
  icon: ComponentType<{ className?: string; strokeWidth?: number }>
  eyebrow?: string
  title: string
  text: string
  action: { label: string; onClick: () => void; testId?: string }
  getKey?: { href: string; label: string; hint: string }
}

export function ConfigGate({ icon: Icon, eyebrow, title, text, action, getKey }: ConfigGateProps) {
  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="flex max-w-md flex-col items-center text-center">
        <span className="bg-secondary text-primary ring-brand-100 flex size-14 items-center justify-center rounded-2xl ring-1">
          <Icon className="size-7" strokeWidth={1.75} />
        </span>
        {eyebrow && (
          <p className="text-primary mt-5 mb-2 text-xs font-semibold tracking-[0.14em] uppercase">
            {eyebrow}
          </p>
        )}
        <h2 className="font-display text-foreground mt-5 text-2xl leading-tight text-balance">
          {title}
        </h2>
        <p className="text-muted-foreground mt-3 text-[15px] leading-relaxed text-pretty">{text}</p>
        <Button onClick={action.onClick} size="lg" className="mt-7" data-testid={action.testId}>
          <Icon className="size-4" />
          {action.label}
        </Button>
        {getKey && (
          <>
            <a
              href={getKey.href}
              target="_blank"
              rel="noreferrer"
              className="text-primary mt-4 inline-flex items-center gap-1 text-sm hover:underline"
            >
              <ExternalLink className="size-3.5" />
              {getKey.label}
            </a>
            <p className="text-muted-foreground mt-2 max-w-xs text-xs text-pretty">{getKey.hint}</p>
          </>
        )}
      </div>
    </div>
  )
}
