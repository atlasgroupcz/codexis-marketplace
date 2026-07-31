import { useEffect, useState } from 'react'
import { Bell, KeyRound } from 'lucide-react'
import { ConnectionSection } from '@workspace/ui/components/connection-section'
import { NotifySettingsBody } from '@workspace/ui/components/notify-settings-body'
import { Separator } from '@workspace/ui/components/separator'
import { SettingsDialogShell } from '@workspace/ui/components/settings-dialog-shell'
import { ConnectionStatus, connStateOf } from '@workspace/ui/components/connection-status'
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
  /**
   * `null` while the first fetch is in flight. Passing `?? false` instead makes
   * the badge assert "not connected" in red before anything has been checked.
   */
  configured: boolean | null
  maskedKey: string
  fields: Array<ConnField>
  getKey: ConnGetKeyLink
  labels: ConnectionSectionLabels
  sectionLabel?: string
  onSave: (values: Record<string, string>) => Promise<void> | void
  onDelete: () => void
}

/**
 * Section heading inside the dialog: an icon, a label, and an optional trailing
 * slot (the connection badge). Exported because a panel that *is* a settings
 * page rather than a dialog needs the same heading and would otherwise copy the
 * class string; `className` lets it drop the dialog's own padding.
 */
export function GroupLabel({
  icon: Icon,
  children,
  trailing,
  className = 'px-3 pt-3 pb-1',
}: {
  icon: ComponentType<{ className?: string }>
  children: string
  trailing?: ReactNode
  className?: string
}) {
  return (
    <div
      className={`text-muted-foreground flex items-center justify-between gap-2 text-[10.5px] font-semibold tracking-[0.08em] uppercase leading-none ${className}`}
    >
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
 * self-manages its own credential via its own `onSave`/`onDelete` - it never
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
  // settings. No Save/Cancel footer - the dialog just closes (via the header ✕ / backdrop).
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
                <ConnectionStatus
                  state={connStateOf(connection.configured)}
                  labels={connection.labels}
                />
              }
            >
              {connection.sectionLabel}
            </GroupLabel>
          )}
          <ConnectionSection
            loading={connection.configured === null}
            configured={connection.configured ?? false}
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
