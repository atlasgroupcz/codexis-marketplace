import { useEffect, useState } from 'react'
import { NotifySettingsBody } from '@workspace/ui/components/notify-settings-body'
import { SettingsDialogShell } from '@workspace/ui/components/settings-dialog-shell'
import type { NotifyBodyLabels, NotifyValues } from '@workspace/ui/components/notify-settings-body'

export type { NotifyValues } from '@workspace/ui/components/notify-settings-body'

export interface NotifyLabels extends NotifyBodyLabels {
  title: string
  description: string
  save: string
  cancel: string
}
export interface NotifySettingsProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  values: NotifyValues
  userEmail: string
  labels: NotifyLabels
  saving?: boolean
  onSave: (values: NotifyValues) => void
}

export function NotifySettings({
  open,
  onOpenChange,
  values,
  userEmail,
  labels,
  onSave,
}: NotifySettingsProps) {
  const [value, setValue] = useState<NotifyValues>(values)

  useEffect(() => {
    if (!open) return
    setValue({ email: values.email, inApp: values.inApp, recipients: values.recipients })
  }, [open, values.email, values.inApp, values.recipients])

  // Instant save: every change persists immediately (like macOS/iOS settings). No footer —
  // the dialog closes via the header ✕ / backdrop.
  const handleChange = (next: NotifyValues) => {
    setValue(next)
    onSave(next)
  }

  return (
    <SettingsDialogShell
      open={open}
      onOpenChange={onOpenChange}
      title={labels.title}
      description={labels.description}
    >
      <NotifySettingsBody value={value} onChange={handleChange} userEmail={userEmail} labels={labels} />
    </SettingsDialogShell>
  )
}
