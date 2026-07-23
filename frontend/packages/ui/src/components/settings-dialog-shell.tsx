import { useEffect } from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@workspace/ui/components/dialog'
import type { ReactNode } from 'react'

export interface SettingsDialogShellProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  description?: string
  children: ReactNode
  /**
   * Optional footer content. Instant-save dialogs (the notification settings) pass
   * nothing and render no footer - they persist on every change and close via the
   * header ✕ / backdrop. Only dialogs with an explicit commit step pass a footer.
   */
  footer?: ReactNode
}

export function SettingsDialogShell({
  open,
  onOpenChange,
  title,
  description,
  children,
  footer,
}: SettingsDialogShellProps) {
  // Close only on a genuine outside click - the pointer must both PRESS and
  // RELEASE outside the dialog. Radix would dismiss on pointer-down-outside alone,
  // which closes the dialog whenever a text selection crosses the backdrop (start
  // on the backdrop and drag in, or start inside and drag out). We suppress Radix's
  // outside-dismiss (onInteractOutside) and decide here on pointer-up: if either
  // end of the drag is inside the dialog, keep it open.
  //
  // Listeners are registered once while the dialog is open (not per pointer-down),
  // so they always catch the current interaction - that timing is what an earlier
  // per-interaction listener got wrong.
  useEffect(() => {
    if (!open) return
    const isOutside = (target: EventTarget | null) =>
      !(target instanceof Element && target.closest('[data-slot="dialog-content"]'))
    let pressedOutside = false
    const onPointerDown = (event: PointerEvent) => {
      pressedOutside = isOutside(event.target)
    }
    const onPointerUp = (event: PointerEvent) => {
      if (pressedOutside && isOutside(event.target)) onOpenChange(false)
    }
    document.addEventListener('pointerdown', onPointerDown, true)
    document.addEventListener('pointerup', onPointerUp, true)
    return () => {
      document.removeEventListener('pointerdown', onPointerDown, true)
      document.removeEventListener('pointerup', onPointerUp, true)
    }
  }, [open, onOpenChange])

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="sm:max-w-md"
        // Don't let Radix dismiss on the pointer-DOWN; the document pointer-up
        // above decides based on where the pointer is released. Escape and the
        // header ✕ still close normally.
        onInteractOutside={(event) => event.preventDefault()}
      >
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          {description && <DialogDescription>{description}</DialogDescription>}
        </DialogHeader>

        {children}

        {footer && <DialogFooter className="gap-2 sm:gap-2">{footer}</DialogFooter>}
      </DialogContent>
    </Dialog>
  )
}
