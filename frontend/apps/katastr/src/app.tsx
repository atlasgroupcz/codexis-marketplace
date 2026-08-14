import { AppHeader, HeaderGear } from '@workspace/ui/components/app-header'
import { Button } from '@workspace/ui/components/button'
import { ConfigGate } from '@workspace/ui/components/config-gate'
import type { ConnectionSectionLabels } from '@workspace/ui/components/connection-section'
import type { NotifyLabels } from '@workspace/ui/components/notify-settings'
import { SettingsDialog } from '@workspace/ui/components/settings-dialog'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { useNotify } from '@workspace/ui/hooks/use-notify'
import { KeyRound, Landmark, RefreshCw } from 'lucide-react'
import { useQueryState } from 'nuqs'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { AddProceedingForm } from '@/components/add-proceeding-form'
import { EmptyProceedings, NoSelection } from '@/components/empty-state'
import { ProceedingDetail } from '@/components/proceeding-detail'
import { ProceedingsList } from '@/components/proceedings-list'
import { useOverview } from '@/hooks/use-overview'
import { checkAll, deleteApiKey, fetchNotify, saveApiKey, saveNotify } from '@/lib/api'
import { CUZK_REGISTRATION_URL } from '@/lib/constants'
import { errorMessage } from '@/lib/format'
import { cisloRizeniParser, filterParser, viewParser } from '@/lib/url-state'

export function App() {
  const { t, i18n } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const { data: notifyData, refetch: refetchNotify } = useNotify(fetchNotify)
  const [view, setView] = useQueryState('view', viewParser)
  const [cisloRizeni, setCisloRizeni] = useQueryState('cislo_rizeni', cisloRizeniParser)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [checking, setChecking] = useState(false)
  const [savingNotify, setSavingNotify] = useState(false)
  const [filter, setFilter] = useQueryState('filter', filterParser)

  // null until the overview answers - the dialog renders it as neither
  // connected nor not, rather than asserting red on open.
  const apiKeyConfigured = data?.api_key_configured ?? null
  const hasProceedings = (data?.proceedings.length ?? 0) > 0
  const detailOpen = view === 'detail' && cisloRizeni !== null
  const notifyValues = notifyData?.settings ?? { email: false, inApp: true, recipients: [] }
  const userEmail = notifyData?.userEmail ?? ''

  const handleSelect = (next: string) => {
    void setView('detail')
    void setCisloRizeni(next)
  }
  const handleBack = () => {
    void setView('list')
    void setCisloRizeni(null)
  }
  const handleCheck = () => {
    setChecking(true)
    checkAll()
      .then((res) => {
        if (res.errors.length > 0) {
          toast.error(
            t('proceedings.checkResultErrors', {
              newChanges: res.new_changes,
              errors: res.errors.length,
            }),
          )
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
  const connectionLabels: ConnectionSectionLabels = {
    connected: t('connection.connected'),
    notConnected: t('connection.notConnected'),
    remove: t('connection.remove'),
    confirmRemove: t('connection.confirmRemove'),
    save: t('connection.save'),
    saving: t('connection.saving'),
  }

  const handleSaveNotify = (values: typeof notifyValues) => {
    setSavingNotify(true)
    saveNotify(values)
      .then(() => refetchNotify())
      .catch((e: unknown) => toast.error(errorMessage(e)))
      .finally(() => setSavingNotify(false))
  }
  const handleSaveApiKey = async (values: Record<string, string>) => {
    // Reject propagates to ConnectionSection, which shows the ČÚZK validation
    // error inline; success refetches so the masked key + connected state update.
    await saveApiKey(values.apiKey ?? '')
    refetch()
  }
  const handleDeleteApiKey = () => {
    deleteApiKey()
      .then(() => refetch())
      .catch((e: unknown) => toast.error(errorMessage(e)))
  }

  const renderMain = () => {
    if (data && !apiKeyConfigured) {
      return (
        <ConfigGate
          icon={KeyRound}
          title={t('apiKeyBanner.title')}
          text={t('apiKeyBanner.text')}
          action={{
            label: t('apiKeyBanner.action'),
            onClick: () => setSettingsOpen(true),
            testId: 'open-settings',
          }}
          getKey={{
            href: CUZK_REGISTRATION_URL,
            label: t('settings.getKey'),
            hint: t('settings.getKeyHint'),
          }}
        />
      )
    }
    if (data && apiKeyConfigured && !hasProceedings) {
      return <EmptyProceedings onAdded={refetch} />
    }
    return (
      <div className="grid h-full min-h-0 md:grid-cols-[300px_1fr]">
        <section
          className={`min-h-0 overflow-y-auto border-r ${detailOpen ? 'hidden md:block' : 'block'}`}
        >
          <ProceedingsList
            data={data}
            loading={loading}
            error={error}
            refetch={refetch}
            selectedCislo={cisloRizeni}
            onSelect={handleSelect}
            filter={filter}
            onFilterChange={setFilter}
          />
        </section>
        <section className={`min-h-0 overflow-y-auto ${detailOpen ? 'block' : 'hidden md:block'}`}>
          {detailOpen && cisloRizeni ? (
            <ProceedingDetail
              key={cisloRizeni}
              cisloRizeni={cisloRizeni}
              summary={data?.proceedings.find((p) => p.cislo_rizeni === cisloRizeni)}
              onBack={handleBack}
              onChanged={refetch}
            />
          ) : (
            <NoSelection />
          )}
        </section>
      </div>
    )
  }

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="app-root">
      <AppHeader
        brand={{
          icon: Landmark,
          title: t('proceedings.title'),
          subtitle: t('proceedings.subtitle'),
        }}
        docs={{ app: 'katastr', lang: i18n.language, label: t('help.docs') }}
        actions={
          <>
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
            <HeaderGear onClick={() => setSettingsOpen(true)} title={t('notify.title')} />
          </>
        }
      />

      <main className="min-h-0 flex-1">{renderMain()}</main>

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
        connection={{
          configured: apiKeyConfigured,
          sectionLabel: t('connection.section'),
          maskedKey: data?.api_key_masked ?? '',
          fields: [
            {
              name: 'apiKey',
              label: t('settings.newLabel'),
              type: 'password',
              placeholder: t('settings.placeholder'),
              mono: true,
              testId: 'api-key-input',
            },
          ],
          getKey: {
            href: CUZK_REGISTRATION_URL,
            label: t('settings.getKey'),
            hint: t('settings.getKeyHint'),
          },
          labels: connectionLabels,
          onSave: handleSaveApiKey,
          onDelete: handleDeleteApiKey,
        }}
      />
      <Toaster position="bottom-right" />
    </div>
  )
}
