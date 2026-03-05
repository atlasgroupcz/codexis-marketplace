import { Badge } from '@workspace/ui/components/badge'
import { useTranslation } from 'react-i18next'
import type { ChangeType } from '@/lib/schemas'

const changeTypeLabels: Record<
  ChangeType,
  'followedDocs.changeType.documentChange' | 'followedDocs.changeType.relatedChange'
> = {
  document_change: 'followedDocs.changeType.documentChange',
  related_change: 'followedDocs.changeType.relatedChange',
}

const changeTypeVariants: Record<ChangeType, 'default' | 'secondary'> = {
  document_change: 'default',
  related_change: 'secondary',
}

interface ChangeTypeBadgeProps {
  type: ChangeType
}

export function ChangeTypeBadge({ type }: ChangeTypeBadgeProps) {
  const { t } = useTranslation()

  return (
    <Badge variant={changeTypeVariants[type]}>
      {t(changeTypeLabels[type])}
    </Badge>
  )
}
