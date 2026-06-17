import { useQueryState } from 'nuqs'
import { rootParser, sectionParser, uuidParser, viewParser } from '@/lib/url-state'
import { DocumentsHome } from '@/components/documents-home'
import { DocumentDetail } from '@/components/document-detail'
import { FoldersHome } from '@/components/folders-home'
import { FolderDetail } from '@/components/folder-detail'

export function App() {
  const [section, setSection] = useQueryState('section', sectionParser)
  const [view, setView] = useQueryState('view', viewParser)
  const [uuid, setUuid] = useQueryState('uuid', uuidParser)
  const [root, setRoot] = useQueryState('root', rootParser)

  const changeSection = (next: 'documents' | 'folders') => {
    void setSection(next)
  }

  if (section === 'folders') {
    if (root) {
      return (
        <FolderDetail
          root={root}
          section={section}
          onChangeSection={changeSection}
          onBack={() => void setRoot(null)}
        />
      )
    }
    return (
      <FoldersHome
        section={section}
        onChangeSection={changeSection}
        onSelectFolder={(r) => void setRoot(r)}
      />
    )
  }

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

  return (
    <DocumentsHome
      section={section}
      onChangeSection={changeSection}
      onSelectDocument={handleSelectDocument}
    />
  )
}
