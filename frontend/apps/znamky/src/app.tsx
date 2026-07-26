import { useState } from 'react'
import { useQueryState } from 'nuqs'
import { useTranslation } from 'react-i18next'
import { KeyRound, Plus, RefreshCw, ShieldCheck } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { AppHeader, HeaderGear } from '@workspace/ui/components/app-header'
import { ConfigGate } from '@workspace/ui/components/config-gate'
import { SettingsDialog } from '@workspace/ui/components/settings-dialog'
import { useFetch } from '@workspace/ui/hooks/use-fetch'
import type { NotifyValues } from '@workspace/ui/components/notify-settings-body'
import { uuidParser } from '@/lib/url-state'
import { useOverview } from '@/hooks/use-overview'
import { checkAll, deleteConfig, fetchConfig, fetchNotify, saveConfig, saveNotify } from '@/lib/api'
import { MarkList } from '@/components/mark-list'
import { MarkDetail } from '@/components/mark-detail'
import { AddMarkDialog } from '@/components/add-mark-dialog'
import { EmptyMarks, NoSelection } from '@/components/empty-state'
import { DetailSkeleton, ListSkeleton } from '@/components/loading-skeleton'
import { ErrorMessage } from '@/components/error-message'

const EUIPO_GET_KEY_URL = 'https://dev.euipo.europa.eu'

export function App() {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const { data: config, refetch: refetchConfig } = useFetch(fetchConfig)
  const { data: notify, refetch: refetchNotify } = useFetch(fetchNotify)
  const [selectedUuid, setSelectedUuid] = useQueryState('uuid', uuidParser)
  const [addOpen, setAddOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [checking, setChecking] = useState(false)
  const [detailNonce, setDetailNonce] = useState(0)
  const [notifySaving, setNotifySaving] = useState(false)

  const marks = data?.marks ?? []
  const configured = config?.configured ?? null

  const handleSaveNotify = (values: NotifyValues) => {
    setNotifySaving(true)
    saveNotify(values)
      .then((res) => {
        if (!res.ok) {
          toast.error(res.error ?? t('settings.saveError'))
          return
        }
        refetchNotify()
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setNotifySaving(false))
  }

  const handleSaveConnection = async (values: Record<string, string>) => {
    // Reject propagates to ConnectionSection, which shows the error inline;
    // success refetches so the masked key + connected state update.
    const res = await saveConfig(values.clientId ?? '', values.clientSecret ?? '')
    if (!res.ok) throw new Error(res.error ?? t('settings.saveError'))
    refetchConfig()
    refetch()
  }

  const handleDeleteConnection = () => {
    deleteConfig()
      .then((res) => {
        if (!res.ok) {
          toast.error(res.error ?? t('settings.saveError'))
          return
        }
        refetchConfig()
        refetch()
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
  }

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
      <AppHeader
        brand={{ icon: ShieldCheck, title: t('app.title'), subtitle: t('app.subtitle') }}
        actions={
          <>
            {configured && marks.length > 0 && (
              <Button variant="outline" size="sm" onClick={handleCheckAll} disabled={checking} data-testid="check-all">
                <RefreshCw className={`size-4 ${checking ? 'animate-spin' : ''}`} />
                <span className="hidden sm:inline">{t('app.checkAll')}</span>
              </Button>
            )}
            {configured && (
              <Button size="sm" onClick={() => setAddOpen(true)} data-testid="add-mark">
                <Plus className="size-4" />
                <span className="hidden sm:inline">{t('app.add')}</span>
              </Button>
            )}
            <HeaderGear onClick={() => setSettingsOpen(true)} title={t('notify.title')} />
          </>
        }
      />

      <main className="min-h-0 flex-1">
        {configured === null ? (
          <div className="grid h-full min-h-0 md:grid-cols-[320px_1fr]">
            <div className="border-r"><ListSkeleton /></div>
            <div className="hidden md:block"><DetailSkeleton /></div>
          </div>
        ) : !configured ? (
          <ConfigGate
            icon={KeyRound}
            eyebrow={t('app.title')}
            title={t('config.title')}
            text={t('config.text')}
            action={{ label: t('config.action'), onClick: () => setSettingsOpen(true), testId: 'open-settings' }}
            getKey={{ href: EUIPO_GET_KEY_URL, label: t('settings.getKey'), hint: t('config.hint') }}
          />
        ) : loading && !data ? (
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
      <SettingsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        notify={{
          values: notify?.settings ?? { email: false, inApp: false, recipients: [] },
          userEmail: notify?.userEmail ?? '',
          labels: {
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
          },
          sectionLabel: t('notify.section'),
          saving: notifySaving,
          onSave: handleSaveNotify,
        }}
        connection={{
          configured: config?.configured ?? false,
          maskedKey: config?.maskedKey ?? '',
          sectionLabel: t('connection.section'),
          fields: [
            { name: 'clientId', label: t('settings.clientId'), type: 'text', mono: true, testId: 'client-id-input' },
            { name: 'clientSecret', label: t('settings.clientSecret'), type: 'password', mono: true, testId: 'client-secret-input' },
          ],
          getKey: { href: EUIPO_GET_KEY_URL, label: t('settings.getKey'), hint: t('config.hint') },
          labels: {
            connected: t('connection.connected'),
            notConnected: t('connection.notConnected'),
            remove: t('connection.remove'),
            confirmRemove: t('connection.confirmRemove'),
            save: t('connection.save'),
            saving: t('connection.saving'),
          },
          onSave: handleSaveConnection,
          onDelete: handleDeleteConnection,
        }}
      />
      <Toaster position="bottom-right" />
    </div>
  )
}
