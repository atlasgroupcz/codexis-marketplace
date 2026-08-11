import { Button } from '@workspace/ui/components/button'
import { ErrorMessage } from '@workspace/ui/components/error-message'
import { toast } from '@workspace/ui/components/sonner'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@workspace/ui/components/tabs'
import { useConfirmAction } from '@workspace/ui/hooks/use-confirm-action'
import { ArrowLeft, ExternalLink, FileStack, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useDetail } from '@/hooks/use-detail'
import { removeSubjekt } from '@/lib/api'
import { REGISTER_SUBJECT_URL } from '@/lib/constants'
import { errorMessage, formatRelative } from '@/lib/format'
import { ChangeHistory } from './detail/change-history'
import { ListinyList } from './listiny-list'
import { DetailSkeleton } from './loading-skeleton'

interface SubjektDetailProps {
  ico: string
  onBack: () => void
  onChanged?: () => void
}

export function SubjektDetail({ ico, onBack, onChanged }: SubjektDetailProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useDetail(ico)

  const remove = useConfirmAction(() =>
    removeSubjekt(ico)
      .then(() => {
        onChanged?.()
        onBack()
      })
      .catch((e: unknown) => toast.error(errorMessage(e))),
  )

  if (loading && !data) return <DetailSkeleton />
  if (error)
    return (
      <div className="p-6">
        <ErrorMessage message={error.message} onRetry={refetch} retryLabel={t('common.retry')} />
      </div>
    )
  if (!data) return null

  const subjekt = data.subjekt
  const name = subjekt.nazev || subjekt.ico
  const unconfirmed = subjekt.changes.filter((c) => !c.confirmed_on).length

  return (
    <div className="space-y-5 p-4 sm:p-6">
      {/* Header */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex min-w-0 items-start gap-3">
          <Button
            variant="ghost"
            size="icon"
            className="md:hidden"
            onClick={onBack}
            data-testid="back-to-list"
          >
            <ArrowLeft className="size-4" />
          </Button>
          <span className="bg-secondary text-primary ring-brand-100 flex size-10 shrink-0 items-center justify-center rounded-xl ring-1">
            <FileStack className="size-5" strokeWidth={1.75} />
          </span>
          <div className="min-w-0">
            <h1 className="truncate text-xl font-semibold leading-tight">{name}</h1>

            <p className="text-muted-foreground mt-1 text-[13px]">
              <span className="font-mono">IČO {subjekt.ico}</span>
              {subjekt.spisovaznacka && (
                <>
                  <span className="text-border mx-1.5">·</span>
                  <span className="font-mono">{subjekt.spisovaznacka}</span>
                </>
              )}
            </p>
            {subjekt.sidlo && (
              <p className="text-muted-foreground mt-0.5 text-[13px]">{subjekt.sidlo}</p>
            )}
            <p className="text-muted-foreground mt-2 text-[13px]">
              {t('subjekty.lastCheck')}{' '}
              <span className="text-foreground">{formatRelative(subjekt.last_check_at)}</span>
              <span className="text-border mx-1.5">·</span>
              <a
                href={REGISTER_SUBJECT_URL(subjekt.ico)}
                target="_blank"
                rel="noopener noreferrer"
                className="text-brand-700 inline-flex items-center gap-1 align-baseline hover:underline"
                data-testid="open-register"
              >
                {t('subjekty.openRegister')}
                <ExternalLink className="size-3" />
              </a>
            </p>
          </div>
        </div>
        <Button
          variant={remove.confirming ? 'destructive' : 'outline'}
          size="sm"
          onClick={remove.trigger}
          disabled={remove.pending}
          data-testid="remove-subjekt"
          className="shrink-0"
        >
          <Trash2 className="size-4" />
          <span className="hidden sm:inline">
            {remove.confirming ? t('subjekty.confirmRemove') : t('subjekty.remove')}
          </span>
        </Button>
      </div>

      {/* Listiny / Historie změn */}
      <Tabs defaultValue="listiny" className="gap-4">
        <TabsList variant="line" className="w-full justify-start gap-4">
          <TabsTrigger value="listiny" className="flex-none px-0" data-testid="tab-listiny">
            {t('subjekty.listinyTab')}
            <CountChip>{subjekt.listiny.length}</CountChip>
          </TabsTrigger>
          <TabsTrigger value="zmeny" className="flex-none px-0" data-testid="tab-zmeny">
            {t('subjekty.changeHistory')}
            {unconfirmed > 0 && <CountChip>{unconfirmed}</CountChip>}
          </TabsTrigger>
        </TabsList>
        <TabsContent value="listiny">
          <ListinyList ico={subjekt.ico} listiny={subjekt.listiny} onChanged={refetch} />
        </TabsContent>
        <TabsContent value="zmeny">
          <ChangeHistory
            ico={ico}
            changes={subjekt.changes}
            onConfirmed={() => {
              refetch()
              onChanged?.()
            }}
          />
        </TabsContent>
      </Tabs>
    </div>
  )
}

function CountChip({ children }: { children: React.ReactNode }) {
  return (
    <span className="bg-secondary text-secondary-foreground ml-1.5 inline-flex min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-semibold">
      {children}
    </span>
  )
}
