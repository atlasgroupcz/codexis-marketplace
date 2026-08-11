import { Button } from '@workspace/ui/components/button'
import { toast } from '@workspace/ui/components/sonner'
import { Plus } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { addProceeding } from '@/lib/api'
import { errorMessage } from '@/lib/format'

interface AddProceedingFormProps {
  onAdded: () => void
}

export function AddProceedingForm({ onAdded }: AddProceedingFormProps) {
  const { t } = useTranslation()
  const [cislo, setCislo] = useState('')
  const [adding, setAdding] = useState(false)

  const handleAdd = () => {
    const value = cislo.trim()
    if (!value) return
    setAdding(true)
    addProceeding(value)
      .then(() => {
        setCislo('')
        onAdded()
      })
      .catch((e: unknown) => {
        toast.error(errorMessage(e))
      })
      .finally(() => setAdding(false))
  }

  return (
    <div className="flex items-center gap-2">
      <input
        id="cislo"
        type="text"
        autoComplete="off"
        spellCheck={false}
        className="border-input bg-background w-60 rounded-md border px-2.5 py-1.5 font-mono text-sm lg:w-80"
        placeholder="V-1234/2026-701"
        value={cislo}
        onChange={(e) => setCislo(e.target.value)}
        onKeyDown={(e) => e.key === 'Enter' && handleAdd()}
        disabled={adding}
        data-testid="add-proceeding-input"
      />
      <Button
        size="sm"
        onClick={handleAdd}
        disabled={adding || !cislo.trim()}
        data-testid="add-proceeding-button"
      >
        <Plus className="size-4" />
        <span className="hidden sm:inline">
          {adding ? t('proceedings.adding') : t('proceedings.add')}
        </span>
      </Button>
    </div>
  )
}
