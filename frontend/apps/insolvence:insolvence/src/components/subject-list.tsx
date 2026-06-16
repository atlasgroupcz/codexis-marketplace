import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { SubjectListItem } from './subject-list-item'
import type { SubjectSummary } from '@/lib/schemas'

interface SubjectListProps {
  subjects: Array<SubjectSummary>
  selectedUuid: string | null
  onSelect: (uuid: string) => void
}

export function SubjectList({ subjects, selectedUuid, onSelect }: SubjectListProps) {
  return (
    <ScrollArea className="h-full min-h-0">
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
  )
}
