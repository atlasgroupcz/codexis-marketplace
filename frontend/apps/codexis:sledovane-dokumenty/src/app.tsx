import { useQueryState } from 'nuqs'
import { uuidParser, viewParser } from '@/lib/url-state'
import { DocumentList } from '@/components/document-list'
import { DocumentDetail } from '@/components/document-detail'

export function App() {
  const [view, setView] = useQueryState('view', viewParser)
  const [uuid, setUuid] = useQueryState('uuid', uuidParser)

  const handleSelectDocument = (docUuid: string) => {
    void setView('detail')
    void setUuid(docUuid)
  }

  const handleBack = () => {
    void setView('list')
    void setUuid(null)
  }

  if (view === 'detail' && uuid) {
    return <DocumentDetail uuid={uuid} onBack={handleBack} />
  }

  return <DocumentList onSelectDocument={handleSelectDocument} />
}
