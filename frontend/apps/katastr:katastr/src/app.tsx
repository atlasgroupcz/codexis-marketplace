import { useQueryState } from 'nuqs'
import { Toaster } from '@workspace/ui/components/sonner'
import { uuidParser, viewParser } from '@/lib/url-state'
import { ProceedingsList } from '@/components/proceedings-list'
import { ProceedingDetail } from '@/components/proceeding-detail'

export function App() {
  const [view, setView] = useQueryState('view', viewParser)
  const [uuid, setUuid] = useQueryState('uuid', uuidParser)

  const handleSelect = (proceedingUuid: string) => {
    void setView('detail')
    void setUuid(proceedingUuid)
  }

  const handleBack = () => {
    void setView('list')
    void setUuid(null)
  }

  return (
    <>
      {view === 'detail' && uuid ? (
        <ProceedingDetail uuid={uuid} onBack={handleBack} />
      ) : (
        <ProceedingsList onSelect={handleSelect} />
      )}
      <Toaster position="bottom-right" />
    </>
  )
}
