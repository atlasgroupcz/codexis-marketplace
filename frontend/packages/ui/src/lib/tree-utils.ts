import type { TreeItem, TreeNode } from './tree-picker-types'

interface GroupSegment {
  key: string
  label: string
  iconPath?: string | null
}

/**
 * Transforms a flat list of items into a tree grouped by the path returned
 * from `getGroupPath`. Items sharing the same path keys merge into the same nodes.
 *
 * Returns groups sorted alphabetically by label; item order within groups is preserved.
 */
export function buildTree<T extends TreeItem>(
  items: Array<T>,
  getGroupPath: (item: T) => Array<GroupSegment>,
): Array<TreeNode<T>> {
  const root: Array<TreeNode<T>> = []
  const nodeMap = new Map<string, TreeNode<T>>()

  for (const item of items) {
    const path = getGroupPath(item)
    let currentChildren = root
    let compositeKey = ''

    for (let i = 0; i < path.length; i++) {
      const segment = path[i]!
      compositeKey = compositeKey ? `${compositeKey}/${segment.key}` : segment.key

      let node = nodeMap.get(compositeKey)
      if (!node) {
        node = { key: segment.key, label: segment.label, iconPath: segment.iconPath }
        nodeMap.set(compositeKey, node)
        currentChildren.push(node)
      }

      if (i < path.length - 1) {
        if (!node.children) {
          node.children = []
        }
        currentChildren = node.children
      } else {
        if (!node.items) {
          node.items = []
        }
        node.items.push(item)
      }
    }
  }

  sortNodes(root)
  return root
}

function sortNodes<T extends TreeItem>(nodes: Array<TreeNode<T>>): void {
  nodes.sort((a, b) => a.label.localeCompare(b.label))
  for (const node of nodes) {
    if (node.children) {
      sortNodes(node.children)
    }
  }
}
