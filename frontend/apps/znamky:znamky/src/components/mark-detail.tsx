import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ArrowLeft, Check, RefreshCw, ShieldCheck, Trash2 } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { toast } from '@workspace/ui/components/sonner'
import { CollisionCard } from './collision-card'
import { DetailSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { useDetail } from '@/hooks/use-detail'
import { checkSubject, confirmChange, logoUrl, removeMark } from '@/lib/api'
import { formatRelative } from '@/lib/format'
import type { Collision } from '@/lib/schemas'

export function MarkDetail({
  uuid,
  onBack,
  onRemoved,
  onChanged,
}: {
  uuid: string
  onBack: () => void
  onRemoved: () => void
  onChanged: () => void
}) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useDetail(uuid)
  const [busy, setBusy] = useState(false)

  if (loading && !data) return <DetailSkeleton />
  if (error) return <div className="p-6"><ErrorMessage message={error.message} onRetry={refetch} /></div>
  if (!data) return null

  const mark = data.mark
  const collisions = mark.collisions
  const unconfirmed = collisions.filter((c) => !c.confirmed_on)
  const sorted: Collision[] = [
    ...unconfirmed,
    ...collisions.filter((c) => c.confirmed_on),
  ]

  const runCheck = () => {
    setBusy(true)
    checkSubject(uuid)
      .then((res) => {
        if (!res.ok) {
          toast.error(res.error ?? t('detail.refreshError'))
          return
        }
        const n = res.new_collisions ?? 0
        toast.success(n > 0 ? t('detail.refreshedNew', { count: n }) : t('detail.refreshedNone'))
        refetch()
        onChanged()
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setBusy(false))
  }

  const confirmOne = (index: number) => {
    confirmChange(uuid, index)
      .then(() => { refetch(); onChanged() })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
  }

  const confirmAllUnread = () => {
    confirmChange(uuid)
      .then(() => { refetch(); onChanged(); toast.success(t('detail.allMarked')) })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
  }

  const remove = () => {
    if (!window.confirm(t('detail.confirmRemove'))) return
    setBusy(true)
    removeMark(uuid)
      .then(() => { toast.success(t('detail.removed')); onRemoved() })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setBusy(false))
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-center gap-2 border-b px-4 py-3">
        <Button variant="ghost" size="icon" className="md:hidden" onClick={onBack} aria-label={t('common.back')}>
          <ArrowLeft className="size-5" />
        </Button>
        {mark.has_logo && (
          <img src={logoUrl(uuid)} alt="" className="bg-background size-9 rounded border object-contain p-0.5" />
        )}
        <div className="min-w-0 flex-1">
          <h2 className="font-display truncate text-lg leading-tight">{mark.display_name}</h2>
          <p className="text-muted-foreground truncate text-xs">
            {t(`kind.${mark.kind}`)} · {mark.territories.join('/')}
            {mark.nice_classes.length > 0 && <> · {t('list.classes')} {mark.nice_classes.join(', ')}</>}
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={runCheck} disabled={busy} data-testid="check-mark">
          <RefreshCw className={`size-4 ${busy ? 'animate-spin' : ''}`} />
          <span className="hidden sm:inline">{t('detail.refresh')}</span>
        </Button>
        <Button variant="ghost" size="icon" onClick={remove} disabled={busy} aria-label={t('detail.remove')}>
          <Trash2 className="size-4" />
        </Button>
      </div>

      <div className="min-h-0 flex-1 space-y-4 overflow-y-auto p-4 sm:p-6">
        <div className="text-muted-foreground grid grid-cols-2 gap-3 text-sm sm:grid-cols-4">
          <Stat label={t('detail.owner')} value={mark.owner_name || '—'} />
          <Stat label={t('detail.threshold')} value={`${Math.round(mark.threshold * 100)}%`} />
          <Stat label={t('detail.lastCheck')} value={formatRelative(mark.last_check_at)} />
          <Stat label={t('detail.collisionsCount')} value={String(mark.collisions_count)} />
        </div>

        <div className="flex items-center justify-between">
          <h3 className="font-display text-base">{t('detail.collisions')}</h3>
          {unconfirmed.length > 0 && (
            <Button variant="ghost" size="sm" onClick={confirmAllUnread} data-testid="mark-all-read">
              <Check className="size-4" />
              {t('detail.markAllRead', { count: unconfirmed.length })}
            </Button>
          )}
        </div>

        {sorted.length === 0 ? (
          <div className="text-muted-foreground flex flex-col items-center gap-3 rounded-xl border border-dashed p-10 text-center">
            <ShieldCheck className="text-primary size-8" />
            <p className="text-sm">{t('detail.noCollisions')}</p>
            <p className="max-w-xs text-xs">{t('detail.noCollisionsHint')}</p>
          </div>
        ) : (
          <div className="grid items-start gap-3 xl:grid-cols-2">
            {sorted.map((c) => {
              const realIndex = collisions.indexOf(c)
              return (
                <CollisionCard
                  key={`${c.source}:${c.source_id}:${realIndex}`}
                  collision={c}
                  index={realIndex}
                  markUuid={uuid}
                  markHasLogo={mark.has_logo}
                  onConfirm={confirmOne}
                />
              )
            })}
          </div>
        )}
      </div>
    </div>
  )
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-secondary/40 rounded-lg p-3">
      <dt className="text-muted-foreground text-xs">{label}</dt>
      <dd className="text-foreground mt-0.5 truncate text-sm font-medium">{value}</dd>
    </div>
  )
}
