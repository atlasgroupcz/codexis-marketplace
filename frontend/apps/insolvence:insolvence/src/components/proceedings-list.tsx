import { useTranslation } from 'react-i18next'
import { ExternalLink, Scale } from 'lucide-react'
import { DruhBadge } from './status-badge'
import type { Rizeni } from '@/lib/schemas'
import { formatDate } from '@/lib/format'

export function ProceedingsList({ rizeni }: { rizeni: Array<Rizeni> }) {
  const { t } = useTranslation()

  if (rizeni.length === 0) {
    return (
      <div className="flex flex-col items-center gap-2 rounded-lg border border-emerald-200 bg-emerald-50 p-6 text-center dark:border-emerald-900 dark:bg-emerald-950/30">
        <Scale className="size-6 text-emerald-600 dark:text-emerald-400" />
        <p className="text-sm font-medium text-emerald-800 dark:text-emerald-200">
          {t('detail.noProceedings')}
        </p>
        <p className="text-muted-foreground text-xs">{t('detail.noProceedingsHint')}</p>
      </div>
    )
  }

  return (
    <div className="space-y-3">
      <h2 className="text-sm font-semibold">{t('detail.proceedings')}</h2>
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
          {r.url_detail && (
            <a
              href={r.url_detail}
              target="_blank"
              rel="noreferrer noopener"
              className="text-primary mt-3 inline-flex items-center gap-1.5 text-xs font-medium hover:underline"
            >
              <ExternalLink className="size-3.5" />
              {t('detail.openInIsir')}
            </a>
          )}
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
