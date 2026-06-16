import { useTranslation } from 'react-i18next'
import { Activity, FileText, ShieldAlert } from 'lucide-react'
import { Card, CardContent } from '@workspace/ui/components/card'
import { NoChangesArt } from './illustrations'
import type { SubjectChange } from '@/lib/schemas'
import { formatDateTime } from '@/lib/format'

function iconFor(typ: string) {
  if (typ === 'nove_rizeni') return <ShieldAlert className="size-4 text-red-600" />
  if (typ === 'nova_udalost') return <FileText className="size-4 text-blue-600" />
  return <Activity className="size-4 text-amber-600" />
}

/** Immutable, read-only log of detected changes (newest first). */
export function ChangeTimeline({ changes }: { changes: Array<SubjectChange> }) {
  const { t } = useTranslation()

  if (changes.length === 0) {
    return (
      <div className="flex flex-col items-center px-6 py-10 text-center">
        <NoChangesArt className="mb-3 h-28 w-auto" />
        <p className="text-foreground text-sm font-medium">{t('detail.noChanges')}</p>
        <p className="text-muted-foreground mt-1 max-w-xs text-xs text-pretty">
          {t('detail.noChangesHint')}
        </p>
      </div>
    )
  }

  const ordered = [...changes].sort((a, b) => (a.detected_on < b.detected_on ? 1 : -1))

  return (
    <ol className="relative space-y-3 border-l pl-5">
      {ordered.map((change, i) => (
        <li key={`${change.detected_on}-${i}`} className="relative">
          <span className="bg-background absolute top-0.5 -left-[26px] flex size-6 items-center justify-center rounded-full border">
            {iconFor(change.typ)}
          </span>
          <Card className="gap-0 py-0">
            <CardContent className="p-3">
              <time className="text-muted-foreground text-xs">
                {formatDateTime(change.detected_on)}
              </time>
              <p className="mt-1 text-sm">{change.popis}</p>
            </CardContent>
          </Card>
        </li>
      ))}
    </ol>
  )
}
