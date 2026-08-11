import { Badge } from '@workspace/ui/components/badge'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@workspace/ui/components/card'
import { useTranslation } from 'react-i18next'
import { formatDate } from '@/lib/format'
import type { TrackedDocumentSummary } from '@/lib/schemas'
import { TrackingTypeBadge } from './tracking-type-badge'

interface DocumentCardProps {
  document: TrackedDocumentSummary
  onClick: (uuid: string) => void
}

export function DocumentCard({ document, onClick }: DocumentCardProps) {
  const { t } = useTranslation()

  return (
    <Card
      role="button"
      tabIndex={0}
      onClick={() => onClick(document.uuid)}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault()
          onClick(document.uuid)
        }
      }}
      className="hover:border-primary/40 hover:bg-accent/30 cursor-pointer gap-3 py-4 transition-colors"
    >
      <CardHeader className="gap-1 px-4">
        <CardTitle className="font-display text-base leading-snug">{document.name}</CardTitle>
        <CardDescription>
          {t('followedDocs.trackedSince', { date: formatDate(document.added_on) })}
        </CardDescription>
        {document.unconfirmed_changes > 0 && (
          <CardAction>
            <span className="bg-primary text-primary-foreground flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-semibold tabular-nums">
              {document.unconfirmed_changes}
            </span>
          </CardAction>
        )}
      </CardHeader>
      <CardContent className="flex flex-wrap items-center gap-1.5 px-4">
        <TrackingTypeBadge type={document.tracking_type} />
        <span className="text-muted-foreground text-xs">
          {t('followedDocs.total', { count: document.total_changes })}
        </span>
        {document.groups.map((g) => (
          <Badge key={g.id} variant="outline" className="py-0 text-xs">
            {g.name}
          </Badge>
        ))}
      </CardContent>
    </Card>
  )
}
