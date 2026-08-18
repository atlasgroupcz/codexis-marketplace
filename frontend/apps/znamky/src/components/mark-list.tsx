import { useTranslation } from 'react-i18next'
import type { MarkSummary } from '@/lib/schemas'
import { MarkListItem } from './mark-list-item'

export function MarkList({
  marks,
  selectedUuid,
  onSelect,
}: {
  marks: Array<MarkSummary>
  selectedUuid: string | null
  onSelect: (uuid: string) => void
}) {
  const { t } = useTranslation()
  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="text-muted-foreground shrink-0 px-4 py-3 text-xs font-medium tracking-wide uppercase">
        {t('list.count', { count: marks.length })}
      </div>
      <div className="min-h-0 flex-1 space-y-1.5 overflow-y-auto px-3 pb-4">
        {marks.map((mark) => (
          <MarkListItem
            key={mark.uuid}
            mark={mark}
            selected={mark.uuid === selectedUuid}
            onSelect={onSelect}
          />
        ))}
      </div>
    </div>
  )
}
