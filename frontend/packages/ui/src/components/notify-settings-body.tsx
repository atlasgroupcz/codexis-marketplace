import { useEffect, useRef, useState } from 'react'
import { Bell, Mail, Plus, X } from 'lucide-react'
import { isValidEmail } from '@workspace/ui/lib/notify-recipients'
import { cn } from '@workspace/ui/lib/utils'

/** Pill toggle sized to the notify-panel design (38×22, 18px knob). */
function ToggleSwitch({
  checked,
  onChange,
  label,
}: {
  checked: boolean
  onChange: (checked: boolean) => void
  label: string
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={() => onChange(!checked)}
      className={cn(
        'focus-visible:ring-ring/50 inline-flex h-[22px] w-[38px] shrink-0 items-center rounded-full px-0.5 transition-colors outline-none focus-visible:ring-2',
        checked ? 'bg-primary' : 'bg-input',
      )}
    >
      <span
        className={cn(
          'size-[18px] rounded-full bg-white shadow-sm transition-transform',
          checked ? 'translate-x-4' : 'translate-x-0',
        )}
      />
    </button>
  )
}

export interface NotifyValues {
  email: boolean
  inApp: boolean
  recipients: Array<string>
}
export interface NotifyBodyLabels {
  emailLabel: string
  emailDescription: string
  yourAddressLabel: string
  copyToLabel: string
  addRecipient: string
  removeRecipient: string
  addPlaceholder: string
  invalidEmail: string
  ownAddress: string
  duplicate: string
  inAppLabel: string
  inAppDescription: string
}
export interface NotifySettingsBodyProps {
  value: NotifyValues
  onChange: (value: NotifyValues) => void
  userEmail: string
  labels: NotifyBodyLabels
}

/**
 * Notification settings content (email toggle, "your address", "copy to" list,
 * in-app bell toggle) with no dialog shell of its own — a controlled component
 * that reads/writes `value` via `onChange`. Used directly by `NotifySettings`
 * (as a standalone dialog) and by composite dialogs that combine it with other
 * sections (e.g. `SettingsDialog` + `ConnectionSection`).
 */
export function NotifySettingsBody({ value, onChange, userEmail, labels }: NotifySettingsBodyProps) {
  const { email, inApp, recipients } = value
  const [draft, setDraft] = useState('')
  const [err, setErr] = useState('')
  const [adding, setAdding] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (adding) inputRef.current?.focus()
  }, [adding])

  const cancelAdd = () => {
    setAdding(false)
    setDraft('')
    setErr('')
  }

  // Commit the typed address: save by default (Enter / Esc / click-away all add it).
  // Empty closes the field; an invalid address keeps the field open with the error.
  const commit = () => {
    const v = draft.trim()
    if (!v) {
      cancelAdd()
      return
    }
    if (!isValidEmail(v)) {
      setErr(labels.invalidEmail)
      return
    }
    const key = v.toLowerCase()
    // the user's own address is redundant (they always receive it) and dupes are pointless —
    // both keep the field open with an explanation instead of silently dropping the address.
    if (key === userEmail.trim().toLowerCase()) {
      setErr(labels.ownAddress)
      return
    }
    if (recipients.some((r) => r.toLowerCase() === key)) {
      setErr(labels.duplicate)
      return
    }
    onChange({ ...value, recipients: [...recipients, v] })
    cancelAdd()
  }

  const remove = (v: string) =>
    onChange({ ...value, recipients: recipients.filter((r) => r !== v) })

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between gap-3 rounded-lg px-3 py-2.5">
        <span className="flex items-start gap-2.5">
          <Mail className="text-muted-foreground mt-0.5 size-4" />
          <span>
            <span className="text-sm font-medium">{labels.emailLabel}</span>
            {labels.emailDescription && (
              <>
                <br />
                <span className="text-muted-foreground text-xs">{labels.emailDescription}</span>
              </>
            )}
          </span>
        </span>
        <ToggleSwitch
          checked={email}
          onChange={(checked) => onChange({ ...value, email: checked })}
          label={labels.emailLabel}
        />
      </div>

      <div
        aria-hidden={!email}
        className={cn(
          'grid transition-[grid-template-rows] duration-200 ease-out',
          email ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]',
        )}
      >
        <div className="overflow-hidden">
          <div className="flex flex-wrap items-center gap-1.5 pt-1 pr-3 pb-2 pl-[38px]">
            {userEmail && (
              <span
                title={labels.yourAddressLabel}
                className="bg-muted text-foreground ring-border inline-flex h-[26px] items-center gap-1.5 rounded-full px-2.5 font-mono text-xs ring-1"
              >
                <span className="bg-primary size-1.5 shrink-0 rounded-full" />
                {userEmail}
              </span>
            )}
            {recipients.map((r) => (
              <span
                key={r}
                className="bg-secondary text-secondary-foreground ring-brand-100 inline-flex h-[26px] items-center gap-1 rounded-full py-0 pr-1.5 pl-2.5 text-xs ring-1"
              >
                {r}
                <button
                  type="button"
                  aria-label={`${labels.removeRecipient}: ${r}`}
                  onClick={() => remove(r)}
                  className="hover:bg-secondary-foreground/15 grid place-items-center rounded-full p-0.5"
                >
                  <X className="size-3" />
                </button>
              </span>
            ))}
            {adding ? (
              <input
                ref={inputRef}
                data-keep-dialog-open=""
                value={draft}
                onChange={(e) => {
                  setDraft(e.target.value)
                  setErr('')
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    e.preventDefault()
                    commit()
                  } else if (e.key === 'Escape') {
                    // keep Escape scoped to the field — save + close it, don't close the dialog
                    e.preventDefault()
                    e.stopPropagation()
                    e.nativeEvent.stopImmediatePropagation()
                    commit()
                  }
                }}
                onBlur={() => commit()}
                placeholder={labels.addPlaceholder}
                className="border-input bg-background h-[26px] w-48 rounded-full border px-3 font-mono text-xs"
              />
            ) : (
              <button
                type="button"
                onClick={() => setAdding(true)}
                className="text-muted-foreground border-border hover:bg-muted inline-flex h-[26px] items-center gap-1 rounded-full border border-dashed px-2.5 text-xs"
              >
                <Plus className="size-3" />
                {labels.addRecipient}
              </button>
            )}
            {err && <p className="text-destructive w-full pt-0.5 text-xs">{err}</p>}
          </div>
        </div>
      </div>

      <div className="flex items-center justify-between gap-3 rounded-lg px-3 py-2.5">
        <span className="flex items-start gap-2.5">
          <Bell className="text-muted-foreground mt-0.5 size-4" />
          <span>
            <span className="text-sm font-medium">{labels.inAppLabel}</span>
            {labels.inAppDescription && (
              <>
                <br />
                <span className="text-muted-foreground text-xs">{labels.inAppDescription}</span>
              </>
            )}
          </span>
        </span>
        <ToggleSwitch
          checked={inApp}
          onChange={(checked) => onChange({ ...value, inApp: checked })}
          label={labels.inAppLabel}
        />
      </div>
    </div>
  )
}
