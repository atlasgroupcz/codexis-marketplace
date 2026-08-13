import { toast } from '@workspace/ui/components/sonner'
import { useState } from 'react'
import { confirmChange } from '@/lib/api'
import { errorMessage } from '@/lib/format'

/**
 * Confirming a change is triggered from two places - the whole-tab action next
 * to the tabs and the control on a single row - so the pending flag lives above
 * both. Without it, marking everything read would leave the row buttons live
 * against a list the server is already rewriting.
 */
export function useConfirmChange(cisloRizeni: string, onConfirmed: () => void) {
  const [pending, setPending] = useState(false)

  const confirm = (index?: number) => {
    setPending(true)
    confirmChange(cisloRizeni, index)
      .then(() => onConfirmed())
      .catch((e: unknown) => toast.error(errorMessage(e)))
      .finally(() => setPending(false))
  }

  return { confirm, pending }
}
