import { ArrowLeft, ExternalLink, Settings } from "lucide-react"
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
  /** Manual link rendered next to the title. Ignored without `brand` or `back`. */
  docs?: HeaderDocsLinkProps
}

/**
 * Title with the manual link hanging off it. When space runs out the title is
 * the one that gives way: the reader already knows which app they opened, but
 * a clipped "Jak to funguje?" would stop being a sentence.
 */
function TitleRow({
  title,
  titleClassName,
  docs,
}: {
  title: string
  titleClassName: string
  docs?: HeaderDocsLinkProps
}) {
  return (
    <div className="flex min-w-0 items-baseline gap-2.5">
      <h1 className={titleClassName}>{title}</h1>
      {docs && (
        <span className="shrink-0">
          <HeaderDocsLink {...docs} />
        </span>
      )}
    </div>
  )
}

export function AppHeader({ brand, back, actions, docs }: AppHeaderProps) {
  return (
    <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b px-4">
      {brand ? (
        <div className="flex min-w-0 items-center gap-2.5">
          <span className="bg-secondary text-primary ring-brand-100 flex size-9 items-center justify-center rounded-xl ring-1">
            <brand.icon className="size-5" strokeWidth={1.75} />
          </span>
          <div className="min-w-0">
            <TitleRow
              title={brand.title}
              titleClassName="font-display text-foreground truncate text-[15px] leading-tight"
              docs={docs}
            />
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
            <TitleRow
              title={back.title}
              titleClassName="font-display truncate text-[15px] leading-tight"
              docs={docs}
            />
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

export interface HeaderDocsLinkProps {
  app: DocsApp
  lang: string
  /** Visible text of the link, not a tooltip. */
  label: string
}

/**
 * Link to the app's manual page, reading the same as the daemon's docs link.
 * A user moving between the daemon and an app then meets one pattern, not two,
 * and the label says what is behind the link instead of hinting at it.
 *
 * The daemon sets this link one third smaller than the title it sits next to
 * (18px title, 14px link). Copying its 14px here would put the link within a
 * pixel of the 15px app title, and the two would compete instead of reading as
 * heading and aside - which also makes the serif title / sans link pairing look
 * like a collision rather than a hierarchy. So the ratio is copied, not the size.
 */
export function HeaderDocsLink({ app, lang, label }: HeaderDocsLinkProps) {
  return (
    <a
      href={docsUrl(app, lang)}
      target="_blank"
      rel="noreferrer"
      className="text-muted-foreground hover:text-foreground inline-flex items-center gap-1 text-xs underline underline-offset-2 transition-colors"
    >
      {label}
      <ExternalLink className="size-3" />
    </a>
  )
}
