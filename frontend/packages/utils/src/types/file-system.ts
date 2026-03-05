/**
 * Shared file system types — single source of truth.
 */

/**
 * Base file/directory entry type (matches GraphQL FileEntry).
 */
export interface FileEntry {
  id: string
  name: string
  path: string
  type: 'file' | 'directory' | 'symlink' | 'other'
  size?: number | null
  sizeFormatted?: string | null
  extension?: string | null
  mimeType?: string | null
  modifiedTime?: string | null
  createdTime?: string | null
}

/**
 * Rich file node used throughout the file explorer UI.
 * Extends FileEntry (GraphQL source of truth) with tree-specific fields.
 */
export interface FileNode extends FileEntry {
  children?: Array<FileNode>
  isLoading?: boolean
  localChatCount?: number
  validWorkingDir?: boolean
  isDirectory?: boolean
}

/**
 * Flattened tree item for virtualized tree rendering.
 */
export interface FlatTreeItem {
  id: string
  name: string
  path: string
  type: 'file' | 'directory' | 'symlink' | 'other'
  depth: number
  isExpanded?: boolean
  isLoaded?: boolean
  isLoading?: boolean
  fileNode: FileNode
  parentPath: string | null
}

/**
 * Item for delete operations.
 */
export interface DeleteItem {
  path: string
  isDirectory: boolean
}

/**
 * Result of a batch delete operation.
 */
export interface BatchDeleteResult {
  success: number
  failed: number
}
