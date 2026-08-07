import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { BookMarked, FileText, FolderClock, FolderPlus, Loader2, Scale } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { AppHeader, HeaderGear } from '@workspace/ui/components/app-header'
import { SectionSwitch } from './section-switch'
import { FolderPicker } from './folder-picker'
import { FeedSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import type { FolderSummary } from '@/lib/schemas'
import { useFolders } from '@/hooks/use-folders'
import { useUnconfirmedCount } from '@/hooks/use-unconfirmed-count'
import { sumUnconfirmed } from '@/lib/counts'
import { postFolderAction } from '@/lib/api'

interface FoldersHomeProps {
  section: 'documents' | 'folders'
  onChangeSection: (section: 'documents' | 'folders') => void
  onSelectFolder: (root: string) => void
  onOpenSettings: () => void
}

export function FoldersHome({
  section,
  onChangeSection,
  onSelectFolder,
  onOpenSettings,
}: FoldersHomeProps) {
  const { t, i18n } = useTranslation()
  const { data, loading, error, refetch } = useFolders()
  const [pickerOpen, setPickerOpen] = useState(false)
  const [adding, setAdding] = useState(false)
  const actChanges = useUnconfirmedCount('documents')

  const folders = data?.folders ?? []
  const unconfirmed = sumUnconfirmed(folders)

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
      <AppHeader
        brand={{ icon: BookMarked, title: t('followedDocs.title'), subtitle: t('followedDocs.tagline') }}
        docs={{ app: 'sledovane-dokumenty', lang: i18n.language, label: t('help.docs') }}
        actions={
          <>
            <Button size="sm" onClick={() => setPickerOpen(true)}>
              <FolderPlus className="size-4" />
              <span className="hidden sm:inline">{t('folders.add')}</span>
            </Button>
            <HeaderGear onClick={onOpenSettings} title={t('notify.title')} />
          </>
        }
      />

      <div className="flex min-h-0 flex-1">
        <SectionSwitch
          section={section}
          onChange={onChangeSection}
          counts={{ documents: actChanges, folders: unconfirmed }}
        />

        <main className="min-h-0 min-w-0 flex-1">
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
                  <FolderCard
                    key={folder.root}
                    folder={folder}
                    onClick={() => onSelectFolder(folder.root)}
                  />
                ))}
              </div>
            </ScrollArea>
          )}
        </main>
      </div>

      <FolderPicker open={pickerOpen} onOpenChange={setPickerOpen} onSelect={handleAdd} busy={adding} />
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
