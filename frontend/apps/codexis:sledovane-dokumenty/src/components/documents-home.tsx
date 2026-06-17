import { useQueryState } from 'nuqs'
import { useTranslation } from 'react-i18next'
import { BookMarked, RefreshCw } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@workspace/ui/components/tabs'
import { Toaster } from '@workspace/ui/components/sonner'
import { ChangesFeed } from './changes-feed'
import { DocumentGrid } from './document-grid'
import { EmptyDocuments } from './empty-state'
import { FeedSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { SectionSwitch } from './section-switch'
import { useOverview } from '@/hooks/use-overview'
import { tabParser } from '@/lib/url-state'

interface DocumentsHomeProps {
  section: 'documents' | 'folders'
  onChangeSection: (section: 'documents' | 'folders') => void
  onSelectDocument: (uuid: string) => void
}

export function DocumentsHome({ section, onChangeSection, onSelectDocument }: DocumentsHomeProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const [tab, setTab] = useQueryState('tab', tabParser)

  const documents = data?.tracked_documents ?? []
  const groups = data?.groups ?? []
  const unconfirmed = documents.reduce((sum, d) => sum + d.unconfirmed_changes, 0)

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="app-root">
      <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b px-4">
        <div className="flex min-w-0 items-center gap-2.5">
          <span className="bg-secondary text-primary ring-brand-100 flex size-9 items-center justify-center rounded-xl ring-1">
            <BookMarked className="size-5" strokeWidth={1.75} />
          </span>
          <div className="min-w-0">
            <h1 className="font-display text-foreground truncate text-[15px] leading-tight">
              {t('followedDocs.title')}
            </h1>
            <p className="text-muted-foreground hidden truncate text-xs sm:block">
              {t('followedDocs.subtitle')}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <SectionSwitch section={section} onChange={onChangeSection} />
          {documents.length > 0 && (
            <Button variant="outline" size="sm" onClick={refetch} disabled={loading}>
              <RefreshCw className={`size-4 ${loading ? 'animate-spin' : ''}`} />
              <span className="hidden sm:inline">{t('followedDocs.refresh')}</span>
            </Button>
          )}
        </div>
      </header>

      <main className="min-h-0 flex-1">
        {loading && !data ? (
          <div className="mx-auto max-w-5xl p-4 sm:p-6">
            <FeedSkeleton />
          </div>
        ) : error ? (
          <div className="p-6">
            <ErrorMessage message={error.message} onRetry={refetch} />
          </div>
        ) : documents.length === 0 ? (
          <EmptyDocuments />
        ) : (
          <ScrollArea className="h-full">
            <div className="mx-auto max-w-5xl space-y-5 p-4 sm:p-6">
              <Tabs value={tab} onValueChange={(v) => void setTab(v as 'changes' | 'documents')}>
                <TabsList variant="line" className="w-full justify-start gap-4">
                  <TabsTrigger value="changes" className="flex-none px-0">
                    {t('followedDocs.tabs.changes')}
                    {unconfirmed > 0 && (
                      <span className="bg-primary text-primary-foreground ml-1.5 inline-flex min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-semibold">
                        {unconfirmed}
                      </span>
                    )}
                  </TabsTrigger>
                  <TabsTrigger value="documents" className="flex-none px-0">
                    {t('followedDocs.tabs.documents')}
                    <span className="bg-secondary text-secondary-foreground ml-1.5 inline-flex min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-semibold">
                      {documents.length}
                    </span>
                  </TabsTrigger>
                </TabsList>
                <TabsContent value="changes" className="pt-4">
                  <ChangesFeed documents={documents} onSelectDocument={onSelectDocument} />
                </TabsContent>
                <TabsContent value="documents" className="pt-4">
                  <DocumentGrid
                    documents={documents}
                    groups={groups}
                    onSelectDocument={onSelectDocument}
                    onMutate={refetch}
                  />
                </TabsContent>
              </Tabs>
            </div>
          </ScrollArea>
        )}
      </main>

      <Toaster position="bottom-right" />
    </div>
  )
}
