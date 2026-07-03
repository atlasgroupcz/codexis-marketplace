import { useTranslation } from 'react-i18next'
import { ListSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { ProceedingRow } from './proceeding-row'
import { ListFilter } from './list-filter'
import type { ListFilterValue } from './list-filter'
import type { OverviewResponse } from '@/lib/schemas'

interface ProceedingsListProps {
  data: OverviewResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
  selectedUuid: string | null
  onSelect: (uuid: string) => void
  filter: ListFilterValue
  onFilterChange: (value: ListFilterValue) => void
}

export function ProceedingsList({
  data,
  loading,
  error,
  refetch,
  selectedUuid,
  onSelect,
  filter,
  onFilterChange,
}: ProceedingsListProps) {
  const { t } = useTranslation()
  if (loading && !data) return <div className="p-3"><ListSkeleton /></div>
  if (error) return <div className="p-3"><ErrorMessage message={error.message} onRetry={refetch} /></div>
  if (!data) return null

  const unconfirmed = data.proceedings.filter((p) => p.unconfirmed_count > 0)
  const unconfirmedCount = unconfirmed.length
  const visible = filter === 'unconfirmed' ? unconfirmed : data.proceedings

  return (
    <div data-testid="proceedings-table">
      <ListFilter value={filter} onChange={onFilterChange} unconfirmedCount={unconfirmedCount} />
      {filter === 'unconfirmed' && visible.length === 0 ? (
        <p
          data-testid="no-unconfirmed"
          className="text-muted-foreground px-3 py-8 text-center text-sm"
        >
          {t('proceedings.noUnconfirmed')}
        </p>
      ) : (
        visible.map((proc) => (
          <ProceedingRow
            key={proc.uuid}
            proceeding={proc}
            selected={proc.uuid === selectedUuid}
            onClick={onSelect}
          />
        ))
      )}
    </div>
  )
}
