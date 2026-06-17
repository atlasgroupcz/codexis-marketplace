import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Bell, Loader2, Mail } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@workspace/ui/components/dialog'
import { Button } from '@workspace/ui/components/button'
import { Switch } from '@workspace/ui/components/switch'
import { toast } from '@workspace/ui/components/sonner'
import type { NotifySettings } from '@/lib/schemas'
import { postFolderAction } from '@/lib/api'

interface NotifySettingsDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  settings: NotifySettings
  onSaved: (settings: NotifySettings) => void
}

export function NotifySettingsDialog({
  open,
  onOpenChange,
  settings,
  onSaved,
}: NotifySettingsDialogProps) {
  const { t } = useTranslation()
  const [email, setEmail] = useState(settings.email)
  const [inApp, setInApp] = useState(settings.inApp)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    if (open) {
      setEmail(settings.email)
      setInApp(settings.inApp)
    }
  }, [open, settings.email, settings.inApp])

  const save = () => {
    setSaving(true)
    postFolderAction('save_notify_settings', { email, inApp, recipients: settings.recipients })
      .then(() => {
        toast.success(t('folders.notify.saved'))
        onSaved({ email, inApp, recipients: settings.recipients })
        onOpenChange(false)
      })
      .catch((err: unknown) =>
        toast.error(err instanceof Error ? err.message : String(err)),
      )
      .finally(() => setSaving(false))
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t('folders.notify.title')}</DialogTitle>
          <DialogDescription>{t('folders.notify.description')}</DialogDescription>
        </DialogHeader>

        <div className="space-y-1">
          <label className="hover:bg-accent/50 flex items-center justify-between gap-3 rounded-lg px-3 py-2.5">
            <span className="flex items-center gap-2.5">
              <Mail className="text-muted-foreground size-4" />
              <span className="text-sm font-medium">{t('folders.notify.email')}</span>
            </span>
            <Switch checked={email} onCheckedChange={setEmail} />
          </label>
          <label className="hover:bg-accent/50 flex items-center justify-between gap-3 rounded-lg px-3 py-2.5">
            <span className="flex items-center gap-2.5">
              <Bell className="text-muted-foreground size-4" />
              <span className="text-sm font-medium">{t('folders.notify.inApp')}</span>
            </span>
            <Switch checked={inApp} onCheckedChange={setInApp} />
          </label>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={saving}>
            {t('folders.cancel')}
          </Button>
          <Button onClick={save} disabled={saving}>
            {saving && <Loader2 className="size-4 animate-spin" />}
            {t('folders.notify.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
