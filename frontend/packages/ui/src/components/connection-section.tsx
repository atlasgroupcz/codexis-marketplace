import { Button } from '@workspace/ui/components/button'
import type { ConnField, ConnGetKeyLink } from '@workspace/ui/components/connection-settings'
import { Input } from '@workspace/ui/components/input'
import { Label } from '@workspace/ui/components/label'
import { Skeleton } from '@workspace/ui/components/skeleton'
import { Check, ExternalLink } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'

export interface ConnectionSectionLabels {
  connected: string
  notConnected: string
  remove: string
  confirmRemove: string
  save: string
  saving: string
}

export interface ConnectionSectionProps {
  configured: boolean
  /**
   * The first fetch has not answered yet. Renders a placeholder at this
   * component's own heights, so the card is the size it will stay and the
   * empty field is never shown for a frame before the stored key replaces it.
   */
  loading?: boolean
  maskedKey: string
  fields: Array<ConnField>
  getKey: ConnGetKeyLink
  labels: ConnectionSectionLabels
  /**
   * Persist the credential (only ever called from the not-configured state).
   * MUST reject (throw) on an invalid key so the error shows inline; on success
   * the parent refetches and `configured` flips to the read-only view.
   */
  onSave: (values: Record<string, string>) => Promise<void> | void
  onDelete: () => void
}

/**
 * Inline credential block for the settings dialog. Two states share one layout so
 * nothing shifts: the top is the only thing that changes - a READ-ONLY masked key
 * (configured) or editable field(s) (not configured). Below it a reserved error
 * slot, then a fixed bottom row with the "get key" link on the left and the single
 * action on the right - Odebrat (2-step confirm) when configured, Uložit
 * (validates on save) when not. Changing a key is remove-then-add. The connected/
 * not-connected status dot lives on the section heading (rendered by the dialog).
 */
export function ConnectionSection({
  configured,
  loading = false,
  maskedKey,
  fields,
  getKey,
  labels,
  onSave,
  onDelete,
}: ConnectionSectionProps) {
  const [values, setValues] = useState<Record<string, string>>({})
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')
  const [confirmRemove, setConfirmRemove] = useState(false)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // Switching between configured / not-configured resets the transient state.
  useEffect(() => {
    setValues({})
    setError('')
    setConfirmRemove(false)
  }, [configured])

  useEffect(
    () => () => {
      if (timerRef.current) clearTimeout(timerRef.current)
    },
    [],
  )

  const allFilled = fields.every((f) => (values[f.name] ?? '').trim().length > 0)

  const save = () => {
    if (!allFilled || saving) return
    setSaving(true)
    setError('')
    Promise.resolve(onSave(values))
      .catch((e: unknown) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setSaving(false))
  }

  const remove = () => {
    if (!confirmRemove) {
      setConfirmRemove(true)
      timerRef.current = setTimeout(() => setConfirmRemove(false), 3000)
      return
    }
    if (timerRef.current) clearTimeout(timerRef.current)
    setConfirmRemove(false)
    onDelete()
  }

  // Same three blocks at the same heights, so the placeholder cannot drift
  // from the real layout the way a hand-measured copy in a consumer would.
  if (loading) {
    return (
      <div className="space-y-2.5" aria-busy="true">
        <Skeleton className="h-9 w-full" />
        <div className="min-h-[1.25rem]" />
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0 space-y-1">
            <Skeleton className="h-5 w-32" />
            {getKey.hint && <Skeleton className="h-4 w-44" />}
          </div>
          <Skeleton className="h-8 w-20 shrink-0" />
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-2.5">
      {/* Top - the only thing that changes between states. */}
      {configured ? (
        <div className="bg-brand-50 ring-brand-100 flex items-center gap-2.5 rounded-[10px] px-3 py-2.5 ring-1">
          <Check className="text-primary size-4 shrink-0" />
          <code className="min-w-0 flex-1 truncate font-mono text-sm">{maskedKey}</code>
        </div>
      ) : (
        <div className="space-y-2.5">
          {fields.map((f) => (
            <div key={f.name}>
              {fields.length > 1 && (
                <Label
                  htmlFor={f.name}
                  className="text-muted-foreground mb-1 block text-xs font-medium"
                >
                  {f.label}
                </Label>
              )}
              <Input
                id={f.name}
                type={f.type ?? 'text'}
                autoComplete="off"
                spellCheck={false}
                data-testid={f.testId}
                className={`h-9 ${f.mono ? 'font-mono' : ''}`}
                placeholder={f.placeholder}
                value={values[f.name] ?? ''}
                onChange={(e) => {
                  const next = e.target.value
                  setValues((v) => ({ ...v, [f.name]: next }))
                  setError('')
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    e.preventDefault()
                    save()
                  }
                }}
              />
            </div>
          ))}
        </div>
      )}

      {/* Reserved slot: holds the validation error; fixed height so nothing shifts. */}
      <p className="text-destructive min-h-[1.25rem] text-xs" role={error ? 'alert' : undefined}>
        {error}
      </p>

      {/* Fixed bottom row: get-key link (left) + single action (right), same in both states. */}
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <a
            href={getKey.href}
            target="_blank"
            rel="noreferrer"
            className="text-primary inline-flex items-center gap-1 text-sm hover:underline"
          >
            <ExternalLink className="size-3.5 shrink-0" />
            {getKey.label}
          </a>
          {getKey.hint && <p className="text-muted-foreground mt-1 text-xs">{getKey.hint}</p>}
        </div>
        {configured ? (
          <Button
            type="button"
            size="sm"
            variant={confirmRemove ? 'destructive' : 'outline'}
            onClick={remove}
            className="shrink-0"
          >
            {confirmRemove ? labels.confirmRemove : labels.remove}
          </Button>
        ) : (
          <Button
            type="button"
            size="sm"
            onClick={save}
            disabled={!allFilled || saving}
            className="shrink-0"
          >
            {saving ? labels.saving : labels.save}
          </Button>
        )}
      </div>
    </div>
  )
}
