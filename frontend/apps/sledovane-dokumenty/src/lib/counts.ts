/** Unconfirmed changes across a watchdog's list - the number every badge shows. */
export function sumUnconfirmed(items: ReadonlyArray<{ unconfirmed_changes: number }>): number {
  return items.reduce((sum, item) => sum + item.unconfirmed_changes, 0)
}
