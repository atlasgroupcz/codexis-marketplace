import { useTranslation } from 'react-i18next'
import { BookMarked, FolderClock } from 'lucide-react'

type Section = 'documents' | 'folders'

interface SectionSwitchProps {
  section: Section
  onChange: (section: Section) => void
}

export function SectionSwitch({ section, onChange }: SectionSwitchProps) {
  const { t } = useTranslation()
  const items = [
    { key: 'documents' as const, label: t('folders.section.documents'), icon: BookMarked },
    { key: 'folders' as const, label: t('folders.section.folders'), icon: FolderClock },
  ]
  return (
    <div className="bg-secondary inline-flex items-center gap-0.5 rounded-lg p-0.5" role="tablist">
      {items.map(({ key, label, icon: Icon }) => (
        <button
          key={key}
          type="button"
          role="tab"
          aria-selected={section === key}
          onClick={() => onChange(key)}
          className={`inline-flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs font-medium transition-colors ${
            section === key
              ? 'bg-background text-foreground shadow-sm'
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          <Icon className="size-3.5" strokeWidth={1.75} />
          <span className="hidden sm:inline">{label}</span>
        </button>
      ))}
    </div>
  )
}
