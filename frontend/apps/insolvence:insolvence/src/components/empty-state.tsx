import { useTranslation } from 'react-i18next'
import { Plus, ShieldQuestion } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'

export function EmptySubjects({ onAdd }: { onAdd: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="flex max-w-md flex-col items-center text-center">
        <span className="bg-secondary text-primary ring-brand-100 mb-6 flex size-16 items-center justify-center rounded-2xl ring-1">
          <ShieldQuestion className="size-8" strokeWidth={1.5} />
        </span>
        <p className="text-primary mb-3 text-xs font-semibold tracking-[0.14em] uppercase">
          {t('app.title')}
        </p>
        <h2 className="font-display text-foreground text-3xl leading-tight text-balance">
          {t('empty.title')}
        </h2>
        <p className="text-muted-foreground mt-3 text-[15px] leading-relaxed text-pretty">
          {t('empty.text')}
        </p>
        <Button onClick={onAdd} size="lg" className="mt-7" data-testid="empty-add">
          <Plus className="size-4" />
          {t('empty.add')}
        </Button>
      </div>
    </div>
  )
}

export function NoSelection() {
  const { t } = useTranslation()
  return (
    <div className="text-muted-foreground hidden h-full flex-col items-center justify-center gap-3 p-8 text-center md:flex">
      <ShieldQuestion className="size-8 opacity-30" strokeWidth={1.5} />
      <p className="max-w-[16rem] text-sm text-pretty">{t('empty.noSelection')}</p>
    </div>
  )
}
