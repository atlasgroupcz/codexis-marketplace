import { useTranslation } from 'react-i18next'
import { BookMarked, FolderClock } from 'lucide-react'

type Section = 'documents' | 'folders'

/** Unconfirmed changes waiting in each watchdog. */
export interface SectionCounts {
  documents: number
  folders: number
}

interface SectionSwitchProps {
  section: Section
  onChange: (section: Section) => void
  counts: SectionCounts
}

/**
 * Primary navigation between the two watchdogs, as a rail down the left edge.
 *
 * Vertical on purpose. The screen beside it carries an underlined tab row, and
 * every horizontal shape tried above that row read as the same level however
 * slim it got - two rows of underlined text do not become a hierarchy by
 * shrinking one of them. A different axis cannot be mistaken for tabs at all,
 * and it costs no height.
 *
 * Selection is a tint of the same green the rail is washed in, never a border
 * or a raised chip: a ring around the item reads as the box shape this layout
 * exists to avoid. Collapsed to icons the tint alone is too faint to carry the
 * state, so the selected icon goes green as well.
 *
 * The expanded width is katastr's 300px list column, so the two apps in the
 * plugin put their left panel in the same place. Narrower than that the rail
 * read as a sliver hung under a much heavier header.
 *
 * Below `md` only the icons remain - katastr drops its panel entirely there,
 * but this one is navigation, so hiding it would strand you in one watchdog.
 * The label stays in the accessible name via `sr-only`, so the rail never
 * becomes the icon-only switch that users failed to find.
 *
 * The badge counts unconfirmed changes, so the watchdog you are not looking at
 * can still ask for attention.
 *
 * Marked up as navigation rather than a tablist: picking an item rewrites a URL
 * param and swaps the whole screen, and there is no tabpanel for a tab to own.
 */
export function SectionSwitch({ section, onChange, counts }: SectionSwitchProps) {
  const { t } = useTranslation()
  const items = [
    {
      key: 'documents' as const,
      label: t('folders.section.documents'),
      icon: BookMarked,
      badge: counts.documents,
    },
    {
      key: 'folders' as const,
      label: t('folders.section.folders'),
      icon: FolderClock,
      badge: counts.folders,
    },
  ]
  return (
    <nav
      aria-label={t('folders.section.label')}
      className="bg-secondary/40 flex w-[72px] shrink-0 flex-col gap-1 border-r p-2 md:w-[300px] md:p-3"
      data-testid="section-switch"
    >
      <p
        aria-hidden
        className="text-muted-foreground hidden px-3 pt-1 pb-2 text-[11px] font-semibold tracking-wider uppercase md:block"
      >
        {t('folders.section.label')}
      </p>
      {items.map(({ key, label, icon: Icon, badge }) => {
        const active = section === key
        return (
          <button
            key={key}
            type="button"
            aria-current={active ? 'page' : undefined}
            onClick={() => onChange(key)}
            title={label}
            className={`relative flex h-11 items-center rounded-md text-[15px] transition-colors md:gap-3 md:px-3 ${
              active
                ? 'bg-secondary text-secondary-foreground font-semibold'
                : 'text-muted-foreground hover:bg-secondary/70 hover:text-foreground font-medium'
            }`}
          >
            <Icon
              className={`mx-auto size-[18px] shrink-0 md:mx-0 ${active ? 'text-primary' : ''}`}
              strokeWidth={1.75}
            />
            <span className="sr-only md:not-sr-only">{label}</span>
            {badge > 0 && (
              <span className="bg-primary text-primary-foreground absolute top-0.5 right-0.5 inline-flex h-4 min-w-4 items-center justify-center rounded-full px-1 text-[10px] font-semibold tabular-nums md:static md:ml-auto md:h-5 md:min-w-5 md:px-1.5 md:text-xs">
                {badge}
              </span>
            )}
          </button>
        )
      })}
    </nav>
  )
}
