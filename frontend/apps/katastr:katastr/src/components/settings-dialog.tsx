import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CheckCircle2 } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@workspace/ui/components/dialog'
import { toast } from '@workspace/ui/components/sonner'
import { deleteApiKey, saveApiKey } from '@/lib/api'

interface SettingsDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  configured: boolean
  maskedKey: string
  onSaved?: () => void
}

export function SettingsDialog({
  open,
  onOpenChange,
  configured,
  maskedKey,
  onSaved,
}: SettingsDialogProps) {
  const { t } = useTranslation()
  const [apiKey, setApiKey] = useState('')
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    if (!open) return
    setApiKey('')
  }, [open])

  const handleSave = () => {
    if (!apiKey.trim()) return
    setSaving(true)
    saveApiKey(apiKey.trim())
      .then((res) => {
        if (res.ok) {
          setApiKey('')
          onSaved?.()
          onOpenChange(false)
        } else {
          toast.error(res.error ?? t('settings.saveError'))
        }
      })
      .catch((e: unknown) => {
        toast.error(e instanceof Error ? e.message : String(e))
      })
      .finally(() => setSaving(false))
  }

  const handleDelete = () => {
    setSaving(true)
    deleteApiKey()
      .then((res) => {
        if (res.ok) {
          onSaved?.()
          onOpenChange(false)
        } else {
          toast.error(res.error ?? t('settings.saveError'))
        }
      })
      .catch((e: unknown) => {
        toast.error(e instanceof Error ? e.message : String(e))
      })
      .finally(() => setSaving(false))
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="pr-8">{t('settings.title')}</DialogTitle>
          <DialogDescription>{t('settings.description')}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          {configured ? (
            <div className="flex items-center gap-2 rounded-md border border-green-200 bg-green-50 px-3 py-2.5 text-sm text-green-900 dark:border-green-900 dark:bg-green-950 dark:text-green-200">
              <CheckCircle2 className="size-4 shrink-0" />
              <span className="min-w-0 flex-1">
                {t('settings.currentKey')}:{' '}
                <code className="font-mono break-all">{maskedKey}</code>
              </span>
            </div>
          ) : (
            <div>
              <label htmlFor="apiKey" className="mb-2 block text-sm font-medium">
                {t('settings.newLabel')}
              </label>
              <input
                id="apiKey"
                type="text"
                autoComplete="off"
                spellCheck={false}
                className="border-input bg-background focus-visible:ring-ring focus-visible:border-ring flex h-10 w-full rounded-md border px-3 py-2 font-mono text-sm transition-colors focus-visible:ring-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
                placeholder={t('settings.placeholder')}
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleSave()}
                disabled={saving}
                data-testid="api-key-input"
              />
            </div>
          )}

        </div>

        <DialogFooter className="gap-2 sm:gap-2">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.close')}
          </Button>
          {configured ? (
            <Button
              variant="destructive"
              onClick={handleDelete}
              disabled={saving}
              data-testid="delete-api-key"
            >
              {saving ? t('settings.deleting') : t('settings.deleteKey')}
            </Button>
          ) : (
            <Button onClick={handleSave} disabled={saving || !apiKey.trim()}>
              {saving ? t('settings.saving') : t('settings.save')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
