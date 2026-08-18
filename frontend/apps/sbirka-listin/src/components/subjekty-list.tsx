import { ErrorMessage } from '@workspace/ui/components/error-message'
import { useTranslation } from 'react-i18next'
import type { OverviewResponse } from '@/lib/schemas'
import type { ListFilterValue } from './list-filter'
import { ListFilter } from './list-filter'
import { ListSkeleton } from './loading-skeleton'
import { SubjektRow } from './subjekt-row'

interface SubjektyListProps {
  data: OverviewResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
  selectedIco: string | null
  onSelect: (ico: string) => void
  filter: ListFilterValue
  onFilterChange: (value: ListFilterValue) => void
}

export function SubjektyList({
  data,
  loading,
  error,
  refetch,
  selectedIco,
  onSelect,
  filter,
  onFilterChange,
}: SubjektyListProps) {
  const { t } = useTranslation()
  if (loading && !data) return <ListSkeleton />
  if (error)
    return (
      <div className="p-3">
        <ErrorMessage message={error.message} onRetry={refetch} retryLabel={t('common.retry')} />
      </div>
    )
  if (!data) return null

  const unconfirmed = data.subjekty.filter((s) => s.nepotvrzenych_zmen > 0)
  const unconfirmedCount = unconfirmed.length
  const visible = filter === 'unconfirmed' ? unconfirmed : data.subjekty

  return (
    <div data-testid="subjekty-table">
      <ListFilter value={filter} onChange={onFilterChange} unconfirmedCount={unconfirmedCount} />
      {filter === 'unconfirmed' && visible.length === 0 ? (
        <p
          data-testid="no-unconfirmed"
          className="text-muted-foreground px-3 py-8 text-center text-sm"
        >
          {t('subjekty.noUnconfirmed')}
        </p>
      ) : (
        visible.map((subjekt) => (
          <SubjektRow
            key={subjekt.ico}
            subjekt={subjekt}
            selected={subjekt.ico === selectedIco}
            onClick={onSelect}
          />
        ))
      )}
    </div>
  )
}
