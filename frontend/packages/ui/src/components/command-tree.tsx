'use client'

import * as React from 'react'
import { CheckIcon, ChevronRightIcon } from 'lucide-react'
import { cn } from '@workspace/ui/lib/utils'
import { SkillAgentIcon } from '@workspace/ui/components/skill-agent-icon'
import {
  Command,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
} from '@workspace/ui/components/command'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@workspace/ui/components/collapsible'
import type { TreeItem, TreeNode } from '@workspace/ui/lib/tree-picker-types'

export interface CommandTreeProps<T extends TreeItem> {
  nodes: Array<TreeNode<T>>
  selectedIds: Set<string>
  onItemToggle: (item: T) => void
  multi?: boolean
  searchPlaceholder?: string
  emptyText?: string
  showSearch?: boolean
  renderItem?: (item: T, isSelected: boolean) => React.ReactNode
  className?: string
}

export function CommandTree<T extends TreeItem>({
  nodes,
  selectedIds,
  onItemToggle,
  multi = false,
  searchPlaceholder = 'Search...',
  emptyText = 'No items found.',
  showSearch = true,
  renderItem,
  className,
}: CommandTreeProps<T>) {
  const [search, setSearch] = React.useState('')
  const [collapsed, setCollapsed] = React.useState<Set<string>>(new Set())
  const hasSearch = search.length > 0

  const toggleCollapsed = React.useCallback((key: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev)
      if (next.has(key)) {
        next.delete(key)
      } else {
        next.add(key)
      }
      return next
    })
  }, [])

  const filteredNodes = React.useMemo(() => {
    if (!hasSearch) return nodes
    const lowerSearch = search.toLowerCase()
    return filterTree(nodes, lowerSearch)
  }, [nodes, search, hasSearch])

  return (
    <Command className={cn('flex h-full w-full flex-col overflow-hidden', className)} shouldFilter={false}>
      {showSearch && (
        <CommandInput
          placeholder={searchPlaceholder}
          value={search}
          onValueChange={setSearch}
        />
      )}
      <CommandList>
        {filteredNodes.length === 0 && <CommandEmpty>{emptyText}</CommandEmpty>}
        {filteredNodes.map((node) => (
          <TreeGroup
            key={node.key}
            node={node}
            depth={0}
            selectedIds={selectedIds}
            onItemToggle={onItemToggle}
            multi={multi}
            renderItem={renderItem}
            collapsed={collapsed}
            toggleCollapsed={toggleCollapsed}
            forceExpand={hasSearch}
            pathPrefix=""
          />
        ))}
      </CommandList>
    </Command>
  )
}

interface TreeGroupProps<T extends TreeItem> {
  node: TreeNode<T>
  depth: number
  selectedIds: Set<string>
  onItemToggle: (item: T) => void
  multi: boolean
  renderItem?: (item: T, isSelected: boolean) => React.ReactNode
  collapsed: Set<string>
  toggleCollapsed: (key: string) => void
  forceExpand: boolean
  pathPrefix: string
}

function TreeGroup<T extends TreeItem>({
  node,
  depth,
  selectedIds,
  onItemToggle,
  multi,
  renderItem,
  collapsed,
  toggleCollapsed,
  forceExpand,
  pathPrefix,
}: TreeGroupProps<T>) {
  const compositeKey = pathPrefix ? `${pathPrefix}/${node.key}` : node.key
  const isOpen = forceExpand || !collapsed.has(compositeKey)

  const hasContent =
    (node.items && node.items.length > 0) ||
    (node.children && node.children.length > 0)

  if (!hasContent) return null

  return (
    <Collapsible open={isOpen} onOpenChange={() => !forceExpand && toggleCollapsed(compositeKey)}>
      <CollapsibleTrigger
        className={cn(
          'flex w-full items-center gap-1.5 rounded-sm px-2 py-1.5 text-xs font-semibold text-muted-foreground hover:bg-accent/50 cursor-pointer select-none',
          depth > 0 && 'text-[11px] font-medium',
        )}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
      >
        <ChevronRightIcon
          className={cn(
            'size-3.5 shrink-0 transition-transform duration-200',
            isOpen && 'rotate-90',
          )}
        />
        <SkillAgentIcon
          iconPath={node.iconPath}
          alt={node.label}
          className="size-3.5 shrink-0 rounded-sm object-contain"
        />
        <span className="truncate">{node.label}</span>
      </CollapsibleTrigger>
      <CollapsibleContent>
        {node.children?.map((child) => (
          <TreeGroup
            key={child.key}
            node={child}
            depth={depth + 1}
            selectedIds={selectedIds}
            onItemToggle={onItemToggle}
            multi={multi}
            renderItem={renderItem}
            collapsed={collapsed}
            toggleCollapsed={toggleCollapsed}
            forceExpand={forceExpand}
            pathPrefix={compositeKey}
          />
        ))}
        {node.items?.map((item) => (
          <TreeLeafItem
            key={item.id}
            item={item}
            depth={depth + 1}
            isSelected={selectedIds.has(item.id)}
            onToggle={onItemToggle}
            multi={multi}
            renderItem={renderItem}
            groupLabel={node.label}
          />
        ))}
      </CollapsibleContent>
    </Collapsible>
  )
}

interface TreeLeafItemProps<T extends TreeItem> {
  item: T
  depth: number
  isSelected: boolean
  onToggle: (item: T) => void
  multi: boolean
  renderItem?: (item: T, isSelected: boolean) => React.ReactNode
  groupLabel: string
}

function TreeLeafItem<T extends TreeItem>({
  item,
  depth,
  isSelected,
  onToggle,
  multi,
  renderItem,
  groupLabel,
}: TreeLeafItemProps<T>) {
  const cmdkValue = `${groupLabel} ${item.label}`

  return (
    <CommandItem
      value={cmdkValue}
      onSelect={() => onToggle(item)}
      style={{ paddingLeft: `${depth * 16 + 8}px` }}
    >
      <SelectionIndicator isSelected={isSelected} multi={multi} />
      <span className="min-w-0 flex-1">
        {renderItem ? renderItem(item, isSelected) : (
          <span className="truncate block">{item.label}</span>
        )}
      </span>
    </CommandItem>
  )
}

function filterTree<T extends TreeItem>(
  nodes: Array<TreeNode<T>>,
  search: string,
): Array<TreeNode<T>> {
  const result: Array<TreeNode<T>> = []

  for (const node of nodes) {
    const matchingItems = node.items?.filter(
      (item) =>
        item.label.toLowerCase().includes(search) ||
        node.label.toLowerCase().includes(search),
    )

    const matchingChildren = node.children
      ? filterTree(node.children, search)
      : undefined

    const hasItems = matchingItems && matchingItems.length > 0
    const hasChildren = matchingChildren && matchingChildren.length > 0

    if (hasItems || hasChildren) {
      result.push({
        ...node,
        items: matchingItems,
        children: matchingChildren,
      })
    }
  }

  return result
}

function SelectionIndicator({
  isSelected,
  multi,
}: {
  isSelected: boolean
  multi: boolean
}) {
  if (isSelected) {
    return (
      <div
        className={cn(
          'flex size-4 shrink-0 items-center justify-center bg-primary border-primary',
          multi ? 'rounded-sm' : 'rounded-full',
        )}
      >
        <CheckIcon className="size-3 text-primary-foreground" />
      </div>
    )
  }

  return (
    <div
      className={cn(
        'flex size-4 shrink-0 items-center justify-center border border-input',
        multi ? 'rounded-sm' : 'rounded-full',
      )}
    />
  )
}
