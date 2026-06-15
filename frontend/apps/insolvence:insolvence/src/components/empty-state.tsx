import { useTranslation } from 'react-i18next'
import { Plus, ShieldQuestion } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'

export function EmptySubjects({ onAdd }: { onAdd: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-8 text-center">
      <div className="bg-muted flex size-14 items-center justify-center rounded-2xl">
        <ShieldQuestion className="text-muted-foreground size-7" />
      </div>
      <div className="space-y-1">
        <h2 className="text-base font-semibold">{t('empty.title')}</h2>
        <p className="text-muted-foreground max-w-xs text-sm">{t('empty.text')}</p>
      </div>
      <Button onClick={onAdd} data-testid="empty-add">
        <Plus className="size-4" />
        {t('empty.add')}
      </Button>
    </div>
  )
}

export function NoSelection() {
  const { t } = useTranslation()
  return (
    <div className="text-muted-foreground hidden h-full flex-col items-center justify-center gap-3 p-8 text-center text-sm md:flex">
      <ShieldQuestion className="size-8 opacity-40" />
      <p>{t('empty.noSelection')}</p>
    </div>
  )
}
