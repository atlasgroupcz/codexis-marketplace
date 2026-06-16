import { useTranslation } from 'react-i18next'
import { Building2, User } from 'lucide-react'
import { StatusDot } from './status-badge'
import type { SubjectSummary } from '@/lib/schemas'

interface SubjectListItemProps {
  subject: SubjectSummary
  selected: boolean
  onSelect: (uuid: string) => void
}

export function SubjectListItem({ subject, selected, onSelect }: SubjectListItemProps) {
  const { t } = useTranslation()
  const hasHistory = subject.rizeni_count > 0 && !subject.ma_aktivni_insolvenci
  const Icon = subject.kind === 'company' ? Building2 : User
  const subtitle =
    subject.kind === 'company'
      ? subject.ico
        ? `IČO ${subject.ico}`
        : ''
      : subject.datum_narozeni
        ? t('list.bornOn', { date: subject.datum_narozeni })
        : ''

  return (
    <button
      type="button"
      onClick={() => onSelect(subject.uuid)}
      data-testid="subject-list-item"
      aria-current={selected}
      className={`group flex w-full items-center gap-3 rounded-lg border px-3 py-2.5 text-left transition-colors ${
        selected
          ? 'border-primary/40 bg-accent'
          : 'hover:bg-accent/50 border-transparent'
      }`}
    >
      <StatusDot active={subject.ma_aktivni_insolvenci} hasHistory={hasHistory} />
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          <Icon className="text-muted-foreground size-3.5 shrink-0" />
          <span className="truncate text-sm font-medium">{subject.display_name}</span>
        </div>
        {subtitle && (
          <span className="text-muted-foreground block truncate text-xs">{subtitle}</span>
        )}
      </div>
      {subject.unconfirmed_count > 0 && (
        <span
          className="inline-flex min-w-5 shrink-0 items-center justify-center rounded-full bg-red-600 px-1.5 text-xs font-semibold text-white dark:bg-red-500"
          title={t('list.unreadChanges', { count: subject.unconfirmed_count })}
        >
          {subject.unconfirmed_count}
        </span>
      )}
    </button>
  )
}
