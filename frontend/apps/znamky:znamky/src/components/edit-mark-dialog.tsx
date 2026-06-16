import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@workspace/ui/components/dialog'
import { Input } from '@workspace/ui/components/input'
import { Button } from '@workspace/ui/components/button'
import { Label } from '@workspace/ui/components/label'
import { toast } from '@workspace/ui/components/sonner'
import { updateMark } from '@/lib/api'
import type { MarkDetail } from '@/lib/schemas'

const TERRITORIES = ['CZ', 'EU'] as const

function parseClasses(value: string): number[] {
  return Array.from(
    new Set(
      value
        .split(/[^0-9]+/)
        .map((s) => parseInt(s, 10))
        .filter((n) => Number.isInteger(n) && n >= 1 && n <= 45),
    ),
  )
}

export function EditMarkDialog({
  open,
  onOpenChange,
  mark,
  onSaved,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  mark: MarkDetail
  onSaved: () => void
}) {
  const { t } = useTranslation()
  const [text, setText] = useState(mark.text)
  const [nice, setNice] = useState(mark.nice_classes.join(', '))
  const [territories, setTerritories] = useState<string[]>(mark.territories)
  const [owner, setOwner] = useState(mark.owner_name)
  const [label, setLabel] = useState(mark.label)
  const [threshold, setThreshold] = useState(mark.threshold)
  const [submitting, setSubmitting] = useState(false)

  // Re-seed fields whenever a different mark (or fresh data) is opened.
  useEffect(() => {
    setText(mark.text)
    setNice(mark.nice_classes.join(', '))
    setTerritories(mark.territories)
    setOwner(mark.owner_name)
    setLabel(mark.label)
    setThreshold(mark.threshold)
  }, [mark])

  const toggleTerritory = (terr: string) =>
    setTerritories((cur) => (cur.includes(terr) ? cur.filter((x) => x !== terr) : [...cur, terr]))

  const save = () => {
    setSubmitting(true)
    updateMark(mark.uuid, {
      text: mark.kind === 'figurative' ? undefined : text.trim(),
      nice_classes: parseClasses(nice),
      territories,
      owner_name: owner.trim(),
      label: label.trim(),
      threshold,
    })
      .then((res) => {
        if (!res.ok) {
          toast.error(res.error ?? t('edit.error'))
          return
        }
        toast.success(t('edit.success'))
        onOpenChange(false)
        onSaved()
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setSubmitting(false))
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="font-display text-xl font-normal">{t('edit.title')}</DialogTitle>
          <DialogDescription>{t('edit.description')}</DialogDescription>
        </DialogHeader>

        <div className="max-h-[60vh] space-y-4 overflow-y-auto">
          {mark.kind !== 'figurative' && (
            <div className="space-y-1.5">
              <Label htmlFor="edit-text">{t('add.text')}</Label>
              <Input id="edit-text" value={text} onChange={(e) => setText(e.target.value)} disabled={submitting} data-testid="edit-text-input" />
            </div>
          )}

          <div className="space-y-2">
            <div className="flex items-baseline justify-between">
              <Label htmlFor="edit-threshold">{t('detail.threshold')}</Label>
              <span className="text-primary font-mono text-sm font-semibold tabular-nums">{Math.round(threshold * 100)}%</span>
            </div>
            <input
              id="edit-threshold"
              type="range"
              min={0.3}
              max={0.95}
              step={0.05}
              value={threshold}
              onChange={(e) => setThreshold(parseFloat(e.target.value))}
              disabled={submitting}
              className="accent-primary h-2 w-full cursor-pointer"
              data-testid="edit-threshold"
            />
            <p className="text-muted-foreground text-xs">{t('edit.thresholdHint')}</p>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="edit-nice">{t('add.nice')}</Label>
            <Input id="edit-nice" placeholder="9, 42" value={nice} onChange={(e) => setNice(e.target.value)} disabled={submitting} data-testid="edit-nice-input" />
          </div>

          <div className="space-y-1.5">
            <Label>{t('add.territory')}</Label>
            <div className="flex gap-2">
              {TERRITORIES.map((terr) => (
                <button
                  key={terr}
                  type="button"
                  onClick={() => toggleTerritory(terr)}
                  disabled={submitting}
                  data-testid={`edit-territory-${terr}`}
                  aria-pressed={territories.includes(terr)}
                  className={`rounded-md border px-3 py-1.5 text-sm transition-colors ${
                    territories.includes(terr) ? 'border-primary bg-secondary text-primary' : 'text-muted-foreground'
                  }`}
                >
                  {t(`territory.${terr}`)}
                </button>
              ))}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <Label htmlFor="edit-owner">{t('add.owner')}</Label>
              <Input id="edit-owner" value={owner} onChange={(e) => setOwner(e.target.value)} disabled={submitting} />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="edit-label">{t('add.label')}</Label>
              <Input id="edit-label" value={label} onChange={(e) => setLabel(e.target.value)} disabled={submitting} data-testid="edit-label-input" />
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={submitting}>{t('common.cancel')}</Button>
          <Button onClick={save} disabled={submitting} data-testid="edit-save">
            {submitting ? t('edit.saving') : t('edit.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
