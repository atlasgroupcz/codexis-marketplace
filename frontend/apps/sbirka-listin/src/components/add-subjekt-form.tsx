import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { toast } from '@workspace/ui/components/sonner'
import { addSubjekt } from '@/lib/api'
import { errorMessage } from '@/lib/format'

interface AddSubjektFormProps {
  onAdded: () => void
}

export function AddSubjektForm({ onAdded }: AddSubjektFormProps) {
  const { t } = useTranslation()
  const [ico, setIco] = useState('')
  const [adding, setAdding] = useState(false)

  const handleAdd = () => {
    const value = ico.trim()
    if (!value) return
    setAdding(true)
    addSubjekt(value)
      .then(() => {
        setIco('')
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
        id="ico"
        type="text"
        inputMode="numeric"
        autoComplete="off"
        spellCheck={false}
        className="border-input bg-background w-44 rounded-md border px-2.5 py-1.5 font-mono text-sm lg:w-52"
        placeholder={t('subjekty.icoPlaceholder')}
        value={ico}
        onChange={(e) => setIco(e.target.value)}
        onKeyDown={(e) => e.key === 'Enter' && handleAdd()}
        disabled={adding}
        data-testid="add-subjekt-input"
      />
      <Button
        size="sm"
        onClick={handleAdd}
        disabled={adding || !ico.trim()}
        data-testid="add-subjekt-button"
      >
        <Plus className="size-4" />
        <span className="hidden sm:inline">
          {adding ? t('subjekty.adding') : t('subjekty.add')}
        </span>
      </Button>
    </div>
  )
}
