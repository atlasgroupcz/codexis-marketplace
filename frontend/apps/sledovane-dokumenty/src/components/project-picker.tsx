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
import { Check, FolderKanban, Loader2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { fetchAvailableProjects } from '@/lib/api'
import type { AvailableProject } from '@/lib/schemas'
import { ErrorMessage } from './error-message'

interface ProjectPickerProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSelect: (projectId: string) => void
  busy?: boolean
}

/**
 * Picks one of the user's projects that is not watched yet.
 *
 * The list comes from the daemon through the CGI handler, so it is the same
 * set of projects the user sees in the sidebar - a watched project is a
 * project, never a path the user has to know.
 */
export function ProjectPicker({ open, onOpenChange, onSelect, busy }: ProjectPickerProps) {
  const { t } = useTranslation()
  const [projects, setProjects] = useState<Array<AvailableProject> | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const [selected, setSelected] = useState<string | null>(null)
  const [reloadKey, setReloadKey] = useState(0)

  useEffect(() => {
    if (!open) {
      return
    }
    let cancelled = false
    setSelected(null)
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
  }, [open, reloadKey])

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t('projects.picker.title')}</DialogTitle>
          <DialogDescription>{t('projects.picker.description')}</DialogDescription>
        </DialogHeader>

        <ScrollArea className="h-64 rounded-md border">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="text-muted-foreground size-5 animate-spin" />
            </div>
          ) : error ? (
            <div className="p-4">
              <ErrorMessage message={error.message} onRetry={() => setReloadKey((k) => k + 1)} />
            </div>
          ) : projects && projects.length > 0 ? (
            <ul className="divide-y" aria-label={t('projects.picker.title')}>
              {projects.map((project) => {
                const active = project.projectId === selected
                return (
                  <li key={project.projectId}>
                    <button
                      type="button"
                      aria-pressed={active}
                      onClick={() => setSelected(project.projectId)}
                      className={`flex w-full items-center justify-between gap-2 px-3 py-2 text-left text-sm ${
                        active ? 'bg-secondary text-secondary-foreground' : 'hover:bg-accent'
                      }`}
                    >
                      <span className="flex min-w-0 items-center gap-2">
                        <FolderKanban className="text-muted-foreground size-4 shrink-0" />
                        <span className="truncate">{project.name}</span>
                      </span>
                      {active && <Check className="text-primary size-4 shrink-0" />}
                    </button>
                  </li>
                )
              })}
            </ul>
          ) : (
            <div className="text-muted-foreground flex items-center justify-center px-6 py-12 text-center text-sm">
              {t('projects.picker.none')}
            </div>
          )}
        </ScrollArea>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>
            {t('projects.cancel')}
          </Button>
          <Button onClick={() => selected && onSelect(selected)} disabled={!selected || busy}>
            {busy && <Loader2 className="size-4 animate-spin" />}
            {t('projects.picker.watchThis')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
