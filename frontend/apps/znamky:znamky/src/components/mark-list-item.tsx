import { useTranslation } from 'react-i18next'
import { Image as ImageIcon, Layers, Type } from 'lucide-react'
import { RiskDot } from './risk-badge'
import type { MarkSummary } from '@/lib/schemas'

const KIND_ICON = { word: Type, figurative: ImageIcon, combined: Layers }

export function MarkListItem({
  mark,
  selected,
  onSelect,
}: {
  mark: MarkSummary
  selected: boolean
  onSelect: (uuid: string) => void
}) {
  const { t } = useTranslation()
  const Icon = KIND_ICON[mark.kind]

  return (
    <button
      type="button"
      onClick={() => onSelect(mark.uuid)}
      data-testid="mark-list-item"
      aria-current={selected}
      className={`group flex w-full items-center gap-3 rounded-lg border px-3 py-2.5 text-left transition-colors ${
        selected ? 'border-primary/40 bg-accent' : 'hover:bg-accent/50 border-transparent'
      }`}
    >
      <RiskDot tier={mark.worst_tier} />
      {mark.logo_data_url ? (
        <img
          src={mark.logo_data_url}
          alt=""
          className="bg-background size-8 shrink-0 rounded border object-contain p-0.5"
        />
      ) : (
        <span className="bg-secondary text-primary flex size-8 shrink-0 items-center justify-center rounded">
          <Icon className="size-4" strokeWidth={1.75} />
        </span>
      )}
      <div className="min-w-0 flex-1">
        <span className="block truncate text-sm font-medium">{mark.display_name}</span>
        <div className="text-muted-foreground mt-0.5 flex items-center gap-1.5 text-xs">
          <span>{mark.territories.join('/') || '-'}</span>
          {mark.nice_classes.length > 0 && (
            <span className="truncate">· {t('list.classes')} {mark.nice_classes.join(', ')}</span>
          )}
        </div>
      </div>
      {mark.unconfirmed_count > 0 && (
        <span
          className="bg-primary text-primary-foreground inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-semibold"
          data-testid="unconfirmed-badge"
        >
          {mark.unconfirmed_count}
        </span>
      )}
    </button>
  )
}
