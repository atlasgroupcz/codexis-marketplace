'use client'

import * as React from 'react'
import { ChevronsUpDownIcon, XIcon } from 'lucide-react'
import { cn } from '@workspace/ui/lib/utils'
import { Button } from '@workspace/ui/components/button'
import { Badge } from '@workspace/ui/components/badge'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@workspace/ui/components/popover'
import { CommandTree  } from '@workspace/ui/components/command-tree'
import type {CommandTreeProps} from '@workspace/ui/components/command-tree';
import type { TreeItem } from '@workspace/ui/lib/tree-picker-types'

export interface TreePickerProps<T extends TreeItem>
  extends Pick<
    CommandTreeProps<T>,
    'nodes' | 'selectedIds' | 'onItemToggle' | 'multi' | 'searchPlaceholder' | 'emptyText' | 'renderItem'
  > {
  placeholder?: string
  selectedItems: Array<{ id: string; label: string }>
  onClear?: () => void
  onItemRemove?: (id: string) => void
  disabled?: boolean
  triggerClassName?: string
  popoverClassName?: string
  'data-testid'?: string
  popoverTestId?: string
}

export function TreePicker<T extends TreeItem>({
  nodes,
  selectedIds,
  onItemToggle,
  multi = false,
  searchPlaceholder,
  emptyText,
  renderItem,
  placeholder = 'Select...',
  selectedItems,
  onClear,
  onItemRemove,
  disabled,
  triggerClassName,
  popoverClassName,
  'data-testid': testId,
  popoverTestId,
}: TreePickerProps<T>) {
  const [open, setOpen] = React.useState(false)

  const handleItemToggle = React.useCallback(
    (item: T) => {
      onItemToggle(item)
      if (!multi) {
        setOpen(false)
      }
    },
    [onItemToggle, multi],
  )

  const handleClear = React.useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation()
      onClear?.()
    },
    [onClear],
  )

  const triggerLabel = multi
    ? selectedItems.length > 0
      ? `${selectedItems.length} selected`
      : placeholder
    : selectedItems[0]?.label ?? placeholder

  const showClearButton = !multi && selectedItems.length > 0 && onClear

  return (
    <div className="space-y-2">
      <Popover open={open} onOpenChange={setOpen} modal>
        <PopoverTrigger asChild>
          <Button
            type="button"
            variant="outline"
            disabled={disabled}
            className={cn('w-full justify-between font-normal px-3', triggerClassName)}
            data-testid={testId}
          >
            <span
              className={cn(
                'truncate',
                selectedItems.length === 0 && 'text-muted-foreground',
              )}
            >
              {triggerLabel}
            </span>
            {showClearButton ? (
              <span
                role="button"
                tabIndex={0}
                onClick={handleClear}
                onPointerDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                }}
                className="rounded-sm hover:bg-muted-foreground/20 transition-colors cursor-pointer p-0.5"
              >
                <XIcon className="size-4 shrink-0 opacity-50 hover:opacity-100" />
              </span>
            ) : (
              <ChevronsUpDownIcon className="size-4 shrink-0 opacity-50" />
            )}
          </Button>
        </PopoverTrigger>
        <PopoverContent
          className={cn('w-(--radix-popover-trigger-width) max-w-[calc(100vw-2rem)] overflow-hidden p-0 !animate-none data-[state=open]:!animate-none data-[state=closed]:!animate-none', popoverClassName)}
          align="start"
          sideOffset={-36}
          data-testid={popoverTestId}
        >
          <CommandTree
            nodes={nodes}
            selectedIds={selectedIds}
            onItemToggle={handleItemToggle}
            multi={multi}
            searchPlaceholder={searchPlaceholder}
            emptyText={emptyText}
            renderItem={renderItem}
          />
        </PopoverContent>
      </Popover>

      {multi && selectedItems.length > 0 && onItemRemove && (
        <div className="flex flex-wrap gap-1">
          {selectedItems.map((item) => (
            <Badge key={item.id} variant="secondary" className="gap-1 pr-1">
              <span className="truncate max-w-[150px]">{item.label}</span>
              <button
                type="button"
                onClick={() => onItemRemove(item.id)}
                className="rounded-sm p-0.5 hover:bg-muted-foreground/20 cursor-pointer"
              >
                <XIcon className="size-3" />
              </button>
            </Badge>
          ))}
        </div>
      )}
    </div>
  )
}
