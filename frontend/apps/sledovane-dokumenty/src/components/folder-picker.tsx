import { Button } from '@workspace/ui/components/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@workspace/ui/components/dialog'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import { ArrowUp, ChevronRight, Folder, Home, Loader2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { browseFolders } from '@/lib/api'
import type { BrowseResponse } from '@/lib/schemas'

interface FolderPickerProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSelect: (path: string) => void
  busy?: boolean
}

export function FolderPicker({ open, onOpenChange, onSelect, busy }: FolderPickerProps) {
  const { t } = useTranslation()
  const [data, setData] = useState<BrowseResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [path, setPath] = useState<string | undefined>(undefined)

  useEffect(() => {
    if (open) {
      setPath(undefined)
    }
  }, [open])

  useEffect(() => {
    if (!open) {
      return
    }
    let cancelled = false
    setLoading(true)
    browseFolders(path)
      .then((res) => {
        if (!cancelled) {
          setData(res)
        }
      })
      .catch(() => {
        if (!cancelled) {
          setData(null)
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
  }, [open, path])

  const current = data?.path

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t('folders.picker.title')}</DialogTitle>
        </DialogHeader>

        <div className="text-muted-foreground flex items-center gap-1 text-xs">
          <Button
            variant="ghost"
            size="icon"
            className="size-7 shrink-0"
            onClick={() => setPath(data?.home)}
            disabled={loading}
            aria-label={t('folders.picker.home')}
          >
            <Home className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="size-7 shrink-0"
            onClick={() => setPath(data?.parent ?? undefined)}
            disabled={loading || !data?.parent}
            aria-label={t('folders.picker.up')}
          >
            <ArrowUp className="size-4" />
          </Button>
          <span className="truncate font-mono" title={current}>
            {current ?? '…'}
          </span>
        </div>

        <ScrollArea className="h-64 rounded-md border">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="text-muted-foreground size-5 animate-spin" />
            </div>
          ) : data && data.entries.length > 0 ? (
            <ul className="divide-y">
              {data.entries.map((entry) => (
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
            <div className="text-muted-foreground flex items-center justify-center py-12 text-sm">
              {t('folders.picker.noSubfolders')}
            </div>
          )}
        </ScrollArea>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>
            {t('folders.cancel')}
          </Button>
          <Button onClick={() => current && onSelect(current)} disabled={!current || busy}>
            {busy && <Loader2 className="size-4 animate-spin" />}
            {t('folders.picker.watchThis')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
