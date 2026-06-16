import { useTranslation } from 'react-i18next'
import { Check, ExternalLink, Sparkles } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { RiskBadge } from './risk-badge'
import { SimilarityDonut } from './similarity-donut'
import { candidateLogoUrl, logoUrl } from '@/lib/api'
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
  markUuid,
  markHasLogo,
  onConfirm,
}: {
  collision: Collision
  index: number
  markUuid: string
  markHasLogo: boolean
  onConfirm: (index: number) => void
}) {
  const { t } = useTranslation()
  const unconfirmed = !collision.confirmed_on
  const ai = collision.ai_assessment
  const showLogos = markHasLogo || Boolean(collision.image_path)

  return (
    <div
      data-testid="collision-card"
      className={`rounded-xl border p-4 transition-colors ${
        unconfirmed ? 'border-primary/30 bg-secondary/40' : 'bg-card'
      }`}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <span className="font-display truncate text-base">
              {collision.mark_text || t('collision.figurativeMark')}
            </span>
            <RiskBadge tier={collision.tier} />
            <span className="bg-muted text-muted-foreground rounded px-1.5 py-0.5 text-xs">
              {collision.office || collision.territory}
            </span>
          </div>
          <p className="text-muted-foreground mt-0.5 truncate text-sm">
            {collision.applicant || t('collision.unknownApplicant')}
          </p>
          <p className="text-muted-foreground mt-0.5 text-xs">
            {collision.status && <span>{collision.status} · </span>}
            {collision.filing_date && <span>{t('collision.filed')} {formatDate(collision.filing_date)} · </span>}
            {collision.nice_classes.length > 0 && (
              <span>{t('list.classes')} {collision.nice_classes.join(', ')}</span>
            )}
          </p>
        </div>
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

      {showLogos && (
        <div className="mt-3 flex items-center gap-3">
          {markHasLogo && (
            <figure className="text-center">
              <img
                src={logoUrl(markUuid)}
                alt=""
                className="bg-background size-16 rounded border object-contain p-1"
              />
              <figcaption className="text-muted-foreground mt-1 text-[10px]">
                {t('collision.yours')}
              </figcaption>
            </figure>
          )}
          {collision.image_path && (
            <figure className="text-center">
              <img
                src={candidateLogoUrl(markUuid, index)}
                alt=""
                className="bg-background size-16 rounded border object-contain p-1"
              />
              <figcaption className="text-muted-foreground mt-1 text-[10px]">
                {t('collision.candidate')}
              </figcaption>
            </figure>
          )}
        </div>
      )}

      <div className="mt-3">
        <SimilarityDonut scores={collision.scores} tier={collision.tier} />
      </div>

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

      <div className="mt-3 flex items-center justify-between">
        {collision.url_detail ? (
          <a
            href={collision.url_detail}
            target="_blank"
            rel="noreferrer"
            className="text-primary inline-flex items-center gap-1 text-xs hover:underline"
          >
            <ExternalLink className="size-3.5" />
            {t('collision.openSource')}
          </a>
        ) : (
          <span />
        )}
        {!unconfirmed && (
          <span className="text-muted-foreground text-xs">{t('collision.reviewed')}</span>
        )}
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
