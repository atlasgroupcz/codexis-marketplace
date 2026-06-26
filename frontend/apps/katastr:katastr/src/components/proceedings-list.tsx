import { ListSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { ProceedingRow } from './proceeding-row'
import type { OverviewResponse } from '@/lib/schemas'

interface ProceedingsListProps {
  data: OverviewResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
  selectedUuid: string | null
  onSelect: (uuid: string) => void
}

export function ProceedingsList({
  data,
  loading,
  error,
  refetch,
  selectedUuid,
  onSelect,
}: ProceedingsListProps) {
  if (loading && !data) return <div className="p-3"><ListSkeleton /></div>
  if (error) return <div className="p-3"><ErrorMessage message={error.message} onRetry={refetch} /></div>
  if (!data) return null

  return (
    <div data-testid="proceedings-table">
      {data.proceedings.map((proc) => (
        <ProceedingRow
          key={proc.uuid}
          proceeding={proc}
          selected={proc.uuid === selectedUuid}
          onClick={onSelect}
        />
      ))}
    </div>
  )
}
