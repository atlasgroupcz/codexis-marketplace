import { useTranslation } from 'react-i18next'
import { ArrowLeft, ExternalLink } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { Badge } from '@workspace/ui/components/badge'
import { useReport } from '@/hooks/use-report'
import { postAction } from '@/lib/api'
import { formatDate } from '@/lib/format'
import { LoadingSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'

interface ReportViewProps {
  uuid: string
  reportId: string
  onBack: () => void
}

export function ReportView({ uuid, reportId, onBack }: ReportViewProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useReport(uuid, reportId)

  if (loading && !data) return <LoadingSkeleton />
  if (error) return <ErrorMessage error={error} onRetry={refetch} />
  if (!data) return null

  const topic = data.topic
  const report = data.report

  const handleConfirm = async () => {
    await postAction({ action: 'confirm_report', uuid, report_id: reportId })
    refetch()
  }

  // Collect all documents across areas for the table
  const allDocs = report.areas.flatMap((area) =>
    area.documents.map((doc) => ({ ...doc, areaName: area.name })),
  )

  return (
    <div className="space-y-6 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <Button variant="ghost" size="sm" onClick={onBack}>
          <ArrowLeft className="size-4" />
          {t('judikatura.backToTopic')}
        </Button>
        {report.confirmed_on ? (
          <Badge variant="outline">
            {t('judikatura.confirmed')} {formatDate(report.confirmed_on)}
          </Badge>
        ) : (
          <Button size="sm" onClick={() => void handleConfirm()}>
            {t('judikatura.confirmReport')}
          </Button>
        )}
      </div>

      {/* Title */}
      <div>
        <h1 className="text-xl font-semibold">
          {t('judikatura.report')} {report.checked_at ? formatDate(report.checked_at) : reportId}
        </h1>
        <div className="mt-1 flex items-center gap-2 text-sm text-muted-foreground">
          <span>{topic.name}</span>
          <span>&middot;</span>
          <span>
            {t('judikatura.period')}: {report.period_from ? formatDate(report.period_from) : '-'}
            {' \u2013 '}
            {report.period_to ? formatDate(report.period_to) : '-'}
          </span>
          <span>&middot;</span>
          <span>{t('judikatura.foundCount', { count: report.found_count })}</span>
        </div>
      </div>

      {/* Overall summary */}
      {report.overall_summary && (
        <div className="rounded-lg border-l-4 border-primary bg-primary/5 p-4 text-sm leading-relaxed">
          {report.overall_summary}
        </div>
      )}

      {/* Decisions table */}
      {allDocs.length > 0 && (
        <div>
          <h2 className="mb-3 font-semibold">{t('judikatura.overviewTable')}</h2>
          <div className="overflow-x-auto rounded-xl border shadow-sm">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b bg-muted/50">
                  <th className="px-4 py-3 text-left font-medium">{t('judikatura.date')}</th>
                  <th className="px-4 py-3 text-left font-medium">{t('judikatura.caseNumber')}</th>
                  <th className="px-4 py-3 text-left font-medium">{t('judikatura.court')}</th>
                  <th className="px-4 py-3 text-left font-medium">{t('judikatura.area')}</th>
                  <th className="px-4 py-3 text-left font-medium">{t('judikatura.interpretation')}</th>
                </tr>
              </thead>
              <tbody>
                {allDocs.map((doc, i) => (
                  <tr key={i} className="border-b last:border-0 even:bg-muted/20">
                    <td className="px-4 py-3 whitespace-nowrap">
                      {doc.issued_on ? formatDate(doc.issued_on) : '-'}
                    </td>
                    <td className="px-4 py-3">
                      {doc.codexisId ? (
                        <a
                          href={`https://next.codexis.cz/doc/${doc.codexisId}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="font-medium text-primary hover:underline"
                        >
                          {doc.spZn || doc.title}
                        </a>
                      ) : (
                        <span className="font-medium">{doc.spZn || doc.title}</span>
                      )}
                    </td>
                    <td className="px-4 py-3">{doc.court}</td>
                    <td className="px-4 py-3 text-muted-foreground">{doc.areaName}</td>
                    <td className="px-4 py-3">
                      {doc.changes_established_view ? (
                        <Badge className="bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200">
                          {t('judikatura.changesView')}
                        </Badge>
                      ) : (
                        <span className="text-xs text-muted-foreground">{t('judikatura.confirmsView')}</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Detail by area */}
      {report.areas.length > 0 && (
        <div>
          <h2 className="mb-3 font-semibold">{t('judikatura.detailByArea')}</h2>
          <div className="space-y-4">
            {report.areas.map((area) => (
              <div key={area.name} className="rounded-xl border bg-card shadow-sm">
                <div className="border-b px-5 py-3">
                  <h3 className="font-semibold">{area.name}</h3>
                  {area.area_summary && (
                    <p className="mt-1 text-sm text-muted-foreground">{area.area_summary}</p>
                  )}
                </div>
                <div className="divide-y">
                  {area.documents.map((doc, i) => (
                    <div key={i} className="px-5 py-4">
                      <div className="flex items-start justify-between gap-3">
                        <div className="space-y-1">
                          <div className="font-medium">
                            {doc.codexisId ? (
                              <a
                                href={`https://next.codexis.cz/doc/${doc.codexisId}`}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-primary hover:underline"
                              >
                                {doc.spZn} — {doc.court}
                                <ExternalLink className="ml-1 inline size-3" />
                              </a>
                            ) : (
                              <>{doc.spZn} — {doc.court}</>
                            )}
                          </div>
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            {doc.doc_type && <span>{doc.doc_type}</span>}
                            {doc.issued_on && (
                              <>
                                <span>&middot;</span>
                                <span>{formatDate(doc.issued_on)}</span>
                              </>
                            )}
                          </div>
                        </div>
                        {doc.changes_established_view ? (
                          <Badge className="shrink-0 bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200">
                            {t('judikatura.changesView')}
                          </Badge>
                        ) : (
                          <span className="shrink-0 text-xs text-muted-foreground">{t('judikatura.confirmsView')}</span>
                        )}
                      </div>
                      {doc.legal_sentence && (
                        <p className="mt-3 rounded-md bg-muted/30 px-3 py-2 text-xs italic text-muted-foreground">
                          {doc.legal_sentence}
                        </p>
                      )}
                      {doc.summary && (
                        <p className="mt-3 text-sm leading-relaxed">{doc.summary}</p>
                      )}
                      {doc.practical_conclusion && (
                        <p className="mt-2 text-xs text-muted-foreground">
                          <strong>{t('judikatura.practical')}:</strong> {doc.practical_conclusion}
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Summaries */}
      {report.summary_for_lawyer && (
        <div>
          <h2 className="mb-2 font-semibold">{t('judikatura.summaryForLawyer')}</h2>
          <div className="rounded-lg border-l-4 border-primary bg-primary/5 p-4 text-sm leading-relaxed">
            {report.summary_for_lawyer}
          </div>
        </div>
      )}
      {report.summary_for_hr && (
        <div>
          <h2 className="mb-2 font-semibold">{t('judikatura.summaryForHr')}</h2>
          <div className="rounded-lg border-l-4 border-blue-400 bg-blue-50 p-4 text-sm leading-relaxed dark:border-blue-600 dark:bg-blue-950/30">
            {report.summary_for_hr}
          </div>
        </div>
      )}
      {report.summary_en && (
        <div>
          <h2 className="mb-2 font-semibold">{t('judikatura.executiveSummary')}</h2>
          <div className="rounded-lg border-l-4 border-gray-400 bg-gray-50 p-4 text-sm leading-relaxed dark:border-gray-600 dark:bg-gray-950/30">
            {report.summary_en}
          </div>
        </div>
      )}
    </div>
  )
}
