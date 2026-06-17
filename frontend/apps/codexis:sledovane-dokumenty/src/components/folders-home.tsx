import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { FileText, FolderClock, FolderPlus, Loader2, Scale, Settings2 } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { SectionSwitch } from './section-switch'
import { FolderPicker } from './folder-picker'
import { NotifySettingsDialog } from './notify-settings'
import { FeedSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import type { FolderSummary, NotifySettings } from '@/lib/schemas'
import { useFolders } from '@/hooks/use-folders'
import { postFolderAction } from '@/lib/api'

interface FoldersHomeProps {
  section: 'documents' | 'folders'
  onChangeSection: (section: 'documents' | 'folders') => void
  onSelectFolder: (root: string) => void
}

export function FoldersHome({ section, onChangeSection, onSelectFolder }: FoldersHomeProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useFolders()
  const [pickerOpen, setPickerOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [adding, setAdding] = useState(false)
  const [settings, setSettings] = useState<NotifySettings | null>(null)

  const folders = data?.folders ?? []
  const effectiveSettings = settings ?? data?.settings ?? { email: true, inApp: true, recipients: [] }

  const handleAdd = (path: string) => {
    setAdding(true)
    postFolderAction('folder_add', { root: path })
      .then(() => {
        toast.success(t('folders.added'))
        setPickerOpen(false)
        refetch()
      })
      .catch((err: unknown) => toast.error(err instanceof Error ? err.message : String(err)))
      .finally(() => setAdding(false))
  }

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="folders-root">
      <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b px-4">
        <div className="flex min-w-0 items-center gap-2.5">
          <span className="bg-secondary text-primary ring-brand-100 flex size-9 items-center justify-center rounded-xl ring-1">
            <FolderClock className="size-5" strokeWidth={1.75} />
          </span>
          <div className="min-w-0">
            <h1 className="font-display text-foreground truncate text-[15px] leading-tight">
              {t('folders.title')}
            </h1>
            <p className="text-muted-foreground hidden truncate text-xs sm:block">
              {t('folders.subtitle')}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <SectionSwitch section={section} onChange={onChangeSection} />
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setSettingsOpen(true)}
            aria-label={t('folders.notify.title')}
          >
            <Settings2 className="size-4" />
          </Button>
          <Button size="sm" onClick={() => setPickerOpen(true)}>
            <FolderPlus className="size-4" />
            <span className="hidden sm:inline">{t('folders.add')}</span>
          </Button>
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
        ) : folders.length === 0 ? (
          <EmptyFolders onAdd={() => setPickerOpen(true)} />
        ) : (
          <ScrollArea className="h-full">
            <div className="mx-auto grid max-w-5xl gap-3 p-4 sm:grid-cols-2 sm:p-6">
              {folders.map((folder) => (
                <FolderCard key={folder.root} folder={folder} onClick={() => onSelectFolder(folder.root)} />
              ))}
            </div>
          </ScrollArea>
        )}
      </main>

      <FolderPicker open={pickerOpen} onOpenChange={setPickerOpen} onSelect={handleAdd} busy={adding} />
      <NotifySettingsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        settings={effectiveSettings}
        onSaved={setSettings}
      />
      <Toaster position="bottom-right" />
    </div>
  )
}

function FolderCard({ folder, onClick }: { folder: FolderSummary; onClick: () => void }) {
  const { t } = useTranslation()
  return (
    <button
      type="button"
      onClick={onClick}
      className="hover:border-primary/40 hover:bg-accent/40 group flex flex-col gap-3 rounded-xl border p-4 text-left transition-colors"
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          <h3 className="font-display truncate text-sm">{folder.name}</h3>
          <p className="text-muted-foreground truncate font-mono text-[11px]" title={folder.root}>
            {folder.root}
          </p>
        </div>
        {folder.unconfirmed_changes > 0 && (
          <span className="bg-primary text-primary-foreground inline-flex min-w-5 shrink-0 items-center justify-center rounded-full px-1.5 text-xs font-semibold">
            {folder.unconfirmed_changes}
          </span>
        )}
      </div>
      <div className="text-muted-foreground flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
        <span className="inline-flex items-center gap-1.5">
          <FileText className="size-3.5" />
          {t('folders.card.documents', { count: folder.documents })}
        </span>
        <span className="inline-flex items-center gap-1.5">
          <Scale className="size-3.5" />
          {t('folders.card.legislation', { count: folder.legislation })}
        </span>
        {folder.pending_harvest > 0 && (
          <span className="text-primary inline-flex items-center gap-1.5">
            <Loader2 className="size-3.5 animate-spin" />
            {t('folders.card.extracting', { count: folder.pending_harvest })}
          </span>
        )}
      </div>
    </button>
  )
}

function EmptyFolders({ onAdd }: { onAdd: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="mx-auto flex h-full max-w-md flex-col items-center justify-center gap-4 px-6 text-center">
      <span className="bg-secondary text-primary flex size-16 items-center justify-center rounded-2xl">
        <FolderClock className="size-8" strokeWidth={1.5} />
      </span>
      <div className="space-y-1.5">
        <h2 className="font-display text-lg">{t('folders.landing.title')}</h2>
        <p className="text-muted-foreground text-sm">{t('folders.landing.text')}</p>
      </div>
      <Button onClick={onAdd}>
        <FolderPlus className="size-4" />
        {t('folders.add')}
      </Button>
    </div>
  )
}
