import { useState } from 'react'
import { useQueryState } from 'nuqs'
import { FileStack, RefreshCw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { AppHeader, HeaderGear } from '@workspace/ui/components/app-header'
import { Button } from '@workspace/ui/components/button'
import { SettingsDialog } from '@workspace/ui/components/settings-dialog'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { useNotify } from '@workspace/ui/hooks/use-notify'
import type { NotifyLabels } from '@workspace/ui/components/notify-settings'
import { filterParser, icoParser, viewParser } from '@/lib/url-state'
import { SubjektyList } from '@/components/subjekty-list'
import { SubjektDetail } from '@/components/subjekt-detail'
import { AddSubjektForm } from '@/components/add-subjekt-form'
import { EmptySubjekty, NoSelection } from '@/components/empty-state'
import { useOverview } from '@/hooks/use-overview'
import { checkAll, fetchNotify, saveNotify } from '@/lib/api'
import { errorMessage } from '@/lib/format'

export function App() {
  const { t, i18n } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const { data: notifyData, refetch: refetchNotify } = useNotify(fetchNotify)
  const [view, setView] = useQueryState('view', viewParser)
  const [ico, setIco] = useQueryState('ico', icoParser)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [checking, setChecking] = useState(false)
  const [savingNotify, setSavingNotify] = useState(false)
  const [filter, setFilter] = useQueryState('filter', filterParser)

  const hasSubjekty = (data?.subjekty.length ?? 0) > 0
  const detailOpen = view === 'detail' && ico !== null
  const notifyValues = notifyData?.settings ?? { email: false, inApp: true, recipients: [] }
  const userEmail = notifyData?.userEmail ?? ''

  const handleSelect = (next: string) => {
    void setView('detail')
    void setIco(next)
  }
  const handleBack = () => {
    void setView('list')
    void setIco(null)
  }
  const handleCheck = () => {
    setChecking(true)
    checkAll()
      .then((res) => {
        if (res.errors.length > 0) {
          toast.error(t('subjekty.checkResultErrors', { newChanges: res.new_changes, errors: res.errors.length }))
        } else if (res.new_changes > 0) {
          toast.success(t('subjekty.checkResultChanges', { count: res.new_changes }))
        } else {
          toast.success(t('subjekty.checkResultNone'))
        }
        refetch()
      })
      .catch((e: unknown) => toast.error(errorMessage(e)))
      .finally(() => setChecking(false))
  }

  const notifyLabels: NotifyLabels = {
    title: t('notify.title'),
    description: t('notify.description'),
    save: t('notify.save'),
    cancel: t('notify.cancel'),
    emailLabel: t('notify.emailLabel'),
    emailDescription: t('notify.emailDescription'),
    yourAddressLabel: t('notify.yourAddressLabel'),
    copyToLabel: t('notify.copyToLabel'),
    addRecipient: t('notify.addRecipient'),
    removeRecipient: t('notify.removeRecipient'),
    addPlaceholder: t('notify.addPlaceholder'),
    invalidEmail: t('notify.invalidEmail'),
    ownAddress: t('notify.ownAddress'),
    duplicate: t('notify.duplicate'),
    inAppLabel: t('notify.inAppLabel'),
    inAppDescription: t('notify.inAppDescription'),
  }

  const handleSaveNotify = (values: typeof notifyValues) => {
    setSavingNotify(true)
    saveNotify(values)
      .then(() => refetchNotify())
      .catch((e: unknown) => toast.error(errorMessage(e)))
      .finally(() => setSavingNotify(false))
  }

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="app-root">
      <AppHeader
        brand={{ icon: FileStack, title: t('subjekty.title'), subtitle: t('subjekty.subtitle') }}
        docs={{ app: 'sbirka-listin', lang: i18n.language, label: t('help.docs') }}
        actions={
          <>
            {hasSubjekty && (
              <div className="hidden sm:block">
                <AddSubjektForm onAdded={refetch} />
              </div>
            )}
            <Button
              variant="outline"
              size="sm"
              onClick={handleCheck}
              disabled={checking}
              data-testid="check-all-button"
            >
              <RefreshCw className={`size-4 ${checking ? 'animate-spin' : ''}`} />
              <span className="hidden sm:inline">{t('subjekty.checkAll')}</span>
            </Button>
            <HeaderGear onClick={() => setSettingsOpen(true)} title={t('notify.title')} />
          </>
        }
      />

      <main className="min-h-0 flex-1">
        {data && !hasSubjekty ? (
          <EmptySubjekty onAdded={refetch} />
        ) : (
          <div className="grid h-full min-h-0 md:grid-cols-[320px_1fr]">
            <section
              className={`bg-muted/40 min-h-0 overflow-y-auto border-r ${detailOpen ? 'hidden md:block' : 'block'}`}
            >
              <SubjektyList
                data={data}
                loading={loading}
                error={error}
                refetch={refetch}
                selectedIco={ico}
                onSelect={handleSelect}
                filter={filter}
                onFilterChange={setFilter}
              />
            </section>
            <section className={`min-h-0 overflow-y-auto ${detailOpen ? 'block' : 'hidden md:block'}`}>
              {detailOpen && ico ? (
                <SubjektDetail key={ico} ico={ico} onBack={handleBack} onChanged={refetch} />
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
        notify={{
          values: notifyValues,
          userEmail,
          labels: notifyLabels,
          sectionLabel: t('notify.section'),
          saving: savingNotify,
          onSave: handleSaveNotify,
        }}
      />
      <Toaster position="bottom-right" />
    </div>
  )
}
