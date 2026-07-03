import { useState } from 'react'
import { useQueryState } from 'nuqs'
import { Landmark, RefreshCw, Settings as SettingsIcon } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@workspace/ui/components/button'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { filterParser, uuidParser, viewParser } from '@/lib/url-state'
import { ProceedingsList } from '@/components/proceedings-list'
import { ProceedingDetail } from '@/components/proceeding-detail'
import { AddProceedingForm } from '@/components/add-proceeding-form'
import { EmptyProceedings, NoSelection } from '@/components/empty-state'
import { ConfigSetup } from '@/components/config-setup'
import { SettingsDialog } from '@/components/settings-dialog'
import { useOverview } from '@/hooks/use-overview'
import { checkAll } from '@/lib/api'
import { errorMessage } from '@/lib/format'

export function App() {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const [view, setView] = useQueryState('view', viewParser)
  const [uuid, setUuid] = useQueryState('uuid', uuidParser)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [checking, setChecking] = useState(false)
  const [filter, setFilter] = useQueryState('filter', filterParser)

  const apiKeyConfigured = data?.api_key_configured ?? false
  const hasProceedings = (data?.proceedings.length ?? 0) > 0
  const detailOpen = view === 'detail' && uuid !== null

  const handleSelect = (next: string) => {
    void setView('detail')
    void setUuid(next)
  }
  const handleBack = () => {
    void setView('list')
    void setUuid(null)
  }
  const handleCheck = () => {
    setChecking(true)
    checkAll()
      .then((res) => {
        if (res.errors.length > 0) {
          toast.error(t('proceedings.checkResultErrors', { newChanges: res.new_changes, errors: res.errors.length }))
        } else if (res.new_changes > 0) {
          toast.success(t('proceedings.checkResultChanges', { count: res.new_changes }))
        } else {
          toast.success(t('proceedings.checkResultNone'))
        }
        refetch()
      })
      .catch((e: unknown) => toast.error(errorMessage(e)))
      .finally(() => setChecking(false))
  }

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="app-root">
      <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b px-4">
        <div className="flex min-w-0 items-center gap-2.5">
          <span className="bg-secondary text-primary ring-brand-100 flex size-9 items-center justify-center rounded-xl ring-1">
            <Landmark className="size-5" strokeWidth={1.75} />
          </span>
          <div className="min-w-0">
            <h1 className="font-display text-foreground truncate text-[15px] leading-tight">
              {t('proceedings.title')}
            </h1>
            <p className="text-muted-foreground hidden truncate text-xs sm:block">
              {t('proceedings.subtitle')}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {apiKeyConfigured && hasProceedings && (
            <div className="hidden sm:block">
              <AddProceedingForm onAdded={refetch} />
            </div>
          )}
          {apiKeyConfigured && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleCheck}
              disabled={checking}
              data-testid="check-all-button"
            >
              <RefreshCw className={`size-4 ${checking ? 'animate-spin' : ''}`} />
              <span className="hidden sm:inline">{t('proceedings.checkAll')}</span>
            </Button>
          )}
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setSettingsOpen(true)}
            title={t('settings.title')}
            data-testid="open-settings"
          >
            <SettingsIcon className="size-4" />
          </Button>
        </div>
      </header>

      <main className="min-h-0 flex-1">
        {data && !apiKeyConfigured ? (
          <ConfigSetup onConfigure={() => setSettingsOpen(true)} />
        ) : data && apiKeyConfigured && !hasProceedings ? (
          <EmptyProceedings onAdded={refetch} />
        ) : (
          <div className="grid h-full min-h-0 md:grid-cols-[300px_1fr]">
            <section
              className={`min-h-0 overflow-y-auto border-r ${detailOpen ? 'hidden md:block' : 'block'}`}
            >
              <ProceedingsList
                data={data}
                loading={loading}
                error={error}
                refetch={refetch}
                selectedUuid={uuid}
                onSelect={handleSelect}
                filter={filter}
                onFilterChange={setFilter}
              />
            </section>
            <section className={`min-h-0 overflow-y-auto ${detailOpen ? 'block' : 'hidden md:block'}`}>
              {detailOpen && uuid ? (
                <ProceedingDetail key={uuid} uuid={uuid} onBack={handleBack} onChanged={refetch} />
              ) : (
                <NoSelection />
              )}
            </section>
          </div>
        )}
      </main>

      <SettingsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        configured={data?.api_key_configured ?? false}
        maskedKey={data?.api_key_masked ?? ''}
        onSaved={refetch}
      />
      <Toaster position="bottom-right" />
    </div>
  )
}
