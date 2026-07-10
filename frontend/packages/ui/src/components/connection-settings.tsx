import { useEffect, useState } from 'react'
import { CheckCircle2, ExternalLink } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Input } from '@workspace/ui/components/input'
import { Label } from '@workspace/ui/components/label'
import { SettingsDialogShell } from '@workspace/ui/components/settings-dialog-shell'

export interface ConnField {
  name: string
  label: string
  type?: 'text' | 'password'
  placeholder?: string
  mono?: boolean
  testId?: string
}
export interface ConnGetKeyLink {
  href: string
  label: string
  hint?: string
}
export interface ConnectionSettingsProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  configured: boolean
  maskedKey: string
  fields: Array<ConnField>
  getKey: ConnGetKeyLink
  labels: {
    title: string
    description?: string
    currentKey: string
    deleteKey: string
    save: string
    cancel: string
    deleting: string
  }
  onSave: (values: Record<string, string>) => void
  onDelete: () => void
  saving?: boolean
}

export function ConnectionSettings({
  open,
  onOpenChange,
  configured,
  maskedKey,
  fields,
  getKey,
  labels,
  onSave,
  onDelete,
  saving = false,
}: ConnectionSettingsProps) {
  const [values, setValues] = useState<Record<string, string>>({})

  useEffect(() => {
    if (open) setValues({})
  }, [open])

  const allFilled = fields.every((f) => (values[f.name] ?? '').trim().length > 0)

  const footer = (
    <>
      <Button variant="outline" onClick={() => onOpenChange(false)} disabled={saving}>
        {labels.cancel}
      </Button>
      {configured ? (
        <Button variant="destructive" onClick={onDelete} disabled={saving}>
          {saving ? labels.deleting : labels.deleteKey}
        </Button>
      ) : (
        <Button onClick={() => onSave(values)} disabled={saving || !allFilled}>
          {labels.save}
        </Button>
      )}
    </>
  )

  return (
    <SettingsDialogShell
      open={open}
      onOpenChange={onOpenChange}
      title={labels.title}
      description={labels.description}
      footer={footer}
    >
      <div className="space-y-4 py-2">
        <a
          href={getKey.href}
          target="_blank"
          rel="noreferrer"
          className="text-primary inline-flex items-center gap-1 text-sm hover:underline"
        >
          <ExternalLink className="size-3.5" />
          {getKey.label}
        </a>
        {getKey.hint && <p className="text-muted-foreground -mt-2 text-xs">{getKey.hint}</p>}

        {configured ? (
          <div className="flex items-center gap-2 rounded-md border border-green-200 bg-green-50 px-3 py-2.5 text-sm text-green-900 dark:border-green-900 dark:bg-green-950 dark:text-green-200">
            <CheckCircle2 className="size-4 shrink-0" />
            <span className="min-w-0 flex-1">
              {labels.currentKey}: <code className="font-mono break-all">{maskedKey}</code>
            </span>
          </div>
        ) : (
          <div className="space-y-3">
            {fields.map((f) => (
              <div key={f.name}>
                <Label htmlFor={f.name} className="mb-1.5 block">
                  {f.label}
                </Label>
                <Input
                  id={f.name}
                  type={f.type ?? 'text'}
                  autoComplete="off"
                  spellCheck={false}
                  placeholder={f.placeholder}
                  data-testid={f.testId}
                  className={f.mono ? 'font-mono' : undefined}
                  value={values[f.name] ?? ''}
                  onChange={(e) => setValues((v) => ({ ...v, [f.name]: e.target.value }))}
                />
              </div>
            ))}
          </div>
        )}
      </div>
    </SettingsDialogShell>
  )
}
