import { useTranslation } from 'react-i18next'

export type ListFilterValue = 'all' | 'unconfirmed'

interface ListFilterProps {
  value: ListFilterValue
  onChange: (value: ListFilterValue) => void
  unconfirmedCount: number
}

function segClass(active: boolean): string {
  return `flex h-7 items-center gap-1.5 rounded-md px-2.5 text-[12.5px] font-semibold outline-none transition-colors focus-visible:ring-[3px] focus-visible:ring-ring/50 ${
    active ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground'
  }`
}

export function ListFilter({ value, onChange, unconfirmedCount }: ListFilterProps) {
  const { t } = useTranslation()
  return (
    <div
      data-testid="list-filter"
      className="bg-background sticky top-0 z-10 flex items-center gap-1.5 border-b px-2.5 py-2"
    >
      <button
        type="button"
        aria-pressed={value === 'all'}
        data-testid="filter-all"
        onClick={() => onChange('all')}
        className={segClass(value === 'all')}
      >
        {t('proceedings.filterAll')}
      </button>
      <button
        type="button"
        aria-pressed={value === 'unconfirmed'}
        data-testid="filter-unconfirmed"
        onClick={() => onChange('unconfirmed')}
        className={segClass(value === 'unconfirmed')}
      >
        {t('proceedings.filterUnconfirmed')}
        <span
          data-testid="filter-unconfirmed-count"
          className="bg-secondary text-secondary-foreground inline-flex h-[18px] min-w-[18px] items-center justify-center rounded-full px-1 text-[11px] font-bold"
        >
          {unconfirmedCount}
        </span>
      </button>
    </div>
  )
}
