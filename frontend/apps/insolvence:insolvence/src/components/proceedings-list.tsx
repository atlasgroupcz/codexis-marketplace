import { useTranslation } from 'react-i18next'
import { ExternalLink } from 'lucide-react'
import { DruhBadge } from './status-badge'
import { NoProceedingsArt } from './illustrations'
import type { Rizeni } from '@/lib/schemas'
import { formatDate } from '@/lib/format'

const ISIR_PORTAL = 'https://isir.justice.cz/isir/common/index.do'

export function ProceedingsList({ rizeni }: { rizeni: Array<Rizeni> }) {
  const { t } = useTranslation()

  if (rizeni.length === 0) {
    return (
      <div className="bg-card border-border flex flex-col items-center rounded-xl border px-6 py-8 text-center">
        <NoProceedingsArt className="mb-4 h-32 w-auto" />
        <p className="text-foreground text-sm font-medium">{t('detail.noProceedings')}</p>
        <p className="text-muted-foreground mt-1 max-w-xs text-xs text-pretty">
          {t('detail.noProceedingsHint')}
        </p>
      </div>
    )
  }

  return (
    <div className="space-y-3">
      {rizeni.map((r, i) => (
        <div
          key={`${r.spisova_znacka}-${i}`}
          className={`rounded-lg border p-4 ${
            r.aktivni ? 'border-l-4 border-l-red-500' : ''
          }`}
        >
          <div className="flex flex-wrap items-center justify-between gap-2">
            <span className="font-mono text-sm font-semibold">{r.spisova_znacka}</span>
            <DruhBadge druh={r.druh} />
          </div>
          {r.soud && <p className="text-muted-foreground mt-1 text-sm">{r.soud}</p>}
          <dl className="mt-3 grid grid-cols-2 gap-x-4 gap-y-2 text-sm sm:grid-cols-3">
            <Field label={t('detail.stav')} value={r.stav || '—'} />
            <Field label={t('detail.datumZahajeni')} value={formatDate(r.datum_zahajeni)} />
            {r.datum_ukonceni && (
              <Field label={t('detail.datumUkonceni')} value={formatDate(r.datum_ukonceni)} />
            )}
          </dl>
          <a
            href={r.url_detail || ISIR_PORTAL}
            target="_blank"
            rel="noreferrer noopener"
            className="text-primary mt-3 inline-flex items-center gap-1.5 text-xs font-medium hover:underline"
          >
            <ExternalLink className="size-3.5" />
            {t('detail.openInIsir')}
          </a>
        </div>
      ))}
    </div>
  )
}

function Field({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-muted-foreground text-xs font-medium">{label}</dt>
      <dd className="mt-0.5">{value}</dd>
    </div>
  )
}
