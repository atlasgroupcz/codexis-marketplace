import { useCallback, useMemo, useState } from 'react'
import { AlertTriangleIcon } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { TEST_ID } from '@workspace/utils/test-ids'
import {
  generateUniqueName,
  getExistingNames,
} from '@workspace/utils/upload-conflict-utils'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@workspace/ui/components/dialog'
import { Button } from '@workspace/ui/components/button'
import { ButtonGroup } from '@workspace/ui/components/button-group'
import { ScrollArea } from '@workspace/ui/components/scroll-area'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@workspace/ui/components/tooltip'
import { getEntryIcon } from '@workspace/ui/lib/file-icons'
import type {
  ConflictResolution,
  FileConflict,
  ResolvedConflict,
} from '@workspace/utils/file-upload'
import type { FileNode } from '@workspace/utils/types/file-system'

interface UploadConflictDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  conflicts: Array<FileConflict>
  existingItems: Array<FileNode>
  onResolve: (resolutions: Array<ResolvedConflict>) => void
  onCancel: () => void
}

export function UploadConflictDialog({
  open,
  onOpenChange,
  conflicts,
  existingItems,
  onResolve,
  onCancel,
}: UploadConflictDialogProps) {
  const { t } = useTranslation()

  // Track resolution for each conflict
  const [resolutions, setResolutions] = useState<
    Map<string, ConflictResolution>
  >(() => new Map())

  // "Apply to all" selection
  const [applyToAll, setApplyToAll] = useState<ConflictResolution | null>(null)

  // Reset state when dialog opens
  const handleOpenChange = useCallback(
    (isOpen: boolean) => {
      if (isOpen) {
        setResolutions(new Map())
        setApplyToAll(null)
      }
      onOpenChange(isOpen)
    },
    [onOpenChange],
  )

  // Get resolution for a specific conflict
  const getResolution = useCallback(
    (name: string): ConflictResolution => {
      if (applyToAll) return applyToAll
      return resolutions.get(name) ?? 'replace'
    },
    [applyToAll, resolutions],
  )

  // Set resolution for a specific conflict
  const setResolution = useCallback(
    (name: string, resolution: ConflictResolution) => {
      // Clear "apply to all" when user changes individual item
      if (applyToAll) {
        setApplyToAll(null)
      }
      setResolutions((prev) => new Map(prev).set(name, resolution))
    },
    [applyToAll],
  )

  // Handle "Apply to all" selection
  const handleApplyToAll = useCallback((resolution: ConflictResolution) => {
    setApplyToAll(resolution)
    setResolutions(new Map())
  }, [])

  // Check if all conflicts have been resolved
  const allResolved = useMemo(() => {
    if (applyToAll) return true
    return conflicts.every((c) => resolutions.has(c.name))
  }, [applyToAll, conflicts, resolutions])

  // Handle continue
  const handleContinue = useCallback(() => {
    const existingNames = getExistingNames(existingItems)

    const result: Array<ResolvedConflict> = conflicts.map((conflict) => {
      const resolution = getResolution(conflict.name)
      const newName =
        resolution === 'rename'
          ? generateUniqueName(conflict.name, existingNames)
          : undefined

      // Add new name to existing names to prevent duplicates
      if (newName) {
        existingNames.add(newName)
      }

      return {
        file: conflict.file,
        originalName: conflict.name,
        resolution,
        newName,
      }
    })

    onResolve(result)
    handleOpenChange(false)
  }, [conflicts, existingItems, getResolution, onResolve, handleOpenChange])

  // Handle cancel
  const handleCancel = useCallback(() => {
    onCancel()
    handleOpenChange(false)
  }, [onCancel, handleOpenChange])

  // Format file size
  const formatSize = useCallback((bytes?: number | null): string => {
    if (bytes == null) return ''
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }, [])

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[500px] grid-cols-[minmax(0,1fr)]" data-testid={TEST_ID.UPLOAD_CONFLICT_DIALOG}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertTriangleIcon className="size-5 text-amber-500" />
            {t('explorer.uploadConflict.title')}
          </DialogTitle>
          <DialogDescription>
            {t('explorer.uploadConflict.description')}
          </DialogDescription>
        </DialogHeader>

        {/* Apply to all option */}
        {conflicts.length > 1 && (
          <div className="flex items-center gap-2 border-b pb-3">
            <span className="text-sm text-muted-foreground">
              {t('explorer.uploadConflict.applyToAll', {
                count: conflicts.length,
              })}
            </span>
            <ButtonGroup className="ml-auto">
              <Button
                size="sm"
                variant={applyToAll === 'skip' ? 'default' : 'outline'}
                onClick={() => handleApplyToAll('skip')}
              >
                {t('explorer.uploadConflict.skip')}
              </Button>
              <Button
                size="sm"
                variant={applyToAll === 'replace' ? 'default' : 'outline'}
                onClick={() => handleApplyToAll('replace')}
              >
                {t('explorer.uploadConflict.replace')}
              </Button>
              <Button
                size="sm"
                variant={applyToAll === 'rename' ? 'default' : 'outline'}
                onClick={() => handleApplyToAll('rename')}
              >
                {t('explorer.uploadConflict.keepBoth')}
              </Button>
            </ButtonGroup>
          </div>
        )}

        {/* Conflict list */}
        <ScrollArea className="max-h-[300px]">
          <div className="flex flex-col gap-3 pr-4">
            {conflicts.map((conflict) => (
              <ConflictItem
                key={conflict.name}
                conflict={conflict}
                resolution={getResolution(conflict.name)}
                onResolutionChange={(r) => setResolution(conflict.name, r)}
                formatSize={formatSize}
              />
            ))}
          </div>
        </ScrollArea>

        <DialogFooter>
          <Button variant="outline" onClick={handleCancel}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleContinue} disabled={!allResolved} data-testid={TEST_ID.UPLOAD_CONFLICT_CONTINUE}>
            {t('explorer.uploadConflict.continue')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

interface ConflictItemProps {
  conflict: FileConflict
  resolution: ConflictResolution
  onResolutionChange: (resolution: ConflictResolution) => void
  formatSize: (bytes?: number | null) => string
}

function ConflictItem({
  conflict,
  resolution,
  onResolutionChange,
  formatSize,
}: ConflictItemProps) {
  const { t } = useTranslation()
  const isFolder = conflict.existingType === 'directory'
  const extension = conflict.name.includes('.')
    ? conflict.name.split('.').pop()
    : undefined
  const Icon = useMemo(
    () => getEntryIcon(isFolder, undefined, extension),
    [isFolder, extension],
  )

  return (
    <div className="flex flex-col gap-2 rounded-md border p-3 overflow-hidden">
      {/* File info */}
      <div className="flex items-start gap-2 min-w-0">
        {/* eslint-disable-next-line react-hooks/static-components -- Icon is a stateless Lucide icon, memoized above */}
        <Icon className="size-4 text-muted-foreground shrink-0 mt-0.5" />
        <div className="flex flex-col gap-0.5 min-w-0">
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="font-medium text-sm truncate">{conflict.name}</span>
            </TooltipTrigger>
            <TooltipContent>{conflict.name}</TooltipContent>
          </Tooltip>
          <span className="text-xs text-muted-foreground">
            {isFolder
              ? t('explorer.uploadConflict.folderExists')
              : t('explorer.uploadConflict.fileExists')}
          </span>
          {!isFolder && (
            <div className="flex gap-3 text-xs text-muted-foreground">
              <span>
                {t('explorer.uploadConflict.existingInfo', {
                  size: formatSize(conflict.existingSize),
                })}
              </span>
              <span>→</span>
              <span>
                {t('explorer.uploadConflict.uploadingInfo', {
                  size: formatSize(conflict.file.size),
                })}
              </span>
            </div>
          )}
        </div>
      </div>

      {/* Resolution buttons */}
      <ButtonGroup className="self-end">
        <Button
          size="sm"
          variant={resolution === 'skip' ? 'default' : 'outline'}
          onClick={() => onResolutionChange('skip')}
          data-testid={TEST_ID.UPLOAD_CONFLICT_SKIP}
        >
          {t('explorer.uploadConflict.skip')}
        </Button>
        <Button
          size="sm"
          variant={resolution === 'replace' ? 'default' : 'outline'}
          onClick={() => onResolutionChange('replace')}
          title={
            isFolder
              ? t('explorer.uploadConflict.mergeFolderHint')
              : undefined
          }
          data-testid={TEST_ID.UPLOAD_CONFLICT_REPLACE}
        >
          {isFolder
            ? t('explorer.uploadConflict.merge')
            : t('explorer.uploadConflict.replace')}
        </Button>
        <Button
          size="sm"
          variant={resolution === 'rename' ? 'default' : 'outline'}
          onClick={() => onResolutionChange('rename')}
          data-testid={TEST_ID.UPLOAD_CONFLICT_KEEP_BOTH}
        >
          {t('explorer.uploadConflict.keepBoth')}
        </Button>
      </ButtonGroup>
    </div>
  )
}
