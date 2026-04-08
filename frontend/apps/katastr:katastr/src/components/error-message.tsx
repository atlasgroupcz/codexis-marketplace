import { AlertCircle, RefreshCw } from 'lucide-react'
import { Button } from '@workspace/ui/components/button'
import { useTranslation } from 'react-i18next'

interface ErrorMessageProps {
  message: string
  onRetry?: () => void
}

export function ErrorMessage({ message, onRetry }: ErrorMessageProps) {
  const { t } = useTranslation()
  return (
    <div className="flex flex-col items-center gap-4 p-12 text-center">
      <AlertCircle className="text-destructive size-10" />
      <p className="text-muted-foreground text-sm">{message}</p>
      {onRetry && (
        <Button variant="outline" size="sm" onClick={onRetry}>
          <RefreshCw className="mr-2 size-4" />
          {t('common.retry')}
        </Button>
      )}
    </div>
  )
}
