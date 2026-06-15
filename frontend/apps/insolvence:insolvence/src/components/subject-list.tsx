import { useTranslation } from 'react-i18next'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { SubjectListItem } from './subject-list-item'
import type { SubjectSummary } from '@/lib/schemas'

interface SubjectListProps {
  subjects: Array<SubjectSummary>
  selectedUuid: string | null
  onSelect: (uuid: string) => void
}

export function SubjectList({ subjects, selectedUuid, onSelect }: SubjectListProps) {
  const { t } = useTranslation()
  const active = subjects.filter((s) => s.ma_aktivni_insolvenci).length
  const unread = subjects.reduce((acc, s) => acc + s.unconfirmed_count, 0)

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="text-muted-foreground border-b px-4 py-2.5 text-xs">
        <span className="text-foreground font-medium">
          {t('list.count', { count: subjects.length })}
        </span>
        {active > 0 && (
          <span className="text-red-600 dark:text-red-400">
            {' · '}
            {t('list.activeCount', { count: active })}
          </span>
        )}
        {unread > 0 && <span>{` · ${t('list.unreadCount', { count: unread })}`}</span>}
      </div>
      <ScrollArea className="min-h-0 flex-1">
        <div className="space-y-1 p-2">
          {subjects.map((s) => (
            <SubjectListItem
              key={s.uuid}
              subject={s}
              selected={s.uuid === selectedUuid}
              onSelect={onSelect}
            />
          ))}
        </div>
      </ScrollArea>
    </div>
  )
}
