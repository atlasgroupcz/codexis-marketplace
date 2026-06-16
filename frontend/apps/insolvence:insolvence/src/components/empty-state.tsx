import { useTranslation } from 'react-i18next'
import { Plus } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { PickSubjectArt, WatchRegisterArt } from './illustrations'

export function EmptySubjects({ onAdd }: { onAdd: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="flex max-w-md flex-col items-center text-center">
        <WatchRegisterArt className="mb-6 h-44 w-auto" />
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
    <div className="text-muted-foreground hidden h-full flex-col items-center justify-center gap-4 p-8 text-center md:flex">
      <PickSubjectArt className="h-32 w-auto" />
      <p className="max-w-[16rem] text-sm text-pretty">{t('empty.noSelection')}</p>
    </div>
  )
}
