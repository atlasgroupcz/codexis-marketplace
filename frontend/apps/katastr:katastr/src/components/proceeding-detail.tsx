import { useEffect, useState } from 'react'
import { ListSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { DetailHeader } from './detail/detail-header'
import { StatusGrid } from './detail/status-grid'
import { OperationsTable } from './detail/operations-table'
import { ChangeHistory } from './detail/change-history'
import { useDetail } from '@/hooks/use-detail'

interface ProceedingDetailProps {
  uuid: string
  onBack: () => void
}

export function ProceedingDetail({ uuid, onBack }: ProceedingDetailProps) {
  const { data, loading, error, refetch } = useDetail(uuid)
  const [labelOverride, setLabelOverride] = useState<string | null>(null)

  // Reset optimistic override once the server confirms it via refetched data.
  useEffect(() => {
    if (
      labelOverride !== null &&
      data?.proceeding.label === labelOverride
    ) {
      setLabelOverride(null)
    }
  }, [data?.proceeding.label, labelOverride])

  if (loading && !data) return <ListSkeleton />
  if (error) return <ErrorMessage message={error.message} onRetry={refetch} />
  if (!data) return null

  const proc = data.proceeding
  const currentLabel = labelOverride !== null ? labelOverride : proc.label

  return (
    <div className="space-y-6 p-6">
      <DetailHeader
        uuid={uuid}
        cisloRizeni={proc.cislo_rizeni}
        label={currentLabel}
        onBack={onBack}
        onLabelChanged={(next) => {
          setLabelOverride(next)
          refetch()
        }}
        onDeleted={onBack}
      />
      <StatusGrid proc={proc} />
      <OperationsTable operations={proc.provedene_operace} />
      <ChangeHistory uuid={uuid} changes={proc.changes} onConfirmed={refetch} />
    </div>
  )
}
