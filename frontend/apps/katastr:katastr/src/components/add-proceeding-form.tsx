import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, RefreshCw } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { toast } from '@workspace/ui/components/sonner'
import { addProceeding, checkAll } from '@/lib/api'

interface AddProceedingFormProps {
  onAdded: () => void
  onChecked: (newChanges: number, errors: number) => void
}

export function AddProceedingForm({ onAdded, onChecked }: AddProceedingFormProps) {
  const { t } = useTranslation()
  const [cislo, setCislo] = useState('')
  const [adding, setAdding] = useState(false)
  const [checking, setChecking] = useState(false)

  const handleAdd = () => {
    const value = cislo.trim()
    if (!value) return
    setAdding(true)
    addProceeding(value)
      .then((res) => {
        if (res.ok) {
          setCislo('')
          onAdded()
        } else {
          toast.error(res.error ?? t('proceedings.addError'))
        }
      })
      .catch((e: unknown) => {
        toast.error(e instanceof Error ? e.message : String(e))
      })
      .finally(() => setAdding(false))
  }

  const handleCheck = () => {
    setChecking(true)
    checkAll()
      .then((res) => {
        onChecked(res.new_changes, res.errors.length)
      })
      .catch((e: unknown) => {
        toast.error(e instanceof Error ? e.message : String(e))
      })
      .finally(() => setChecking(false))
  }

  return (
    <div className="space-y-2">
      <label htmlFor="cislo" className="text-sm font-medium">
        {t('proceedings.addLabel')}
      </label>
      <div className="flex flex-col gap-2 sm:flex-row">
        <input
          id="cislo"
          type="text"
          autoComplete="off"
          spellCheck={false}
          className="border-input bg-background flex-1 rounded-md border px-3 py-2 font-mono text-sm"
          placeholder="V-1234/2026-701"
          value={cislo}
          onChange={(e) => setCislo(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleAdd()}
          disabled={adding}
          data-testid="add-proceeding-input"
        />
        <div className="flex gap-2">
          <Button
            onClick={handleAdd}
            disabled={adding || !cislo.trim()}
            data-testid="add-proceeding-button"
          >
            <Plus className="size-4" />
            {adding ? t('proceedings.adding') : t('proceedings.add')}
          </Button>
          <Button
            variant="outline"
            onClick={handleCheck}
            disabled={checking}
            data-testid="check-all-button"
          >
            <RefreshCw className={`size-4 ${checking ? 'animate-spin' : ''}`} />
            {checking ? t('proceedings.checking') : t('proceedings.checkAll')}
          </Button>
        </div>
      </div>
    </div>
  )
}
