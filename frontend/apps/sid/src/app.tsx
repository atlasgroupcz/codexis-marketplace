import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Bell } from 'lucide-react'
import { HeaderDocsLink } from '@workspace/ui/components/app-header'
import { ConnectionSection } from '@workspace/ui/components/connection-section'
import { Button } from '@workspace/ui/components/button'
import { Switch } from '@workspace/ui/components/switch'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import type { ConnectionSectionLabels } from '@workspace/ui/components/connection-section'
import type { NotifySettings, StatusResponse } from '@/lib/schemas'
import { deleteApiKey, fetchStatus, saveApiKey, saveNotifySettings } from '@/lib/api'

/**
 * The bell is the only channel this panel offers. The peers (znamky,
 * katastr, ...) watch registries that notify nobody on their own and so
 * ship an e-mail switch too; SID mails its users itself, with better
 * messages than a digest of counts could be. Hence no
 * `NotifySettingsBody` here - that shared component exists to edit the
 * e-mail channel.
 */
const EMPTY_NOTIFY: NotifySettings = { inApp: true }

export function App() {
  const { t, i18n } = useTranslation()
  const [status, setStatus] = useState<StatusResponse | null>(null)
  const [notify, setNotify] = useState<NotifySettings>(EMPTY_NOTIFY)
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
        <div className="flex items-center justify-between gap-3 rounded-lg px-3 py-2.5">
          <span className="flex items-start gap-2.5">
            <Bell className="text-muted-foreground mt-0.5 size-4" />
            <span>
              <span className="text-sm font-medium">{t('notify.inAppLabel')}</span>
              <br />
              <span className="text-muted-foreground text-xs">
                {t('notify.inAppDescription')}
              </span>
            </span>
          </span>
          <Switch
            aria-label={t('notify.inAppLabel')}
            checked={notify.inApp}
            onCheckedChange={(checked) => {
              setNotify({ inApp: checked })
              setNotifyDirty(true)
            }}
          />
        </div>
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
