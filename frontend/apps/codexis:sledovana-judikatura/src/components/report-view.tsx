import { useTranslation } from 'react-i18next'
import { Check, ExternalLink } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Badge } from '@workspace/ui/components/badge'
import { AppHeader, HeaderGear } from '@workspace/ui/components/app-header'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@workspace/ui/components/tabs'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@workspace/ui/components/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@workspace/ui/components/table'
import { LoadingSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { useReport } from '@/hooks/use-report'
import { postAction } from '@/lib/api'
import { formatDate } from '@/lib/format'

interface ReportViewProps {
  uuid: string
  reportId: string
  onBack: () => void
  onOpenSettings: () => void
}

function codexisUrl(codexisId: string): string {
  return `https://next.codexis.cz/doc/${codexisId}`
}

export function ReportView({ uuid, reportId, onBack, onOpenSettings }: ReportViewProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useReport(uuid, reportId)

  if (loading && !data) return <LoadingSkeleton />
  if (error) {
    return (
      <div className="p-6">
        <ErrorMessage error={error} onRetry={refetch} />
      </div>
    )
  }
  if (!data) return null

  const topic = data.topic
  const report = data.report

  const handleConfirm = async () => {
    try {
      await postAction({ action: 'confirm_report', uuid, report_id: reportId })
      toast.success(t('judikatura.confirmed'))
      refetch()
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : String(e))
    }
  }

  const allDocs = report.areas.flatMap((area) =>
    area.documents.map((doc) => ({ ...doc, areaName: area.name })),
  )

  const summaryTabs = [
    { value: 'lawyer', label: t('judikatura.summaryForLawyer'), text: report.summary_for_lawyer },
    { value: 'hr', label: t('judikatura.summaryForHr'), text: report.summary_for_hr },
    { value: 'en', label: t('judikatura.executiveSummary'), text: report.summary_en },
  ].filter((tab): tab is { value: string; label: string; text: string } => Boolean(tab.text))

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="app-root">
      <AppHeader
        back={{
          onClick: onBack,
          label: t('judikatura.back'),
          title: `${t('judikatura.report')} ${report.checked_at ? formatDate(report.checked_at) : reportId}`,
          subtitle: topic.name,
        }}
        actions={
          <>
            {report.confirmed_on ? (
              <Badge variant="outline" className="shrink-0">
                <Check className="size-3.5" />
                {t('judikatura.confirmed')}
              </Badge>
            ) : (
              <Button size="sm" onClick={() => void handleConfirm()}>
                {t('judikatura.confirmReport')}
              </Button>
            )}
            <HeaderGear onClick={onOpenSettings} title={t('judikatura.notify.title')} />
          </>
        }
      />

      <main className="min-h-0 flex-1">
        <ScrollArea className="h-full">
          <div className="mx-auto max-w-3xl space-y-5 p-4 sm:p-6">
            <div className="text-muted-foreground flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
              <span>
                {t('judikatura.period')}: {report.period_from ? formatDate(report.period_from) : '-'}
                {' – '}
                {report.period_to ? formatDate(report.period_to) : '-'}
              </span>
              <span className="text-border">·</span>
              <span>{t('judikatura.foundCount', { count: report.found_count })}</span>
            </div>

            {report.overall_summary && (
              <div className="border-l-primary bg-secondary/40 rounded-r-lg border-l-4 p-4 text-sm leading-relaxed">
                {report.overall_summary}
              </div>
            )}

            <Tabs defaultValue="overview">
              <TabsList variant="line" className="w-full justify-start gap-4">
                <TabsTrigger value="overview" className="flex-none px-0">
                  {t('judikatura.overviewTable')}
                </TabsTrigger>
                {summaryTabs.map((tab) => (
                  <TabsTrigger key={tab.value} value={tab.value} className="flex-none px-0">
                    {tab.label}
                  </TabsTrigger>
                ))}
              </TabsList>

              <TabsContent value="overview" className="space-y-6 pt-4">
                {allDocs.length > 0 && (
                  <div className="rounded-xl border" data-testid="decisions-table">
                    <Table>
                      <TableHeader>
                        <TableRow className="bg-muted/50">
                          <TableHead>{t('judikatura.date')}</TableHead>
                          <TableHead>{t('judikatura.caseNumber')}</TableHead>
                          <TableHead>{t('judikatura.court')}</TableHead>
                          <TableHead>{t('judikatura.area')}</TableHead>
                          <TableHead>{t('judikatura.interpretation')}</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {allDocs.map((doc, i) => (
                          <TableRow key={i}>
                            <TableCell className="text-muted-foreground">
                              {doc.issued_on ? formatDate(doc.issued_on) : '-'}
                            </TableCell>
                            <TableCell>
                              {doc.codexisId ? (
                                <a
                                  href={codexisUrl(doc.codexisId)}
                                  target="_blank"
                                  rel="noopener noreferrer"
                                  className="text-primary font-medium hover:underline"
                                >
                                  {doc.spZn || doc.title}
                                </a>
                              ) : (
                                <span className="font-medium">{doc.spZn || doc.title}</span>
                              )}
                            </TableCell>
                            <TableCell>{doc.court}</TableCell>
                            <TableCell className="text-muted-foreground">{doc.areaName}</TableCell>
                            <TableCell>
                              {doc.changes_established_view ? (
                                <Badge className="bg-amber-100 text-amber-800">
                                  {t('judikatura.changesView')}
                                </Badge>
                              ) : (
                                <span className="text-muted-foreground text-xs">
                                  {t('judikatura.confirmsView')}
                                </span>
                              )}
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                )}

                {report.areas.map((area) => (
                  <Card key={area.name} className="gap-0 py-0">
                    <CardHeader className="gap-1 border-b py-4">
                      <CardTitle className="font-display text-base">{area.name}</CardTitle>
                      {area.area_summary && <CardDescription>{area.area_summary}</CardDescription>}
                    </CardHeader>
                    <CardContent className="divide-y px-0">
                      {area.documents.map((doc, i) => (
                        <div key={i} className="px-6 py-4">
                          <div className="flex items-start justify-between gap-3">
                            <div className="space-y-1">
                              <div className="font-medium">
                                {doc.codexisId ? (
                                  <a
                                    href={codexisUrl(doc.codexisId)}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="text-primary hover:underline"
                                  >
                                    {doc.spZn} - {doc.court}
                                    <ExternalLink className="ml-1 inline size-3" />
                                  </a>
                                ) : (
                                  <>
                                    {doc.spZn} - {doc.court}
                                  </>
                                )}
                              </div>
                              <div className="text-muted-foreground flex items-center gap-2 text-xs">
                                {doc.doc_type && <span>{doc.doc_type}</span>}
                                {doc.issued_on && (
                                  <>
                                    <span className="text-border">·</span>
                                    <span>{formatDate(doc.issued_on)}</span>
                                  </>
                                )}
                              </div>
                            </div>
                            {doc.changes_established_view ? (
                              <Badge className="shrink-0 bg-amber-100 text-amber-800">
                                {t('judikatura.changesView')}
                              </Badge>
                            ) : (
                              <span className="text-muted-foreground shrink-0 text-xs">
                                {t('judikatura.confirmsView')}
                              </span>
                            )}
                          </div>
                          {doc.legal_sentence && (
                            <p className="bg-muted/30 text-muted-foreground mt-3 rounded-md px-3 py-2 text-xs italic">
                              {doc.legal_sentence}
                            </p>
                          )}
                          {doc.summary && <p className="mt-3 text-sm leading-relaxed">{doc.summary}</p>}
                          {doc.practical_conclusion && (
                            <p className="text-muted-foreground mt-2 text-xs">
                              <strong>{t('judikatura.practical')}:</strong> {doc.practical_conclusion}
                            </p>
                          )}
                        </div>
                      ))}
                    </CardContent>
                  </Card>
                ))}
              </TabsContent>

              {summaryTabs.map((tab) => (
                <TabsContent key={tab.value} value={tab.value} className="pt-4">
                  <div className="border-l-primary bg-secondary/30 rounded-r-lg border-l-4 p-4 text-sm leading-relaxed whitespace-pre-line">
                    {tab.text}
                  </div>
                </TabsContent>
              ))}
            </Tabs>
          </div>
        </ScrollArea>
      </main>

      <Toaster position="bottom-right" />
    </div>
  )
}
