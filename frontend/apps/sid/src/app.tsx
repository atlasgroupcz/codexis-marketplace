import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { HeaderDocsLink } from '@workspace/ui/components/app-header'
import { ConnectionSection } from '@workspace/ui/components/connection-section'
import { NotifySettingsBody } from '@workspace/ui/components/notify-settings-body'
import { Button } from '@workspace/ui/components/button'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import type { ConnectionSectionLabels } from '@workspace/ui/components/connection-section'
import type { NotifyBodyLabels, NotifyValues } from '@workspace/ui/components/notify-settings-body'
import type { StatusResponse } from '@/lib/schemas'
import { deleteApiKey, fetchStatus, saveApiKey, saveNotifySettings } from '@/lib/api'

const EMPTY_NOTIFY: NotifyValues = { email: false, inApp: true, recipients: [] }

export function App() {
  const { t, i18n } = useTranslation()
  const [status, setStatus] = useState<StatusResponse | null>(null)
  const [notify, setNotify] = useState<NotifyValues>(EMPTY_NOTIFY)
  const [savingNotify, setSavingNotify] = useState(false)
  const [loadFailed, setLoadFailed] = useState(false)
  // Set once the user edits the notification form. Saving the API key also
  // refetches, and without this those edits would be silently reverted.
  const [notifyDirty, setNotifyDirty] = useState(false)

  const refetch = useCallback(
    async (resetNotify = true) => {
      try {
        const next = await fetchStatus()
        setStatus(next)
        if (resetNotify) {
          setNotify(next.notify)
          setNotifyDirty(false)
        }
        setLoadFailed(false)
      } catch {
        setLoadFailed(true)
      }
    },
    [],
  )

  useEffect(() => {
    void refetch()
  }, [refetch])

  const connectionLabels: ConnectionSectionLabels = {
    connected: t('connection.connected'),
    notConnected: t('connection.notConnected'),
    remove: t('connection.remove'),
    confirmRemove: t('connection.confirmRemove'),
    save: t('connection.save'),
    saving: t('connection.saving'),
  }

  const notifyLabels: NotifyBodyLabels = {
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

  /**
   * Must reject on a rejected key so ConnectionSection renders the error
   * inline; on success the refetch flips the section to its connected state.
   */
  const handleSaveKey = async (values: Record<string, string>) => {
    const result = await saveApiKey(values.apiKey ?? '')
    if (!result.ok) {
      throw new Error(result.error ?? t('app.saveFailed'))
    }
    await refetch(!notifyDirty)
  }

  const handleDeleteKey = () => {
    deleteApiKey()
      .then(() => refetch(!notifyDirty))
      .then(() => toast.success(t('app.keyDeleted')))
      .catch(() => toast.error(t('app.saveFailed')))
  }

  const handleSaveNotify = () => {
    setSavingNotify(true)
    saveNotifySettings(notify)
      .then((result) => {
        if (!result.ok) throw new Error(result.error ?? t('app.saveFailed'))
        return refetch()
      })
      .then(() => toast.success(t('app.saved')))
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : t('app.saveFailed')))
      .finally(() => setSavingNotify(false))
  }

  return (
    <div className="mx-auto max-w-2xl space-y-8 p-6">
      <header className="space-y-1">
        <div className="flex min-w-0 items-baseline gap-2.5">
          <h1 className="truncate text-xl font-semibold">{t('app.title')}</h1>
          <span className="shrink-0">
            <HeaderDocsLink app="sid" lang={i18n.language} label={t('help.docs')} />
          </span>
        </div>
        <p className="text-muted-foreground text-sm">{t('app.lead')}</p>
      </header>

      {loadFailed && <p className="text-destructive text-sm">{t('app.loadFailed')}</p>}

      <section className="space-y-3">
        <h2 className="text-sm font-semibold">{t('app.connectionSection')}</h2>
        <ConnectionSection
          loading={status === null}
          configured={status?.configured ?? false}
          maskedKey={status?.start ? `${status.start}…` : ''}
          fields={[
            {
              name: 'apiKey',
              label: t('settings.newLabel'),
              type: 'password',
              placeholder: t('settings.placeholder'),
              mono: true,
            },
          ]}
          getKey={{
            href: `${status?.baseUrl ?? ''}/api-klice`,
            label: t('settings.getKey'),
            hint: t('settings.getKeyHint'),
          }}
          labels={connectionLabels}
          onSave={handleSaveKey}
          onDelete={handleDeleteKey}
        />
      </section>

      <section className="space-y-3">
        <h2 className="text-sm font-semibold">{t('app.notifySection')}</h2>
        <NotifySettingsBody
          value={notify}
          onChange={(next) => {
            setNotify(next)
            setNotifyDirty(true)
          }}
          userEmail={status?.userEmail ?? ''}
          labels={notifyLabels}
        />
        <div className="flex justify-end">
          <Button onClick={handleSaveNotify} disabled={savingNotify || status === null}>
            {t('notify.save')}
          </Button>
        </div>
      </section>

      <Toaster />
    </div>
  )
}
