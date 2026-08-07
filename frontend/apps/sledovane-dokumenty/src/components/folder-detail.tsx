import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  ArrowLeft,
  CheckCircle2,
  ExternalLink,
  FileText,
  Loader2,
  RefreshCw,
  Scale,
  ScanSearch,
  Trash2,
} from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Badge } from '@workspace/ui/components/badge'
import { Progress } from '@workspace/ui/components/progress'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@workspace/ui/components/tabs'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { HeaderGear } from '@workspace/ui/components/app-header'
import { DetailSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import type { TrackedLegislation, WatchedDocument } from '@/lib/schemas'
import { useFolderDetail } from '@/hooks/use-folder-detail'
import { postFolderAction } from '@/lib/api'

interface FolderDetailProps {
  root: string
  onBack: () => void
  onOpenSettings: () => void
}

export function FolderDetail({ root, onBack, onOpenSettings }: FolderDetailProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useFolderDetail(root)
  const [busy, setBusy] = useState<string | null>(null)

  const folder = data?.folder
  const pending = folder?.pending_harvest ?? 0

  // While extraction is running, poll for progress.
  useEffect(() => {
    if (pending <= 0) {
      return
    }
    const id = window.setInterval(refetch, 4000)
    return () => window.clearInterval(id)
  }, [pending, refetch])

  const run = (action: 'folder_refresh' | 'folder_check', successKey: string) => {
    setBusy(action)
    postFolderAction(action, { root })
      .then(() => {
        toast.success(t(successKey))
        window.setTimeout(refetch, 800)
      })
      .catch((err: unknown) => toast.error(err instanceof Error ? err.message : String(err)))
      .finally(() => setBusy(null))
  }

  const remove = () => {
    setBusy('remove')
    postFolderAction('folder_remove', { root })
      .then(() => {
        toast.success(t('folders.removed'))
        onBack()
      })
      .catch((err: unknown) => toast.error(err instanceof Error ? err.message : String(err)))
      .finally(() => setBusy(null))
  }

  const confirm = (codexisId: string | null | undefined) => {
    postFolderAction('folder_confirm', { root, codexisId: codexisId ?? undefined })
      .then(() => {
        toast.success(t('folders.confirmed'))
        refetch()
      })
      .catch((err: unknown) => toast.error(err instanceof Error ? err.message : String(err)))
  }

  const extracted = (folder?.documents_count ?? 0) - pending
  const progress = folder && folder.documents_count > 0 ? (extracted / folder.documents_count) * 100 : 0

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="folder-detail-root">
      <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b px-4">
        <div className="flex min-w-0 items-center gap-2">
          <Button variant="ghost" size="icon" onClick={onBack} aria-label={t('folders.back')}>
            <ArrowLeft className="size-4" />
          </Button>
          <div className="min-w-0">
            <h1 className="font-display truncate text-[15px] leading-tight">{folder?.name ?? '…'}</h1>
            <p className="text-muted-foreground truncate font-mono text-[11px]" title={folder?.root}>
              {folder?.root ?? root}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => run('folder_check', 'folders.checkStarted')}
            disabled={busy !== null}
            aria-label={t('folders.checkNow')}
          >
            <ScanSearch className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => run('folder_refresh', 'folders.refreshStarted')}
            disabled={busy !== null}
            aria-label={t('folders.refresh')}
          >
            <RefreshCw className={`size-4 ${busy === 'folder_refresh' ? 'animate-spin' : ''}`} />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={remove}
            disabled={busy !== null}
            aria-label={t('folders.remove')}
          >
            <Trash2 className="size-4" />
          </Button>
          <HeaderGear onClick={onOpenSettings} title={t('notify.title')} />
        </div>
      </header>

      <main className="min-h-0 flex-1">
        {loading && !data ? (
          <div className="mx-auto max-w-3xl p-4 sm:p-6">
            <DetailSkeleton />
          </div>
        ) : error ? (
          <div className="p-6">
            <ErrorMessage message={error.message} onRetry={refetch} />
          </div>
        ) : folder ? (
          <ScrollArea className="h-full">
            <div className="mx-auto max-w-3xl space-y-5 p-4 sm:p-6">
              {pending > 0 && (
                <div className="bg-secondary/50 space-y-2 rounded-xl border p-4">
                  <div className="flex items-center gap-2 text-sm font-medium">
                    <Loader2 className="text-primary size-4 animate-spin" />
                    {t('folders.extractingProgress', { done: extracted, total: folder.documents_count })}
                  </div>
                  <Progress value={progress} />
                </div>
              )}

              <Tabs defaultValue="legislation">
                <TabsList variant="line" className="w-full justify-start gap-4">
                  <TabsTrigger value="legislation" className="flex-none px-0">
                    {t('folders.tabs.legislation')}
                    <span className="bg-secondary text-secondary-foreground ml-1.5 inline-flex min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-semibold">
                      {folder.legislation.length}
                    </span>
                  </TabsTrigger>
                  <TabsTrigger value="documents" className="flex-none px-0">
                    {t('folders.tabs.documents')}
                    <span className="bg-secondary text-secondary-foreground ml-1.5 inline-flex min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-semibold">
                      {folder.documents_count}
                    </span>
                  </TabsTrigger>
                </TabsList>

                <TabsContent value="legislation" className="space-y-2 pt-4">
                  {folder.legislation.length === 0 ? (
                    <EmptyHint text={pending > 0 ? t('folders.legislationExtracting') : t('folders.noLegislation')} />
                  ) : (
                    folder.legislation.map((leg, i) => (
                      <LegislationItem key={leg.codexisId ?? leg.uri ?? i} leg={leg} onConfirm={() => confirm(leg.codexisId)} />
                    ))
                  )}
                </TabsContent>

                <TabsContent value="documents" className="space-y-2 pt-4">
                  {folder.documents.map((doc) => (
                    <DocumentItem key={doc.path} doc={doc} />
                  ))}
                </TabsContent>
              </Tabs>
            </div>
          </ScrollArea>
        ) : null}
      </main>
      <Toaster position="bottom-right" />
    </div>
  )
}

function codexisDocUrl(codexisId: string): string {
  return `https://next.codexis.cz/doc/${codexisId}`
}

function LegislationItem({ leg, onConfirm }: { leg: TrackedLegislation; onConfirm: () => void }) {
  const { t } = useTranslation()
  const changes = leg.changes
  const unconfirmed = changes.filter((c) => !c.confirmed_on)
  const latest = changes[changes.length - 1] as (typeof changes)[number] | undefined

  return (
    <div className="space-y-2 rounded-xl border p-3.5">
      <div className="flex items-start justify-between gap-2">
        <div className="flex min-w-0 items-start gap-2">
          <Scale className="text-muted-foreground mt-0.5 size-4 shrink-0" />
          <div className="min-w-0">
            <p className="text-sm leading-snug">{leg.text}</p>
            {leg.codexisId && (
              <a
                href={codexisDocUrl(leg.codexisId)}
                target="_blank"
                rel="noreferrer"
                className="text-muted-foreground hover:text-primary inline-flex items-center gap-1 truncate text-xs"
              >
                <span className="truncate">{codexisDocUrl(leg.codexisId)}</span>
                <ExternalLink className="size-3 shrink-0" />
              </a>
            )}
          </div>
        </div>
        {unconfirmed.length > 0 ? (
          <Badge className="shrink-0">{t('folders.changed')}</Badge>
        ) : leg.tracked ? (
          <Badge variant="secondary" className="shrink-0">
            {t('folders.tracked')}
          </Badge>
        ) : (
          <Badge variant="outline" className="shrink-0">
            {t('folders.untracked')}
          </Badge>
        )}
      </div>

      {unconfirmed.length > 0 && latest && (
        <div className="bg-secondary/40 space-y-2 rounded-lg p-2.5 text-xs">
          {latest.effective_on && (
            <p className="text-muted-foreground">
              {t('folders.effectiveFrom', { date: latest.effective_on })}
            </p>
          )}
          <div className="flex flex-wrap items-center gap-2">
            {latest.compare_url && (
              <a
                href={latest.compare_url}
                target="_blank"
                rel="noreferrer"
                className="text-primary inline-flex items-center gap-1 font-medium"
              >
                {t('folders.viewInCodexis')}
                <ExternalLink className="size-3" />
              </a>
            )}
            <Button size="sm" variant="outline" className="h-7" onClick={onConfirm}>
              <CheckCircle2 className="size-3.5" />
              {t('folders.confirm')}
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}

function DocumentItem({ doc }: { doc: WatchedDocument }) {
  const { t } = useTranslation()
  const extracted = doc.extractedAt !== null
  return (
    <div className="flex items-center justify-between gap-3 rounded-xl border p-3">
      <div className="flex min-w-0 items-center gap-2">
        <FileText className="text-muted-foreground size-4 shrink-0" />
        <div className="min-w-0">
          <p className="truncate font-mono text-xs">{doc.path}</p>
          <p className="text-muted-foreground text-xs">
            {t('folders.refCount', { count: doc.legislation.length })}
          </p>
        </div>
      </div>
      {extracted ? (
        <Badge variant="secondary" className="shrink-0">
          <CheckCircle2 className="size-3" />
          {t('folders.extracted')}
        </Badge>
      ) : (
        <Badge variant="outline" className="text-muted-foreground shrink-0">
          {t('folders.pending')}
        </Badge>
      )}
    </div>
  )
}

function EmptyHint({ text }: { text: string }) {
  return <p className="text-muted-foreground py-8 text-center text-sm">{text}</p>
}
