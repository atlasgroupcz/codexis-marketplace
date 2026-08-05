import { Badge } from '@workspace/ui/components/badge'
import { useTranslation } from 'react-i18next'
import type { TrackingType } from '@/lib/schemas'

const trackingTypeLabels: Record<
  TrackingType,
  | 'followedDocs.trackingType.all'
  | 'followedDocs.trackingType.documentChanges'
  | 'followedDocs.trackingType.relatedDocumentsChanges'
> = {
  all: 'followedDocs.trackingType.all',
  document_changes: 'followedDocs.trackingType.documentChanges',
  related_documents_changes: 'followedDocs.trackingType.relatedDocumentsChanges',
}

const trackingTypeVariants: Record<TrackingType, 'default' | 'secondary' | 'outline'> = {
  all: 'default',
  document_changes: 'secondary',
  related_documents_changes: 'outline',
}

interface TrackingTypeBadgeProps {
  type: TrackingType
}

export function TrackingTypeBadge({ type }: TrackingTypeBadgeProps) {
  const { t } = useTranslation()

  return (
    <Badge variant={trackingTypeVariants[type]}>
      {t(trackingTypeLabels[type])}
    </Badge>
  )
}
