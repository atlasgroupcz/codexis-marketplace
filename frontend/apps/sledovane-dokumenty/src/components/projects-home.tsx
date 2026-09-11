import { AppHeader, HeaderGear } from '@workspace/ui/components/app-header'
import { Button } from '@workspace/ui/components/button'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { BookMarked, FileText, FolderKanban, FolderPlus, Loader2, Scale } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useUnconfirmedCount } from '@/hooks/use-unconfirmed-count'
import { useWatches } from '@/hooks/use-watches'
import { postProjectAction } from '@/lib/api'
import { sumUnconfirmed } from '@/lib/counts'
import type { WatchSummary } from '@/lib/schemas'
import { ErrorMessage } from './error-message'
import { FeedSkeleton } from './loading-skeleton'
import { folderLabel, type PickedFolder, ProjectPicker } from './project-picker'
import { SectionSwitch } from './section-switch'

interface ProjectsHomeProps {
  section: 'documents' | 'projects'
  onChangeSection: (section: 'documents' | 'projects') => void
  onSelectWatch: (watchId: string) => void
  onOpenSettings: () => void
}

export function ProjectsHome({
  section,
  onChangeSection,
  onSelectWatch,
  onOpenSettings,
}: ProjectsHomeProps) {
  const { t, i18n } = useTranslation()
  const { data, loading, error, refetch } = useWatches()
  const [pickerOpen, setPickerOpen] = useState(false)
  const [adding, setAdding] = useState(false)
  const actChanges = useUnconfirmedCount('documents')

  const watches = data?.watches ?? []
  const unconfirmed = sumUnconfirmed(watches)

  const handleAdd = (picked: PickedFolder) => {
    setAdding(true)
    postProjectAction('project_add', { project: picked.projectId, path: picked.path })
      .then(() => {
        toast.success(t('projects.added'))
        setPickerOpen(false)
        refetch()
      })
      .catch((err: unknown) => toast.error(err instanceof Error ? err.message : String(err)))
      .finally(() => setAdding(false))
  }

  return (
    <div
      className="bg-background text-foreground flex h-screen flex-col"
      data-testid="projects-root"
    >
      <AppHeader
        brand={{
          icon: BookMarked,
          title: t('followedDocs.title'),
          subtitle: t('followedDocs.tagline'),
        }}
        docs={{ app: 'sledovane-dokumenty', lang: i18n.language, label: t('help.docs') }}
        actions={
          <>
            <Button size="sm" onClick={() => setPickerOpen(true)}>
              <FolderPlus className="size-4" />
              <span className="hidden sm:inline">{t('projects.add')}</span>
            </Button>
            <HeaderGear onClick={onOpenSettings} title={t('notify.title')} />
          </>
        }
      />

      <div className="flex min-h-0 flex-1">
        <SectionSwitch
          section={section}
          onChange={onChangeSection}
          counts={{ documents: actChanges, projects: unconfirmed }}
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
          ) : watches.length === 0 ? (
            <EmptyProjects onAdd={() => setPickerOpen(true)} />
          ) : (
            <ScrollArea className="h-full">
              <div className="mx-auto grid max-w-5xl gap-3 p-4 sm:grid-cols-2 sm:p-6">
                {watches.map((watch) => (
                  <WatchCard
                    key={watch.uuid}
                    watch={watch}
                    onClick={() => onSelectWatch(watch.uuid)}
                  />
                ))}
              </div>
            </ScrollArea>
          )}
        </main>
      </div>

      <ProjectPicker
        open={pickerOpen}
        onOpenChange={setPickerOpen}
        onSelect={handleAdd}
        busy={adding}
      />
      <Toaster position="bottom-right" />
    </div>
  )
}

function WatchCard({ watch, onClick }: { watch: WatchSummary; onClick: () => void }) {
  const { t } = useTranslation()
  return (
    <button
      type="button"
      onClick={onClick}
      className="hover:border-primary/40 hover:bg-accent/40 group flex flex-col gap-3 rounded-xl border p-4 text-left transition-colors"
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          <h3 className="font-display truncate text-sm">{watch.name}</h3>
          <p
            className="text-muted-foreground truncate font-mono text-[11px]"
            title={folderLabel(watch.path)}
          >
            {folderLabel(watch.path)}
          </p>
        </div>
        {watch.unconfirmed_changes > 0 && (
          <span className="bg-primary text-primary-foreground inline-flex min-w-5 shrink-0 items-center justify-center rounded-full px-1.5 text-xs font-semibold">
            {watch.unconfirmed_changes}
          </span>
        )}
      </div>
      <div className="text-muted-foreground flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
        <span className="inline-flex items-center gap-1.5">
          <FileText className="size-3.5" />
          {t('projects.card.documents', { count: watch.documents })}
        </span>
        <span className="inline-flex items-center gap-1.5">
          <Scale className="size-3.5" />
          {t('projects.card.legislation', { count: watch.legislation })}
        </span>
        {watch.pending_harvest > 0 && (
          <span className="text-primary inline-flex items-center gap-1.5">
            <Loader2 className="size-3.5 animate-spin" />
            {t('projects.card.extracting', { count: watch.pending_harvest })}
          </span>
        )}
      </div>
    </button>
  )
}

function EmptyProjects({ onAdd }: { onAdd: () => void }) {
  const { t } = useTranslation()
  return (
    <div className="mx-auto flex h-full max-w-md flex-col items-center justify-center gap-4 px-6 text-center">
      <span className="bg-secondary text-primary flex size-16 items-center justify-center rounded-2xl">
        <FolderKanban className="size-8" strokeWidth={1.5} />
      </span>
      <div className="space-y-1.5">
        <h2 className="font-display text-lg">{t('projects.landing.title')}</h2>
        <p className="text-muted-foreground text-sm">{t('projects.landing.text')}</p>
      </div>
      <Button onClick={onAdd}>
        <FolderPlus className="size-4" />
        {t('projects.add')}
      </Button>
    </div>
  )
}
