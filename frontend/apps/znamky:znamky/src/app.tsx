import { useState } from 'react'
import { useQueryState } from 'nuqs'
import { useTranslation } from 'react-i18next'
import { Plus, RefreshCw, ShieldCheck } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { uuidParser } from '@/lib/url-state'
import { useOverview } from '@/hooks/use-overview'
import { checkAll } from '@/lib/api'
import { MarkList } from '@/components/mark-list'
import { MarkDetail } from '@/components/mark-detail'
import { AddMarkDialog } from '@/components/add-mark-dialog'
import { EmptyMarks, NoSelection } from '@/components/empty-state'
import { DetailSkeleton, ListSkeleton } from '@/components/loading-skeleton'
import { ErrorMessage } from '@/components/error-message'

export function App() {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const [selectedUuid, setSelectedUuid] = useQueryState('uuid', uuidParser)
  const [addOpen, setAddOpen] = useState(false)
  const [checking, setChecking] = useState(false)
  const [detailNonce, setDetailNonce] = useState(0)

  const marks = data?.marks ?? []

  const handleCheckAll = () => {
    setChecking(true)
    checkAll()
      .then((res) => {
        if (res.errors.length > 0) {
          toast.error(t('app.checkErrors', { count: res.errors.length }))
        } else if (res.new_collisions > 0) {
          toast.success(t('app.checkChanges', { count: res.new_collisions }))
        } else {
          toast.success(t('app.checkNone'))
        }
        refetch()
        setDetailNonce((n) => n + 1)
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setChecking(false))
  }

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="app-root">
      <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b px-4">
        <div className="flex min-w-0 items-center gap-2.5">
          <span className="bg-secondary text-primary ring-brand-100 flex size-9 items-center justify-center rounded-xl ring-1">
            <ShieldCheck className="size-5" strokeWidth={1.75} />
          </span>
          <div className="min-w-0">
            <h1 className="font-display text-foreground truncate text-[15px] leading-tight">
              {t('app.title')}
            </h1>
            <p className="text-muted-foreground hidden truncate text-xs sm:block">
              {t('app.subtitle')}
            </p>
          </div>
        </div>
        <div className="flex shrink-0 gap-2">
          {marks.length > 0 && (
            <Button variant="outline" size="sm" onClick={handleCheckAll} disabled={checking} data-testid="check-all">
              <RefreshCw className={`size-4 ${checking ? 'animate-spin' : ''}`} />
              <span className="hidden sm:inline">{t('app.checkAll')}</span>
            </Button>
          )}
          <Button size="sm" onClick={() => setAddOpen(true)} data-testid="add-mark">
            <Plus className="size-4" />
            <span className="hidden sm:inline">{t('app.add')}</span>
          </Button>
        </div>
      </header>

      <main className="min-h-0 flex-1">
        {loading && !data ? (
          <div className="grid h-full min-h-0 md:grid-cols-[320px_1fr]">
            <div className="border-r"><ListSkeleton /></div>
            <div className="hidden md:block"><DetailSkeleton /></div>
          </div>
        ) : error ? (
          <div className="p-6"><ErrorMessage message={error.message} onRetry={refetch} /></div>
        ) : marks.length === 0 ? (
          <EmptyMarks onAdd={() => setAddOpen(true)} />
        ) : (
          <div className="grid h-full min-h-0 md:grid-cols-[320px_1fr]">
            <div className={`${selectedUuid ? 'hidden md:block' : 'block'} min-h-0 border-r`}>
              <MarkList marks={marks} selectedUuid={selectedUuid} onSelect={(uuid) => void setSelectedUuid(uuid)} />
            </div>
            <div className={`${selectedUuid ? 'block' : 'hidden md:block'} min-h-0`}>
              {selectedUuid ? (
                <MarkDetail
                  key={`${selectedUuid}:${detailNonce}`}
                  uuid={selectedUuid}
                  onBack={() => void setSelectedUuid(null)}
                  onRemoved={() => { void setSelectedUuid(null); refetch() }}
                  onChanged={refetch}
                />
              ) : (
                <NoSelection />
              )}
            </div>
          </div>
        )}
      </main>

      <AddMarkDialog
        open={addOpen}
        onOpenChange={setAddOpen}
        onAdded={(uuid) => { refetch(); if (uuid) void setSelectedUuid(uuid) }}
      />
      <Toaster position="bottom-right" />
    </div>
  )
}
