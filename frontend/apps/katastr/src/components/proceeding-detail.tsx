import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ArrowLeft, Landmark, Pencil, Trash2 } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Input } from '@workspace/ui/components/input'
import { toast } from '@workspace/ui/components/sonner'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@workspace/ui/components/tabs'
import { useConfirmAction } from '@workspace/ui/hooks/use-confirm-action'
import { ErrorMessage } from '@workspace/ui/components/error-message'
import { ListSkeleton } from './loading-skeleton'
import { StavBadge } from './stav-badge'
import { UhradaBadge } from './uhrada-badge'
import { TypeLabel } from './type-label'
import { ProceedingTimeline } from './detail/timeline'
import { ChangeHistory } from './detail/change-history'
import { useDetail } from '@/hooks/use-detail'
import { removeProceeding, setLabel } from '@/lib/api'
import { errorMessage, formatDate, formatRelative } from '@/lib/format'

interface ProceedingDetailProps {
  cisloRizeni: string
  onBack: () => void
  onChanged?: () => void
}

export function ProceedingDetail({ cisloRizeni, onBack, onChanged }: ProceedingDetailProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useDetail(cisloRizeni)

  const [labelOverride, setLabelOverride] = useState<string | null>(null)
  const [editing, setEditing] = useState(false)
  const [labelDraft, setLabelDraft] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)

  const remove = useConfirmAction(() =>
    removeProceeding(cisloRizeni)
      .then(() => {
        onChanged?.()
        onBack()
      })
      .catch((e: unknown) => toast.error(errorMessage(e))),
  )

  useEffect(() => {
    if (editing) {
      inputRef.current?.focus()
      inputRef.current?.select()
    }
  }, [editing])

  // Drop the optimistic label once the server confirms it via refetched data.
  useEffect(() => {
    if (labelOverride !== null && data?.proceeding.label === labelOverride) {
      setLabelOverride(null)
    }
  }, [data?.proceeding.label, labelOverride])

  if (loading && !data) return <ListSkeleton />
  if (error) return <div className="p-6"><ErrorMessage message={error.message} onRetry={refetch} retryLabel={t('common.retry')} /></div>
  if (!data) return null

  const proc = data.proceeding
  const currentLabel = labelOverride !== null ? labelOverride : proc.label

  const startEdit = () => {
    setLabelDraft(currentLabel)
    setEditing(true)
  }

  const saveLabel = () => {
    const next = labelDraft.trim()
    setEditing(false)
    if (next === currentLabel) return
    // Optimistic display first; persist, then refetch so the list reflects the
    // new label instead of racing the write.
    setLabelOverride(next)
    setLabel(cisloRizeni, next)
      .then(() => {
        refetch()
        onChanged?.()
      })
      .catch((e: unknown) => {
        toast.error(errorMessage(e))
        setLabelOverride(null) // rollback to server value
      })
  }

  const sep = <span className="text-border">·</span>

  return (
    <div className="space-y-5 p-4 sm:p-6">
      {/* Header */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex min-w-0 items-start gap-3">
          <Button variant="ghost" size="icon" className="md:hidden" onClick={onBack} data-testid="back-to-list">
            <ArrowLeft className="size-4" />
          </Button>
          <span className="bg-secondary text-primary ring-brand-100 flex size-10 shrink-0 items-center justify-center rounded-xl ring-1">
            <Landmark className="size-5" strokeWidth={1.75} />
          </span>
          <div className="min-w-0">
            <TypeLabel cisloRizeni={proc.cislo_rizeni} className="mb-0.5" />
            <h1 className="truncate font-mono text-xl font-semibold leading-tight">
              {proc.cislo_rizeni}
            </h1>
            {editing ? (
              <Input
                ref={inputRef}
                value={labelDraft}
                onChange={(e) => setLabelDraft(e.target.value)}
                onBlur={saveLabel}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') saveLabel()
                  if (e.key === 'Escape') setEditing(false)
                }}
                placeholder={t('proceedings.labelPlaceholder')}
                className="mt-1 h-7 w-56"
                data-testid="label-input"
              />
            ) : (
              <button
                type="button"
                onClick={startEdit}
                className="text-muted-foreground hover:text-foreground group mt-0.5 inline-flex items-center gap-1 text-sm transition-colors"
                data-testid="label-edit-trigger"
              >
                {currentLabel ? (
                  <span className="truncate">{currentLabel}</span>
                ) : (
                  <span className="italic">{t('proceedings.addLabelHint')}</span>
                )}
                <Pencil className="size-3 opacity-0 transition-opacity group-hover:opacity-100" />
              </button>
            )}

            {/* Inline, scannable summary */}
            <div className="text-muted-foreground mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 text-[13px]">
              <StavBadge stav={proc.stav} />
              <UhradaBadge stavUhrady={proc.stav_uhrady} label={proc.stav_uhrady_label} />
              {sep}
              <span>
                {t('proceedings.datumPrijeti')}{' '}
                <span className="text-foreground">{formatDate(proc.datum_prijeti)}</span>
              </span>
              {sep}
              <span>
                {t('proceedings.lastCheck')}{' '}
                <span className="text-foreground">{formatRelative(proc.last_check_at)}</span>
              </span>
            </div>
          </div>
        </div>
        <Button
          variant={remove.confirming ? 'destructive' : 'outline'}
          size="sm"
          onClick={remove.trigger}
          disabled={remove.pending}
          data-testid="remove-proceeding"
          className="shrink-0"
        >
          <Trash2 className="size-4" />
          <span className="hidden sm:inline">
            {remove.confirming ? t('proceedings.confirmRemove') : t('proceedings.remove')}
          </span>
        </Button>
      </div>

      {/* Průběh / Historie změn */}
      <Tabs defaultValue="prubeh" className="gap-4">
        <TabsList variant="line" className="w-full justify-start gap-4">
          <TabsTrigger value="prubeh" className="flex-none px-0" data-testid="tab-prubeh">
            {t('proceedings.timeline.title')}
            <CountChip>{proc.operace_count}</CountChip>
          </TabsTrigger>
          <TabsTrigger value="zmeny" className="flex-none px-0" data-testid="tab-zmeny">
            {t('proceedings.changeHistory')}
            {proc.changes.length > 0 && <CountChip>{proc.changes.length}</CountChip>}
          </TabsTrigger>
        </TabsList>
        <TabsContent value="prubeh">
          <ProceedingTimeline proc={proc} />
        </TabsContent>
        <TabsContent value="zmeny">
          <ChangeHistory
            cisloRizeni={cisloRizeni}
            changes={proc.changes}
            onConfirmed={() => {
              refetch()
              onChanged?.()
            }}
          />
        </TabsContent>
      </Tabs>
    </div>
  )
}

function CountChip({ children }: { children: React.ReactNode }) {
  return (
    <span className="bg-secondary text-secondary-foreground ml-1.5 inline-flex min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-semibold">
      {children}
    </span>
  )
}
