import { useTranslation } from 'react-i18next'
import { Check, ExternalLink, Sparkles } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { SimilarityDonut } from './similarity-donut'
import { LogoZoom } from './logo-zoom'
import { formatDate } from '@/lib/format'
import type { Collision } from '@/lib/schemas'

const RISK_CLASSES: Record<string, string> = {
  high: 'bg-red-100 text-red-800 dark:bg-red-950/50 dark:text-red-200',
  medium: 'bg-amber-100 text-amber-900 dark:bg-amber-950/50 dark:text-amber-200',
  low: 'bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300',
  none: 'bg-emerald-100 text-emerald-800 dark:bg-emerald-950/50 dark:text-emerald-200',
}

export function CollisionCard({
  collision,
  index,
  markLogoDataUrl,
  onConfirm,
}: {
  collision: Collision
  index: number
  markLogoDataUrl: string | null
  onConfirm: (index: number) => void
}) {
  const { t } = useTranslation()
  const unconfirmed = !collision.confirmed_on
  const s = collision.scores
  const ai = collision.ai_assessment
  const showLogos = Boolean(markLogoDataUrl) || Boolean(collision.image_data_url)

  const breakdown: Array<{ key: string; value: number }> = [
    { key: 'text_combined', value: s.text_combined },
    { key: 'text_phonetic', value: s.text_phonetic },
    { key: 'class_overlap', value: s.class_overlap },
    { key: 'vienna_overlap', value: s.vienna_overlap },
  ]
  if (s.image_phash !== null) breakdown.push({ key: 'image_phash', value: s.image_phash })

  const meta: string[] = []
  if (collision.filing_date) meta.push(formatDate(collision.filing_date))
  if (collision.nice_classes.length > 0) meta.push(`${t('list.classes')} ${collision.nice_classes.join(', ')}`)

  return (
    <div
      data-testid="collision-card"
      className={`rounded-xl border p-4 transition-colors ${
        unconfirmed ? 'border-primary/30 bg-secondary/40' : 'bg-card'
      }`}
    >
      <div className="flex gap-4">
        <div className="flex size-[160px] shrink-0 items-center justify-center self-center">
          <SimilarityDonut overall={s.overall} tier={collision.tier} />
        </div>

        <div className="min-w-0 flex-1">
          <div className="flex items-start justify-between gap-3">
            <h4 className="font-display min-w-0 truncate text-lg">
              {collision.mark_text || t('collision.figurativeMark')}
            </h4>
            {unconfirmed && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => onConfirm(index)}
                data-testid="confirm-collision"
                className="shrink-0"
              >
                <Check className="size-4" />
                <span className="hidden sm:inline">{t('collision.confirm')}</span>
              </Button>
            )}
          </div>

          <p className="text-foreground/80 mt-0.5 truncate text-sm">
            {collision.applicant || t('collision.unknownApplicant')}
          </p>
          {meta.length > 0 && (
            <p className="text-muted-foreground mt-0.5 text-xs">{meta.join(' · ')}</p>
          )}

          {showLogos && (
            <div className="mt-3 flex items-center gap-3">
              {markLogoDataUrl && (
                <figure className="text-center">
                  <LogoZoom src={markLogoDataUrl} className="bg-background size-14 rounded border object-contain p-1" />
                  <figcaption className="text-muted-foreground mt-1 text-[10px]">{t('collision.yours')}</figcaption>
                </figure>
              )}
              {collision.image_data_url && (
                <figure className="text-center">
                  <LogoZoom src={collision.image_data_url} className="bg-background size-14 rounded border object-contain p-1" />
                  <figcaption className="text-muted-foreground mt-1 text-[10px]">{t('collision.candidate')}</figcaption>
                </figure>
              )}
            </div>
          )}

          <dl className="mt-3 space-y-1.5">
            {breakdown.map((b) => {
              const v = Math.round(b.value * 100)
              return (
                <div key={b.key} className="flex items-center gap-2 text-xs">
                  <dt className="text-muted-foreground w-24 shrink-0 truncate">{t(`score.${b.key}`)}</dt>
                  <div className="bg-muted h-1.5 w-16 shrink-0 overflow-hidden rounded-full">
                    <div className="bg-primary/60 h-full rounded-full" style={{ width: `${v}%` }} />
                  </div>
                  <dd className="text-foreground/70 w-8 shrink-0 text-right font-mono tabular-nums">{v}%</dd>
                </div>
              )
            })}
          </dl>

          {ai && (
            <div className="border-border/60 mt-3 rounded-lg border border-dashed p-3" data-testid="ai-assessment">
              <div className="flex items-center gap-2">
                <Sparkles className="text-primary size-4" />
                <span className="text-sm font-medium">{t('collision.aiTitle')}</span>
                <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${RISK_CLASSES[ai.risk] ?? ''}`}>
                  {t(`risk.${ai.risk}`)}
                </span>
              </div>
              {ai.summary && <p className="mt-2 text-sm">{ai.summary}</p>}
              <dl className="text-muted-foreground mt-2 space-y-1 text-xs">
                {ai.visual && <Row label={t('collision.visual')} value={ai.visual} />}
                {ai.aural && <Row label={t('collision.aural')} value={ai.aural} />}
                {ai.conceptual && <Row label={t('collision.conceptual')} value={ai.conceptual} />}
                {ai.goods_similarity && <Row label={t('collision.goods')} value={ai.goods_similarity} />}
              </dl>
            </div>
          )}

          {collision.url_detail && (
            <a
              href={collision.url_detail}
              target="_blank"
              rel="noreferrer"
              className="text-primary mt-3 inline-flex items-center gap-1 text-xs hover:underline"
            >
              <ExternalLink className="size-3.5" />
              {t('collision.openSource')}
            </a>
          )}
        </div>
      </div>
    </div>
  )
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex gap-1.5">
      <dt className="text-foreground/70 shrink-0 font-medium">{label}:</dt>
      <dd>{value}</dd>
    </div>
  )
}
