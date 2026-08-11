import { Button } from '@workspace/ui/components/button'
import { AlertTriangle } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface ErrorMessageProps {
  message: string
  onRetry?: () => void
}

export function ErrorMessage({ message, onRetry }: ErrorMessageProps) {
  const { t } = useTranslation()
  return (
    <div className="flex flex-col items-center gap-3 rounded-lg border border-red-200 bg-red-50 p-6 text-center text-red-900 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200">
      <AlertTriangle className="size-6" />
      <p className="text-sm">{message}</p>
      {onRetry && (
        <Button variant="outline" size="sm" onClick={onRetry}>
          {t('common.retry')}
        </Button>
      )}
    </div>
  )
}
