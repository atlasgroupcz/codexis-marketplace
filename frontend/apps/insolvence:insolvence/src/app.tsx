import { useState } from 'react'
import { useQueryState } from 'nuqs'
import { useTranslation } from 'react-i18next'
import { Plus, RefreshCw, ShieldAlert } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { uuidParser } from '@/lib/url-state'
import { useOverview } from '@/hooks/use-overview'
import { checkAll } from '@/lib/api'
import { SubjectList } from '@/components/subject-list'
import { SubjectDetail } from '@/components/subject-detail'
import { AddSubjectDialog } from '@/components/add-subject-dialog'
import { EmptySubjects, NoSelection } from '@/components/empty-state'
import { DetailSkeleton, ListSkeleton } from '@/components/loading-skeleton'
import { ErrorMessage } from '@/components/error-message'

export function App() {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const [selectedUuid, setSelectedUuid] = useQueryState('uuid', uuidParser)
  const [addOpen, setAddOpen] = useState(false)
  const [checking, setChecking] = useState(false)
  const [detailNonce, setDetailNonce] = useState(0)

  const subjects = data?.subjects ?? []

  const handleCheckAll = () => {
    setChecking(true)
    checkAll()
      .then((res) => {
        if (res.errors.length > 0) {
          toast.error(t('app.checkErrors', { count: res.errors.length }))
        } else if (res.new_changes > 0) {
          toast.success(t('app.checkChanges', { count: res.new_changes }))
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
    <div className="bg-background text-foreground flex h-screen flex-col">
      <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b px-4">
        <div className="flex min-w-0 items-center gap-2.5">
          <span className="flex size-8 items-center justify-center rounded-lg bg-emerald-600/10 text-emerald-700 dark:text-emerald-400">
            <ShieldAlert className="size-5" />
          </span>
          <div className="min-w-0">
            <h1 className="truncate text-sm font-semibold">{t('app.title')}</h1>
            <p className="text-muted-foreground hidden text-xs sm:block">{t('app.subtitle')}</p>
          </div>
        </div>
        <div className="flex shrink-0 gap-2">
          {subjects.length > 0 && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleCheckAll}
              disabled={checking}
              data-testid="check-all"
            >
              <RefreshCw className={`size-4 ${checking ? 'animate-spin' : ''}`} />
              <span className="hidden sm:inline">{t('app.checkAll')}</span>
            </Button>
          )}
          <Button size="sm" onClick={() => setAddOpen(true)} data-testid="add-subject">
            <Plus className="size-4" />
            <span className="hidden sm:inline">{t('app.add')}</span>
          </Button>
        </div>
      </header>

      <main className="min-h-0 flex-1">
        {loading && !data ? (
          <div className="grid h-full min-h-0 md:grid-cols-[320px_1fr]">
            <div className="border-r">
              <ListSkeleton />
            </div>
            <div className="hidden md:block">
              <DetailSkeleton />
            </div>
          </div>
        ) : error ? (
          <div className="p-6">
            <ErrorMessage message={error.message} onRetry={refetch} />
          </div>
        ) : subjects.length === 0 ? (
          <EmptySubjects onAdd={() => setAddOpen(true)} />
        ) : (
          <div className="grid h-full min-h-0 md:grid-cols-[320px_1fr]">
            <div className={`${selectedUuid ? 'hidden md:block' : 'block'} min-h-0 border-r`}>
              <SubjectList
                subjects={subjects}
                selectedUuid={selectedUuid}
                onSelect={(uuid) => void setSelectedUuid(uuid)}
              />
            </div>
            <div className={`${selectedUuid ? 'block' : 'hidden md:block'} min-h-0`}>
              {selectedUuid ? (
                <SubjectDetail
                  key={`${selectedUuid}:${detailNonce}`}
                  uuid={selectedUuid}
                  onBack={() => void setSelectedUuid(null)}
                  onRemoved={() => {
                    void setSelectedUuid(null)
                    refetch()
                  }}
                  onChanged={refetch}
                />
              ) : (
                <NoSelection />
              )}
            </div>
          </div>
        )}
      </main>

      <AddSubjectDialog
        open={addOpen}
        onOpenChange={setAddOpen}
        onAdded={(uuid) => {
          refetch()
          if (uuid) void setSelectedUuid(uuid)
        }}
      />
      <Toaster position="bottom-right" />
    </div>
  )
}
