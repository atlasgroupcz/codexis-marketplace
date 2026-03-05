/**
 * Custom event system for bridging @dnd-kit drag operations with external drop zones.
 *
 * Since chat input is outside the DndContext providers, we use custom events
 * to communicate file path drops from the file tree/explorer to the chat input.
 */

export const FILE_DROP_EVENT = 'cdx:file-drop'
export const CHAT_INPUT_DROP_ZONE_ATTR = 'data-chat-input-drop-zone'

export interface FileDropEventDetail {
  paths: Array<string>
}

/**
 * Dispatch a custom event with file paths to be handled by the chat input.
 */
export function dispatchFileDrop(paths: Array<string>): void {
  window.dispatchEvent(
    new CustomEvent<FileDropEventDetail>(FILE_DROP_EVENT, {
      detail: { paths },
    }),
  )
}

/**
 * Check if a point is over an element with the chat input drop zone attribute.
 * Uses bounding rect comparison instead of elementFromPoint to avoid
 * being blocked by drag overlays.
 */
export function isOverChatInputDropZone(x: number, y: number): boolean {
  // Find all chat input drop zones
  const dropZones = document.querySelectorAll(`[${CHAT_INPUT_DROP_ZONE_ATTR}]`)

  for (const zone of dropZones) {
    const rect = zone.getBoundingClientRect()
    if (
      x >= rect.left &&
      x <= rect.right &&
      y >= rect.top &&
      y <= rect.bottom
    ) {
      return true
    }
  }

  return false
}
