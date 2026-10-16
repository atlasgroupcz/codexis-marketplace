import type { ConnectionSectionLabels } from '@workspace/ui/components/connection-section'
import { ConnectionSection } from '@workspace/ui/components/connection-section'
import type { ConnState } from '@workspace/ui/components/connection-status'
import { connStateOf } from '@workspace/ui/components/connection-status'
import { GroupLabel } from '@workspace/ui/components/settings-dialog'
import { Skeleton } from '@workspace/ui/components/skeleton'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { KeyRound, Scale } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { StoredKey } from '@/components/stored-key'
import type { Settings } from '@/lib/api'
import { deleteApiKey, fetchSettings, saveApiKey } from '@/lib/api'

export function App() {
  const { t } = useTranslation()
  const [settings, setSettings] = useState<Settings | null>(null)
  const [loadFailed, setLoadFailed] = useState(false)

  const refetch = useCallback(async () => {
    try {
      setSettings(await fetchSettings())
      setLoadFailed(false)
    } catch {
      setLoadFailed(true)
    }
  }, [])

  useEffect(() => {
    void refetch()
  }, [refetch])

  const state: ConnState = connStateOf(settings?.configured ?? null)

  const labels: ConnectionSectionLabels = {
    connected: t('connection.connected'),
    notConnected: t('connection.notConnected'),
    remove: t('connection.remove'),
    confirmRemove: t('connection.confirmRemove'),
    save: t('connection.save'),
    saving: t('connection.saving'),
  }

  const getKey = {
    href: settings?.generateUrl ?? '',
    label: t('settings.getKey'),
    hint: t('settings.getKeyHint'),
  }

  const handleSave = async (values: Record<string, string>) => {
    await saveApiKey(values.apiKey ?? '')
    await refetch()
    toast.success(t('app.saved'))
  }

  const handleDelete = () => {
    deleteApiKey()
      .then(refetch)
      .then(() => toast.success(t('app.deleted')))
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : t('app.saveFailed')))
  }

  return (
    <div className="mx-auto max-w-xl space-y-6 p-6">
      <header className="flex items-center gap-4">
        <span className="bg-secondary text-secondary-foreground flex size-11 shrink-0 items-center justify-center rounded-xl">
          <Scale className="size-6" aria-hidden="true" />
        </span>
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-lg font-semibold tracking-tight">Turboleal</h1>
          {state === 'unknown' ? (
            <Skeleton className="mt-0.5 h-5 w-56" />
          ) : (
            <p className="text-muted-foreground mt-0.5 text-sm">
              {state === 'connected' ? t('app.leadConnected') : t('app.leadIdle')}
            </p>
          )}
        </div>
      </header>

      {loadFailed && (
        <p className="text-destructive text-sm" role="alert">
          {t('app.loadFailed')}
        </p>
      )}

      <section className="border-border rounded-xl border p-5">
        <GroupLabel icon={KeyRound} className="mb-4">
          {t('app.connectionSection')}
        </GroupLabel>
        {settings?.configured ? (
          <StoredKey
            maskedKey={settings.maskedKey}
            getKey={getKey}
            labels={{
              remove: t('connection.remove'),
              removeKey: t('connection.removeKey'),
              confirmRemove: t('connection.confirmRemove'),
            }}
            onDelete={handleDelete}
          />
        ) : (
          <ConnectionSection
            loading={settings === null}
            configured={false}
            maskedKey=""
            fields={[
              {
                name: 'apiKey',
                label: t('settings.label'),
                type: 'password',
                placeholder: t('settings.placeholder'),
                mono: true,
              },
            ]}
            getKey={getKey}
            labels={labels}
            onSave={handleSave}
            onDelete={handleDelete}
          />
        )}
      </section>

      {settings?.keySource && settings.keySource !== settings.settingsFile && (
        <p className="text-muted-foreground text-xs">
          {t('app.overridden', { source: settings.keySource })}
        </p>
      )}

      <Toaster />
    </div>
  )
}
