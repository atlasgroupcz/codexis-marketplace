import { Button } from '@workspace/ui/components/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@workspace/ui/components/dialog'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { ArrowLeft, ArrowUp, ChevronRight, Folder, FolderKanban, Loader2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { browseProject, fetchAvailableProjects } from '@/lib/api'
import type { AvailableProject, BrowseResponse } from '@/lib/schemas'
import { ErrorMessage } from './error-message'

export interface PickedFolder {
  projectId: string
  path: string
}

interface ProjectPickerProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSelect: (picked: PickedFolder) => void
  busy?: boolean
}

export const ROOT_FOLDER = '.'

/** How a folder inside the project reads: `/` for the root, `/a/b` below it. */
export function folderLabel(path: string): string {
  return path === ROOT_FOLDER ? '/' : `/${path}`
}

/**
 * Picks a folder to watch in two steps: one of the user's projects, then a
 * folder inside that project's files (its root included).
 *
 * Projects come from the daemon through the CGI handler, so it is the same set
 * the user sees in the sidebar; folders are the project's files as the daemon
 * keeps them, never a path the user has to know.
 */
export function ProjectPicker({ open, onOpenChange, onSelect, busy }: ProjectPickerProps) {
  const { t } = useTranslation()
  const [projects, setProjects] = useState<Array<AvailableProject> | null>(null)
  const [project, setProject] = useState<AvailableProject | null>(null)
  const [folder, setFolder] = useState<BrowseResponse | null>(null)
  const [path, setPath] = useState<string | undefined>(undefined)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const [reloadKey, setReloadKey] = useState(0)

  useEffect(() => {
    if (open) {
      setProject(null)
      setFolder(null)
      setPath(undefined)
    }
  }, [open])

  useEffect(() => {
    if (!open || project) {
      return
    }
    let cancelled = false
    setLoading(true)
    setError(null)
    fetchAvailableProjects()
      .then((res) => {
        if (!cancelled) {
          setProjects(res.projects)
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setProjects(null)
          setError(err instanceof Error ? err : new Error(String(err)))
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false)
        }
      })
    return () => {
      cancelled = true
    }
  }, [open, project, reloadKey])

  useEffect(() => {
    if (!open || !project) {
      return
    }
    let cancelled = false
    setLoading(true)
    setError(null)
    browseProject(project.projectId, path)
      .then((res) => {
        if (!cancelled) {
          setFolder(res)
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setFolder(null)
          setError(err instanceof Error ? err : new Error(String(err)))
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false)
        }
      })
    return () => {
      cancelled = true
    }
  }, [open, project, path, reloadKey])

  const retry = () => setReloadKey((k) => k + 1)
  const current = folder?.path

  const list = loading ? (
    <div className="flex items-center justify-center py-12">
      <Loader2 className="text-muted-foreground size-5 animate-spin" />
    </div>
  ) : error ? (
    <div className="p-4">
      <ErrorMessage message={error.message} onRetry={retry} />
    </div>
  ) : !project ? (
    projects && projects.length > 0 ? (
      <ul className="divide-y" aria-label={t('projects.picker.projectsList')}>
        {projects.map((item) => (
          <li key={item.projectId}>
            <button
              type="button"
              onClick={() => setProject(item)}
              className="hover:bg-accent flex w-full items-center justify-between gap-2 px-3 py-2 text-left text-sm"
            >
              <span className="flex min-w-0 items-center gap-2">
                <FolderKanban className="text-muted-foreground size-4 shrink-0" />
                <span className="truncate">{item.name}</span>
              </span>
              <ChevronRight className="text-muted-foreground size-4 shrink-0" />
            </button>
          </li>
        ))}
      </ul>
    ) : (
      <div className="text-muted-foreground flex items-center justify-center px-6 py-12 text-center text-sm">
        {t('projects.picker.noProjects')}
      </div>
    )
  ) : folder && folder.entries.length > 0 ? (
    <ul className="divide-y" aria-label={t('projects.picker.foldersList')}>
      {folder.entries.map((entry) => (
        <li key={entry.path}>
          <button
            type="button"
            onClick={() => setPath(entry.path)}
            className="hover:bg-accent flex w-full items-center justify-between gap-2 px-3 py-2 text-left text-sm"
          >
            <span className="flex min-w-0 items-center gap-2">
              <Folder className="text-muted-foreground size-4 shrink-0" />
              <span className="truncate">{entry.name}</span>
            </span>
            <ChevronRight className="text-muted-foreground size-4 shrink-0" />
          </button>
        </li>
      ))}
    </ul>
  ) : (
    <div className="text-muted-foreground flex items-center justify-center px-6 py-12 text-center text-sm">
      {t('projects.picker.noSubfolders')}
    </div>
  )

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>
            {project ? t('projects.picker.folderTitle') : t('projects.picker.title')}
          </DialogTitle>
          <DialogDescription>
            {project ? t('projects.picker.folderDescription') : t('projects.picker.description')}
          </DialogDescription>
        </DialogHeader>

        {project && (
          <div className="text-muted-foreground flex items-center gap-1 text-xs">
            <Button
              variant="ghost"
              size="icon"
              className="size-7 shrink-0"
              onClick={() => {
                setProject(null)
                setFolder(null)
                setPath(undefined)
              }}
              disabled={loading}
              aria-label={t('projects.picker.backToProjects')}
            >
              <ArrowLeft className="size-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="size-7 shrink-0"
              onClick={() => setPath(folder?.parent ?? undefined)}
              disabled={loading || !folder?.parent}
              aria-label={t('projects.picker.up')}
            >
              <ArrowUp className="size-4" />
            </Button>
            <span className="truncate font-mono" title={current ? folderLabel(current) : undefined}>
              <span className="text-foreground">{project.name}</span>
              {current ? folderLabel(current) : '…'}
            </span>
          </div>
        )}

        <ScrollArea className="h-64 rounded-md border">{list}</ScrollArea>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>
            {t('projects.cancel')}
          </Button>
          <Button
            onClick={() =>
              project && current && onSelect({ projectId: project.projectId, path: current })
            }
            disabled={!project || !current || busy}
          >
            {busy && <Loader2 className="size-4 animate-spin" />}
            {t('projects.picker.watchThis')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
