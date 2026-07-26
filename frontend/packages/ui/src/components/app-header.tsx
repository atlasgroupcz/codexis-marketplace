import { ArrowLeft, CircleQuestionMark, Settings } from "lucide-react"
import { Button } from "@workspace/ui/components/button"
import { docsUrl } from "@workspace/ui/lib/docs"
import { cn } from "@workspace/ui/lib/utils"
import type { DocsApp } from "@workspace/ui/lib/docs"
import type { ComponentType, ReactNode } from "react"

interface BrandSpec {
  icon: ComponentType<{ className?: string; strokeWidth?: number }>
  title: string
  subtitle?: string
}
interface BackSpec {
  onClick: () => void
  label: string
  title: string
  subtitle?: string
  subtitleMono?: boolean
}
export interface AppHeaderProps {
  brand?: BrandSpec
  back?: BackSpec
  actions?: ReactNode
}

export function AppHeader({ brand, back, actions }: AppHeaderProps) {
  return (
    <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b px-4">
      {brand ? (
        <div className="flex min-w-0 items-center gap-2.5">
          <span className="bg-secondary text-primary ring-brand-100 flex size-9 items-center justify-center rounded-xl ring-1">
            <brand.icon className="size-5" strokeWidth={1.75} />
          </span>
          <div className="min-w-0">
            <h1 className="font-display text-foreground truncate text-[15px] leading-tight">{brand.title}</h1>
            {brand.subtitle && (
              <p className="text-muted-foreground hidden truncate text-xs sm:block">{brand.subtitle}</p>
            )}
          </div>
        </div>
      ) : back ? (
        <div className="flex min-w-0 items-center gap-2">
          <Button variant="ghost" size="icon" onClick={back.onClick} aria-label={back.label}>
            <ArrowLeft className="size-4" />
          </Button>
          <div className="min-w-0">
            <h1 className="font-display truncate text-[15px] leading-tight">{back.title}</h1>
            {back.subtitle && (
              <p
                className={cn(
                  "text-muted-foreground truncate",
                  back.subtitleMono ? "font-mono text-[11px]" : "text-xs"
                )}
              >
                {back.subtitle}
              </p>
            )}
          </div>
        </div>
      ) : (
        <div />
      )}
      <div className="flex items-center gap-2">{actions}</div>
    </header>
  )
}

export interface HeaderGearProps {
  onClick: () => void
  title: string
}

export function HeaderGear({ onClick, title }: HeaderGearProps) {
  return (
    <Button variant="ghost" size="icon" onClick={onClick} title={title} aria-label={title}>
      <Settings className="size-5" />
    </Button>
  )
}

export interface HeaderHelpProps {
  app: DocsApp
  lang: string
  title: string
}

/**
 * Link to the app's manual page. It sits in the header of every screen, so a user
 * who does not know what to do has one place to look, in every app the same.
 */
export function HeaderHelp({ app, lang, title }: HeaderHelpProps) {
  return (
    <Button variant="ghost" size="icon" asChild title={title} aria-label={title}>
      <a href={docsUrl(app, lang)} target="_blank" rel="noreferrer">
        <CircleQuestionMark className="size-5" />
      </a>
    </Button>
  )
}
