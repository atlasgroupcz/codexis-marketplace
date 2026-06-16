import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CheckCircle2, ExternalLink } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@workspace/ui/components/dialog'
import { Input } from '@workspace/ui/components/input'
import { Label } from '@workspace/ui/components/label'
import { toast } from '@workspace/ui/components/sonner'
import type { ConfigResponse } from '@/lib/schemas'
import { deleteConfig, saveConfig } from '@/lib/api'

export function SettingsDialog({
  open,
  onOpenChange,
  config,
  onSaved,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  config: ConfigResponse | null
  onSaved: () => void
}) {
  const { t } = useTranslation()
  const [clientId, setClientId] = useState('')
  const [clientSecret, setClientSecret] = useState('')
  const [saving, setSaving] = useState(false)
  const configured = config?.configured ?? false

  useEffect(() => {
    if (open) {
      setClientId('')
      setClientSecret('')
    }
  }, [open])

  const save = () => {
    if (!clientId.trim() || !clientSecret.trim()) return
    setSaving(true)
    saveConfig(clientId.trim(), clientSecret.trim())
      .then((res) => {
        if (!res.ok) {
          toast.error(res.error ?? t('settings.saveError'))
          return
        }
        toast.success(t('settings.saved'))
        onSaved()
        onOpenChange(false)
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setSaving(false))
  }

  const remove = () => {
    setSaving(true)
    deleteConfig()
      .then((res) => {
        if (res.ok) {
          onSaved()
          onOpenChange(false)
        } else {
          toast.error(res.error ?? t('settings.saveError'))
        }
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setSaving(false))
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="font-display pr-8 text-xl font-normal">{t('settings.title')}</DialogTitle>
          <DialogDescription>{t('settings.description')}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <a
            href="https://dev.euipo.europa.eu"
            target="_blank"
            rel="noreferrer"
            className="text-primary inline-flex items-center gap-1 text-sm hover:underline"
          >
            <ExternalLink className="size-3.5" />
            {t('settings.getKey')}
          </a>

          {configured ? (
            <div className="flex items-center gap-2 rounded-md border border-green-200 bg-green-50 px-3 py-2.5 text-sm text-green-900 dark:border-green-900 dark:bg-green-950 dark:text-green-200">
              <CheckCircle2 className="size-4 shrink-0" />
              <span className="min-w-0 flex-1">
                {t('settings.currentKey')}: <code className="font-mono break-all">{config?.maskedKey}</code>
              </span>
            </div>
          ) : (
            <>
              <div className="space-y-1.5">
                <Label htmlFor="client-id">{t('settings.clientId')}</Label>
                <Input
                  id="client-id"
                  autoComplete="off"
                  spellCheck={false}
                  className="font-mono"
                  value={clientId}
                  onChange={(e) => setClientId(e.target.value)}
                  disabled={saving}
                  data-testid="client-id-input"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="client-secret">{t('settings.clientSecret')}</Label>
                <Input
                  id="client-secret"
                  type="password"
                  autoComplete="off"
                  spellCheck={false}
                  className="font-mono"
                  value={clientSecret}
                  onChange={(e) => setClientSecret(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && save()}
                  disabled={saving}
                  data-testid="client-secret-input"
                />
              </div>
            </>
          )}
        </div>

        <DialogFooter className="gap-2 sm:gap-2">
          <Button variant="outline" onClick={() => onOpenChange(false)}>{t('common.close')}</Button>
          {configured ? (
            <Button variant="destructive" onClick={remove} disabled={saving} data-testid="delete-config">
              {saving ? t('settings.deleting') : t('settings.deleteKey')}
            </Button>
          ) : (
            <Button onClick={save} disabled={saving || !clientId.trim() || !clientSecret.trim()} data-testid="save-config">
              {saving ? t('settings.saving') : t('settings.save')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
