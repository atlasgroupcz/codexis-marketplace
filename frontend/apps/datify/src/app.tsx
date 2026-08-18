import type { ConnectionSectionLabels } from '@workspace/ui/components/connection-section'
import { ConnectionSection } from '@workspace/ui/components/connection-section'
import type { ConnState } from '@workspace/ui/components/connection-status'
import { ConnectionStatus, connStateOf } from '@workspace/ui/components/connection-status'
import { GroupLabel } from '@workspace/ui/components/settings-dialog'
import { Skeleton } from '@workspace/ui/components/skeleton'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { KeyRound } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { Settings } from '@/lib/api'
import { deleteToken, fetchSettings, saveToken } from '@/lib/api'

/**
 * Datify's brand mark, its two shapes taken verbatim from their logo.svg.
 *
 * They sit either side of a gap there, and that gap is exactly what this page
 * is about - so the mark carries the answer: the bar slides away and both go
 * grey when no token is stored, and closes up in Datify blue once one is. The
 * wording beside it says the same thing, so nothing depends on noticing it.
 *
 * While unknown it stays closed and grey, so the gap only ever opens to report
 * a real answer instead of flicking open and shut as the first fetch lands.
 */
function DatifyMark({ state }: { state: ConnState }) {
  return (
    <svg
      viewBox="0 0 150 96.39"
      className="datify-mark h-11 w-auto shrink-0"
      data-state={state}
      aria-hidden="true"
    >
      <rect data-part="bar" width="19.88" height="96.39" />
      <path
        data-part="bracket"
        d="M134.28,18.07v60.25c0,9.98-8.09,18.07-18.07,18.07H39.79l-19.87-19.88h87.6c3.8,0,6.88-3.08,6.88-6.88V26.77c0-3.8-3.08-6.88-6.88-6.88h-32.67L55.11.14V0h61.11c9.98,0,18.07,8.09,18.07,18.07Z"
      />
    </svg>
  )
}

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

  // Three states, not two: `unknown` until the first fetch answers.
  const state: ConnState = connStateOf(settings?.configured ?? null)

  const labels: ConnectionSectionLabels = {
    connected: t('connection.connected'),
    notConnected: t('connection.notConnected'),
    remove: t('connection.remove'),
    confirmRemove: t('connection.confirmRemove'),
    save: t('connection.save'),
    saving: t('connection.saving'),
  }

  /**
   * Must reject on a token Datify refuses, so ConnectionSection renders the
   * reason inline instead of the page reporting a save that did not happen.
   */
  const handleSave = async (values: Record<string, string>) => {
    await saveToken(values.apiKey ?? '')
    await refetch()
    toast.success(t('app.saved'))
  }

  const handleDelete = () => {
    deleteToken()
      .then(refetch)
      .then(() => toast.success(t('app.deleted')))
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : t('app.saveFailed')))
  }

  return (
    <div className="mx-auto max-w-xl space-y-6 p-6">
      <header className="flex items-center gap-4">
        <DatifyMark state={state} />
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-lg font-semibold tracking-tight">Datify</h1>
          {/* Same rule as the status dot: leadIdle is the "no token yet"
              wording, so showing it before the answer arrives asserts a state
              nothing has checked. Reserve the line instead. */}
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
        {/* The dialog's own heading, minus its padding: there the breathing
            room above comes from a separator and the dialog's spacing, which
            inside this card would leave the label on the edge. */}
        <GroupLabel
          icon={KeyRound}
          className="mb-4"
          trailing={
            <ConnectionStatus
              state={state}
              labels={labels}
              // This panel's primary is Datify blue, so green has to be asked
              // for by name; elsewhere primary is the brand colour already.
              tone="success"
            />
          }
        >
          {t('app.connectionSection')}
        </GroupLabel>
        <ConnectionSection
          loading={settings === null}
          configured={settings?.configured ?? false}
          maskedKey={settings?.maskedKey ?? ''}
          fields={[
            {
              name: 'apiKey',
              label: t('settings.label'),
              type: 'password',
              placeholder: t('settings.placeholder'),
              mono: true,
            },
          ]}
          getKey={{
            href: settings?.generateUrl ?? '',
            label: t('settings.getKey'),
            hint: t('settings.getKeyHint'),
          }}
          labels={labels}
          onSave={handleSave}
          onDelete={handleDelete}
        />
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
