import { useQueryState } from 'nuqs'
import { reportParser, uuidParser, viewParser } from '@/lib/url-state'
import { JudikaturaHome } from '@/components/judikatura-home'
import { TopicDetail } from '@/components/topic-detail'
import { ReportView } from '@/components/report-view'

export function App() {
  const [view, setView] = useQueryState('view', viewParser)
  const [uuid, setUuid] = useQueryState('uuid', uuidParser)
  const [reportId, setReportId] = useQueryState('report', reportParser)

  const handleSelectReport = (topicUuid: string, rid: string) => {
    void setUuid(topicUuid)
    void setReportId(rid)
    void setView('report')
  }

  const handleOpenTopic = (topicUuid: string) => {
    void setUuid(topicUuid)
    void setReportId(null)
    void setView('detail')
  }

  const handleBackToList = () => {
    void setView('list')
    void setUuid(null)
    void setReportId(null)
  }

  if (view === 'report' && uuid && reportId) {
    return <ReportView uuid={uuid} reportId={reportId} onBack={handleBackToList} />
  }

  if (view === 'detail' && uuid) {
    return (
      <TopicDetail
        uuid={uuid}
        onBack={handleBackToList}
        onSelectReport={(rid) => handleSelectReport(uuid, rid)}
      />
    )
  }

  return <JudikaturaHome onSelectReport={handleSelectReport} onOpenTopic={handleOpenTopic} />
}
