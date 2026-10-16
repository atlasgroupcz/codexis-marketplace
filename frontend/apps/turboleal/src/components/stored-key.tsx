import { Button } from '@workspace/ui/components/button'
import type { ConnGetKeyLink } from '@workspace/ui/components/connection-settings'
import { cn } from '@workspace/ui/lib/utils'
import { Check, ExternalLink, Trash2 } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'

export interface StoredKeyLabels {
  remove: string
  removeKey: string
  confirmRemove: string
}

interface StoredKeyProps {
  maskedKey: string
  getKey: ConnGetKeyLink
  labels: StoredKeyLabels
  onDelete: () => void
}

export function StoredKey({ maskedKey, getKey, labels, onDelete }: StoredKeyProps) {
  const [confirmRemove, setConfirmRemove] = useState(false)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(
    () => () => {
      if (timerRef.current) clearTimeout(timerRef.current)
    },
    [],
  )

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

  return (
    <div className="space-y-2.5">
      <div className="group bg-brand-50 ring-brand-100 flex items-center gap-2.5 rounded-[10px] px-3 py-2.5 ring-1">
        <Check className="text-primary size-4 shrink-0" />
        <code className="min-w-0 flex-1 truncate font-mono text-sm">{maskedKey}</code>
        <Button
          type="button"
          variant={confirmRemove ? 'destructive' : 'ghost'}
          size={confirmRemove ? 'sm' : 'icon-sm'}
          aria-label={confirmRemove ? labels.confirmRemove : labels.removeKey}
          title={confirmRemove ? labels.confirmRemove : labels.remove}
          onClick={remove}
          className={cn(
            '-my-1 h-7 shrink-0',
            confirmRemove
              ? 'px-2 text-xs'
              : 'text-muted-foreground hover:text-destructive w-7 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 focus-visible:opacity-100 [@media(hover:none)]:opacity-100',
          )}
        >
          {confirmRemove ? labels.confirmRemove : <Trash2 className="size-4" />}
        </Button>
      </div>

      <div className="min-h-[1.25rem]" />

      <div>
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
    </div>
  )
}
