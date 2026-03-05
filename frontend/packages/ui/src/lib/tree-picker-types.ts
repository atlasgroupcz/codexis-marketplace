/** Minimum contract for a selectable leaf item */
export interface TreeItem {
  id: string
  label: string
}

/**
 * A recursive tree node: either a group with children sub-groups,
 * or contains leaf items at this level, or both.
 */
export interface TreeNode<T extends TreeItem> {
  key: string
  label: string
  iconPath?: string | null
  children?: Array<TreeNode<T>>
  items?: Array<T>
}
