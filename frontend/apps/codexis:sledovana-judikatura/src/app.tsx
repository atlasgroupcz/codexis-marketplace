import { useQueryState } from 'nuqs'
import { viewParser, uuidParser, reportParser } from '@/lib/url-state'
import { TopicList } from '@/components/topic-list'
import { TopicDetail } from '@/components/topic-detail'
import { ReportView } from '@/components/report-view'

export function App() {
  const [view, setView] = useQueryState('view', viewParser)
  const [uuid, setUuid] = useQueryState('uuid', uuidParser)
  const [reportId, setReportId] = useQueryState('report', reportParser)

  const handleSelectTopic = (topicUuid: string) => {
    void setView('detail')
    void setUuid(topicUuid)
    void setReportId(null)
  }

  const handleSelectReport = (rid: string) => {
    void setView('report')
    void setReportId(rid)
  }

  const handleBackToList = () => {
    void setView('list')
    void setUuid(null)
    void setReportId(null)
  }

  const handleBackToDetail = () => {
    void setView('detail')
    void setReportId(null)
  }

  if (view === 'report' && uuid && reportId) {
    return <ReportView uuid={uuid} reportId={reportId} onBack={handleBackToDetail} />
  }

  if (view === 'detail' && uuid) {
    return <TopicDetail uuid={uuid} onBack={handleBackToList} onSelectReport={handleSelectReport} />
  }

  return <TopicList onSelectTopic={handleSelectTopic} />
}
