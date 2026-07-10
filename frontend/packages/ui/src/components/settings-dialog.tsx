import { useEffect, useState } from 'react'
import { Bell, KeyRound } from 'lucide-react'
import { ConnectionSection } from '@workspace/ui/components/connection-section'
import { NotifySettingsBody } from '@workspace/ui/components/notify-settings-body'
import { Separator } from '@workspace/ui/components/separator'
import { SettingsDialogShell } from '@workspace/ui/components/settings-dialog-shell'
import type { ConnField, ConnGetKeyLink } from '@workspace/ui/components/connection-settings'
import type { ConnectionSectionLabels } from '@workspace/ui/components/connection-section'
import type { NotifyLabels } from '@workspace/ui/components/notify-settings'
import type { NotifyValues } from '@workspace/ui/components/notify-settings-body'
import type { ComponentType, ReactNode } from 'react'

export interface SettingsDialogNotifyProps {
  values: NotifyValues
  userEmail: string
  labels: NotifyLabels
  sectionLabel?: string
  saving?: boolean
  onSave: (values: NotifyValues) => void
}
export interface SettingsDialogConnectionProps {
  configured: boolean
  maskedKey: string
  fields: Array<ConnField>
  getKey: ConnGetKeyLink
  labels: ConnectionSectionLabels
  sectionLabel?: string
  onSave: (values: Record<string, string>) => Promise<void> | void
  onDelete: () => void
}

function GroupLabel({
  icon: Icon,
  children,
  trailing,
}: {
  icon: ComponentType<{ className?: string }>
  children: string
  trailing?: ReactNode
}) {
  return (
    <div className="text-muted-foreground flex items-center justify-between gap-2 px-3 pt-3 pb-1 text-[10.5px] font-semibold tracking-[0.08em] uppercase leading-none">
      <span className="flex items-center gap-1.5">
        <Icon className="size-3 shrink-0" />
        {children}
      </span>
      {trailing}
    </div>
  )
}
export interface SettingsDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  notify: SettingsDialogNotifyProps
  connection?: SettingsDialogConnectionProps
}

/**
 * Composite settings dialog: notifications is the primary content (the dialog
 * footer's Save persists the notify values), with an optional inline
 * connection/credential section beneath it. The connection section
 * self-manages its own credential via its own `onSave`/`onDelete` — it never
 * hides the notify section, it only ever shows a "not connected" banner.
 *
 * Apps that only need notifications should render `NotifySettings` directly
 * instead of this component.
 */
export function SettingsDialog({ open, onOpenChange, notify, connection }: SettingsDialogProps) {
  const [value, setValue] = useState<NotifyValues>(notify.values)

  useEffect(() => {
    if (!open) return
    setValue({
      email: notify.values.email,
      inApp: notify.values.inApp,
      recipients: notify.values.recipients,
    })
  }, [open, notify.values.email, notify.values.inApp, notify.values.recipients])

  // Instant save: every change (toggle, recipient) persists immediately, like macOS/iOS
  // settings. No Save/Cancel footer — the dialog just closes (via the header ✕ / backdrop).
  const handleChange = (next: NotifyValues) => {
    setValue(next)
    notify.onSave(next)
  }

  return (
    <SettingsDialogShell
      open={open}
      onOpenChange={onOpenChange}
      title={notify.labels.title}
      description={notify.labels.description}
    >
      {notify.sectionLabel && <GroupLabel icon={Bell}>{notify.sectionLabel}</GroupLabel>}
      <NotifySettingsBody
        value={value}
        onChange={handleChange}
        userEmail={notify.userEmail}
        labels={notify.labels}
      />

      {connection && (
        <>
          <Separator />
          {connection.sectionLabel && (
            <GroupLabel
              icon={KeyRound}
              trailing={
                <span
                  className={`inline-flex items-center gap-1.5 text-[11px] font-semibold tracking-normal normal-case ${
                    connection.configured ? 'text-primary' : 'text-destructive'
                  }`}
                >
                  <span
                    className={`size-2 rounded-full ${connection.configured ? 'bg-primary' : 'bg-destructive'}`}
                  />
                  {connection.configured ? connection.labels.connected : connection.labels.notConnected}
                </span>
              }
            >
              {connection.sectionLabel}
            </GroupLabel>
          )}
          <ConnectionSection
            configured={connection.configured}
            maskedKey={connection.maskedKey}
            fields={connection.fields}
            getKey={connection.getKey}
            labels={connection.labels}
            onSave={connection.onSave}
            onDelete={connection.onDelete}
          />
        </>
      )}
    </SettingsDialogShell>
  )
}
