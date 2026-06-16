import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ArrowLeft, Building2, Clock, Eye, Pencil, RefreshCw, Trash2, User } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Input } from '@workspace/ui/components/input'
import { toast } from '@workspace/ui/components/sonner'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@workspace/ui/components/tabs'
import { DetailSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { InsolvencyStatus } from './status-badge'
import { ProceedingsList } from './proceedings-list'
import { ChangeTimeline } from './change-timeline'
import { formatRelative } from '@/lib/format'
import { checkSubject, removeSubject, setLabel } from '@/lib/api'
import { useDetail } from '@/hooks/use-detail'

interface SubjectDetailProps {
  uuid: string
  onBack: () => void
  onRemoved: () => void
  onChanged: () => void
}

export function SubjectDetail({ uuid, onBack, onRemoved, onChanged }: SubjectDetailProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useDetail(uuid)

  const [editing, setEditing] = useState(false)
  const [labelDraft, setLabelDraft] = useState('')
  const [confirmRemove, setConfirmRemove] = useState(false)
  const [refreshing, setRefreshing] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (editing) inputRef.current?.focus()
  }, [editing])

  if (loading && !data) return <DetailSkeleton />
  if (error) return <div className="p-6"><ErrorMessage message={error.message} onRetry={refetch} /></div>
  if (!data) return null

  const s = data.subject
  const hasHistory = s.rizeni_count > 0 && !s.ma_aktivni_insolvenci
  const Icon = s.kind === 'company' ? Building2 : User
  // The old ISIR detail deep-links are dead (register migrated to eISIR);
  // link to the live eISIR by IČO, which falls back to its search.
  const isirUrl =
    s.kind === 'company' && s.ico
      ? `https://eisir.justice.cz/insolvencni-rejstrik/dle-ico/${s.ico}`
      : 'https://eisir.justice.cz/insolvencni-rejstrik'
  const identifier =
    s.kind === 'company'
      ? s.ico
        ? `IČO ${s.ico}`
        : ''
      : [`${s.jmeno} ${s.prijmeni}`.trim(), s.datum_narozeni].filter(Boolean).join(' · ')

  const startEdit = () => {
    setLabelDraft(s.label)
    setEditing(true)
  }

  const saveLabel = () => {
    setEditing(false)
    if (labelDraft === s.label) return
    setLabel(uuid, labelDraft)
      .then(() => {
        refetch()
        onChanged()
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
  }

  const handleRefresh = () => {
    setRefreshing(true)
    checkSubject(uuid)
      .then((res) => {
        if (res.ok) {
          toast.success(t('detail.refreshed'))
        } else {
          toast.error(res.error ?? t('detail.refreshError'))
        }
        refetch()
        onChanged()
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setRefreshing(false))
  }

  const handleRemove = () => {
    if (!confirmRemove) {
      setConfirmRemove(true)
      setTimeout(() => setConfirmRemove(false), 3000)
      return
    }
    removeSubject(uuid)
      .then(() => {
        toast.success(t('detail.removed'))
        onRemoved()
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
  }

  return (
    <ScrollArea className="h-full">
      <div className="space-y-5 p-6">
        {/* Header */}
        <div className="flex items-start justify-between gap-3">
          <div className="flex min-w-0 items-start gap-3">
            <Button variant="ghost" size="icon" className="md:hidden" onClick={onBack}>
              <ArrowLeft className="size-4" />
            </Button>
            <div className="bg-secondary text-primary ring-brand-100 flex size-10 shrink-0 items-center justify-center rounded-xl ring-1">
              <Icon className="size-5" />
            </div>
            <div className="min-w-0">
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
                  placeholder={t('add.labelPlaceholder')}
                  className="h-8 w-56"
                  data-testid="label-edit-input"
                />
              ) : (
                <button
                  type="button"
                  onClick={startEdit}
                  className="hover:text-primary group font-display inline-flex items-center gap-1.5 text-left text-xl font-normal transition-colors"
                  data-testid="label-edit-trigger"
                >
                  <span className="truncate">{s.display_name}</span>
                  <Pencil className="text-muted-foreground size-3.5 opacity-0 transition-opacity group-hover:opacity-100" />
                </button>
              )}
              {identifier && <p className="text-muted-foreground text-sm">{identifier}</p>}

              {/* Inline, readable summary right under the identifier */}
              <div className="mt-2 flex flex-wrap items-center gap-x-2.5 gap-y-1 text-[13px]">
                <InsolvencyStatus active={s.ma_aktivni_insolvenci} hasHistory={hasHistory} />
                {s.last_check_at && (
                  <>
                    <Dot />
                    <span
                      className="text-muted-foreground inline-flex items-center gap-1"
                      title={t('detail.lastCheck')}
                    >
                      <Clock className="size-3.5" />
                      {formatRelative(s.last_check_at)}
                    </span>
                  </>
                )}
                {s.added_on && (
                  <>
                    <Dot />
                    <span
                      className="text-muted-foreground inline-flex items-center gap-1"
                      title={t('detail.addedOn')}
                    >
                      <Eye className="size-3.5" />
                      {formatRelative(s.added_on)}
                    </span>
                  </>
                )}
              </div>
            </div>
          </div>
          <div className="flex shrink-0 gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleRefresh}
              disabled={refreshing}
              data-testid="refresh-subject"
            >
              <RefreshCw className={`size-4 ${refreshing ? 'animate-spin' : ''}`} />
              <span className="hidden sm:inline">{t('detail.refresh')}</span>
            </Button>
            <Button
              variant={confirmRemove ? 'destructive' : 'outline'}
              size="sm"
              onClick={handleRemove}
              data-testid="remove-subject"
            >
              <Trash2 className="size-4" />
              <span className="hidden sm:inline">
                {confirmRemove ? t('detail.confirmRemove') : t('detail.remove')}
              </span>
            </Button>
          </div>
        </div>

        {/* Proceedings / change-history as a two-tab menu */}
        <Tabs defaultValue="rizeni" className="gap-4">
          <TabsList variant="line" className="w-full justify-start gap-4">
            <TabsTrigger value="rizeni" className="flex-none px-0">
              {t('detail.proceedings')}
              <CountChip>{s.rizeni_count}</CountChip>
            </TabsTrigger>
            <TabsTrigger value="zmeny" className="flex-none px-0">
              {t('detail.changeHistory')}
              {s.changes.length > 0 && <CountChip>{s.changes.length}</CountChip>}
            </TabsTrigger>
          </TabsList>
          <TabsContent value="rizeni">
            <ProceedingsList rizeni={s.rizeni} isirUrl={isirUrl} />
          </TabsContent>
          <TabsContent value="zmeny">
            <ChangeTimeline changes={s.changes} />
          </TabsContent>
        </Tabs>
      </div>
    </ScrollArea>
  )
}

function Dot() {
  return <span className="text-border">·</span>
}

function CountChip({ children }: { children: React.ReactNode }) {
  return (
    <span className="bg-secondary text-secondary-foreground ml-1.5 inline-flex min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-semibold">
      {children}
    </span>
  )
}
