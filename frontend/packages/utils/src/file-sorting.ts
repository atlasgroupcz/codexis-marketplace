/**
 * Shared file/directory sorting utilities.
 */

export type SortField = 'name' | 'size' | 'modified' | 'created'
export type SortDirection = 'asc' | 'desc'

export interface SortConfig {
  field: SortField
  direction: SortDirection
}

export interface SortableItem {
  name: string
  type: 'file' | 'directory' | 'symlink' | 'other'
  size?: number | null
  modifiedTime?: string | null
  createdTime?: string | null
}

/**
 * Compare two items by a specific field.
 * Returns negative if a < b, positive if a > b, 0 if equal.
 */
function compareByField<T extends SortableItem>(
  a: T,
  b: T,
  field: SortField,
): number {
  switch (field) {
    case 'name':
      return a.name.localeCompare(b.name)

    case 'size':
      return (a.size ?? 0) - (b.size ?? 0)

    case 'modified': {
      const aTime = a.modifiedTime ? new Date(a.modifiedTime).getTime() : 0
      const bTime = b.modifiedTime ? new Date(b.modifiedTime).getTime() : 0
      return aTime - bTime
    }

    case 'created': {
      const aTime = a.createdTime ? new Date(a.createdTime).getTime() : 0
      const bTime = b.createdTime ? new Date(b.createdTime).getTime() : 0
      return aTime - bTime
    }

    default:
      return 0
  }
}

/**
 * Sort file/directory items with directories first, then by the specified field.
 * Returns a new sorted array (does not mutate the original).
 *
 * @param items - Array of items to sort
 * @param field - Field to sort by
 * @param direction - Sort direction ("asc" or "desc")
 * @returns New sorted array
 */
export function sortFileItems<T extends SortableItem>(
  items: ReadonlyArray<T>,
  field: SortField,
  direction: SortDirection,
): Array<T> {
  return [...items].sort((a, b) => {
    // Directories always come first
    if (a.type !== b.type) {
      return a.type === 'directory' ? -1 : 1
    }

    const comparison = compareByField(a, b, field)
    return direction === 'asc' ? comparison : -comparison
  })
}
