import { useEffect, useRef, useState } from 'react'
import { ArrowLeft, Pencil, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@workspace/ui/components/button'
import { toast } from '@workspace/ui/components/sonner'
import { removeProceeding, setLabel } from '@/lib/api'

interface DetailHeaderProps {
  uuid: string
  cisloRizeni: string
  label: string
  onBack: () => void
  onLabelChanged: (newLabel: string) => void
  onDeleted: () => void
}

export function DetailHeader({
  uuid,
  cisloRizeni,
  label,
  onBack,
  onLabelChanged,
  onDeleted,
}: DetailHeaderProps) {
  const { t } = useTranslation()

  const [editingLabel, setEditingLabel] = useState(false)
  const [labelDraft, setLabelDraft] = useState('')
  const labelInputRef = useRef<HTMLInputElement>(null)

  const [confirmRemove, setConfirmRemove] = useState(false)
  const [removing, setRemoving] = useState(false)
  const confirmTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    if (editingLabel) {
      labelInputRef.current?.focus()
      labelInputRef.current?.select()
    }
  }, [editingLabel])

  const startEditLabel = () => {
    setLabelDraft(label)
    setEditingLabel(true)
  }

  const cancelEditLabel = () => {
    setEditingLabel(false)
    setLabelDraft('')
  }

  const saveLabel = () => {
    const next = labelDraft.trim()
    if (next === label) {
      setEditingLabel(false)
      return
    }
    // Optimistic — caller updates immediately, server sync happens in bg.
    onLabelChanged(next)
    setEditingLabel(false)
    setLabel(uuid, next).catch((e: unknown) => {
      toast.error(e instanceof Error ? e.message : String(e))
      onLabelChanged(label) // rollback
    })
  }

  useEffect(() => {
    return () => {
      if (confirmTimerRef.current) clearTimeout(confirmTimerRef.current)
    }
  }, [])

  const handleRemove = () => {
    if (!confirmRemove) {
      setConfirmRemove(true)
      confirmTimerRef.current = setTimeout(() => setConfirmRemove(false), 3000)
      return
    }
    if (confirmTimerRef.current) clearTimeout(confirmTimerRef.current)
    setRemoving(true)
    removeProceeding(uuid)
      .then(() => onDeleted())
      .finally(() => setRemoving(false))
  }

  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="sm" onClick={onBack} data-testid="back-to-list">
          <ArrowLeft className="size-4" />
          {t('common.back')}
        </Button>
        <h1 className="text-xl font-semibold">{cisloRizeni}</h1>
        <div className="min-w-[180px]">
          {editingLabel ? (
            <input
              ref={labelInputRef}
              type="text"
              value={labelDraft}
              onChange={(e) => setLabelDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') saveLabel()
                if (e.key === 'Escape') cancelEditLabel()
              }}
              onBlur={saveLabel}
              placeholder={t('proceedings.labelPlaceholder')}
              className="border-input bg-background w-full rounded-md border px-2 py-1 text-sm"
              data-testid="label-input"
            />
          ) : (
            <button
              type="button"
              onClick={startEditLabel}
              className="text-muted-foreground hover:text-foreground inline-flex items-center gap-1 text-sm transition-colors"
              data-testid="label-edit-trigger"
            >
              {label ? (
                <span>({label})</span>
              ) : (
                <span className="italic">{t('proceedings.addLabelHint')}</span>
              )}
              <Pencil className="size-3" />
            </button>
          )}
        </div>
      </div>
      <Button
        variant={confirmRemove ? 'destructive' : 'outline'}
        size="sm"
        onClick={handleRemove}
        disabled={removing}
        data-testid="remove-proceeding"
      >
        <Trash2 className="size-4" />
        {confirmRemove ? t('proceedings.confirmRemove') : t('proceedings.remove')}
      </Button>
    </div>
  )
}
