import { describe, expect, it } from 'vitest'
import { buildTree } from './tree-utils'
import type { TreeItem } from './tree-picker-types'

interface TestItem extends TreeItem {
  group: string
  subGroup: string
}

function makeItem(id: string, label: string, group: string, subGroup: string): TestItem {
  return { id, label, group, subGroup }
}

describe('buildTree', () => {
  it('returns empty array for empty input', () => {
    const result = buildTree<TestItem>([], () => [])
    expect(result).toEqual([])
  })

  it('groups items by single level', () => {
    const items: Array<TestItem> = [
      makeItem('1', 'Alpha', 'G1', 'S1'),
      makeItem('2', 'Beta', 'G1', 'S1'),
      makeItem('3', 'Gamma', 'G2', 'S1'),
    ]

    const result = buildTree(items, (item) => [
      { key: item.group, label: item.group },
    ])

    expect(result).toHaveLength(2)
    expect(result[0]!.key).toBe('G1')
    expect(result[0]!.items).toHaveLength(2)
    expect(result[1]!.key).toBe('G2')
    expect(result[1]!.items).toHaveLength(1)
  })

  it('groups items by two levels', () => {
    const items: Array<TestItem> = [
      makeItem('1', 'Alpha', 'G1', 'S1'),
      makeItem('2', 'Beta', 'G1', 'S2'),
      makeItem('3', 'Gamma', 'G2', 'S1'),
    ]

    const result = buildTree(items, (item) => [
      { key: item.group, label: item.group },
      { key: item.subGroup, label: item.subGroup },
    ])

    expect(result).toHaveLength(2)

    const g1 = result[0]!
    expect(g1.key).toBe('G1')
    expect(g1.children).toHaveLength(2)
    expect(g1.items).toBeUndefined()

    const g1s1 = g1.children![0]!
    expect(g1s1.key).toBe('S1')
    expect(g1s1.items).toHaveLength(1)
    expect(g1s1.items![0]!.id).toBe('1')

    const g1s2 = g1.children![1]!
    expect(g1s2.key).toBe('S2')
    expect(g1s2.items).toHaveLength(1)
    expect(g1s2.items![0]!.id).toBe('2')
  })

  it('merges items into same group node', () => {
    const items: Array<TestItem> = [
      makeItem('1', 'Alpha', 'G1', 'S1'),
      makeItem('2', 'Beta', 'G1', 'S1'),
    ]

    const result = buildTree(items, (item) => [
      { key: item.group, label: item.group },
      { key: item.subGroup, label: item.subGroup },
    ])

    expect(result).toHaveLength(1)
    expect(result[0]!.children).toHaveLength(1)
    expect(result[0]!.children![0]!.items).toHaveLength(2)
  })

  it('sorts groups alphabetically by label', () => {
    const items: Array<TestItem> = [
      makeItem('1', 'Item', 'Zebra', 'Z'),
      makeItem('2', 'Item', 'Apple', 'A'),
      makeItem('3', 'Item', 'Mango', 'M'),
    ]

    const result = buildTree(items, (item) => [
      { key: item.group, label: item.group },
    ])

    expect(result.map((n) => n.label)).toEqual(['Apple', 'Mango', 'Zebra'])
  })

  it('sorts nested groups alphabetically', () => {
    const items: Array<TestItem> = [
      makeItem('1', 'Item', 'G1', 'Zebra'),
      makeItem('2', 'Item', 'G1', 'Apple'),
    ]

    const result = buildTree(items, (item) => [
      { key: item.group, label: item.group },
      { key: item.subGroup, label: item.subGroup },
    ])

    const children = result[0]!.children!
    expect(children.map((n) => n.label)).toEqual(['Apple', 'Zebra'])
  })

  it('preserves item order within groups', () => {
    const items: Array<TestItem> = [
      makeItem('3', 'Gamma', 'G1', 'S1'),
      makeItem('1', 'Alpha', 'G1', 'S1'),
      makeItem('2', 'Beta', 'G1', 'S1'),
    ]

    const result = buildTree(items, (item) => [
      { key: item.group, label: item.group },
      { key: item.subGroup, label: item.subGroup },
    ])

    const leafItems = result[0]!.children![0]!.items!
    expect(leafItems.map((i) => i.id)).toEqual(['3', '1', '2'])
  })

  it('handles items with same key but different levels', () => {
    const items: Array<TestItem> = [
      makeItem('1', 'Alpha', 'Shared', 'Sub1'),
      makeItem('2', 'Beta', 'Other', 'Shared'),
    ]

    const result = buildTree(items, (item) => [
      { key: item.group, label: item.group },
      { key: item.subGroup, label: item.subGroup },
    ])

    expect(result).toHaveLength(2)
    // 'Shared' at level 0 and 'Shared' at level 1 under 'Other' are distinct nodes
    const otherNode = result.find((n) => n.key === 'Other')!
    expect(otherNode.children![0]!.key).toBe('Shared')
  })
})
