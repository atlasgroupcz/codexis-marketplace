import { useState } from 'react'
import { Settings as SettingsIcon } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@workspace/ui/components/button'
import { toast } from '@workspace/ui/components/sonner'
import { ListSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { ProceedingRow } from './proceeding-row'
import { AddProceedingForm } from './add-proceeding-form'
import { SettingsDialog } from './settings-dialog'
import { ApiKeyBanner } from './api-key-banner'
import { useOverview } from '@/hooks/use-overview'

interface ProceedingsListProps {
  onSelect: (uuid: string) => void
}

export function ProceedingsList({ onSelect }: ProceedingsListProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const [settingsOpen, setSettingsOpen] = useState(false)

  const Header = () => (
    <div className="flex items-center justify-between">
      <h1 className="text-xl font-semibold">{t('proceedings.title')}</h1>
      <Button
        variant="ghost"
        size="sm"
        onClick={() => setSettingsOpen(true)}
        title={t('settings.title')}
        data-testid="open-settings"
      >
        <SettingsIcon className="size-4" />
      </Button>
    </div>
  )

  if (loading && !data) {
    return <ListSkeleton />
  }

  if (error) {
    return (
      <>
        <ErrorMessage message={error.message} onRetry={refetch} />
        <SettingsDialog
          open={settingsOpen}
          onOpenChange={setSettingsOpen}
          configured={data?.api_key_configured ?? false}
          maskedKey={data?.api_key_masked ?? ''}
          onSaved={refetch}
        />
      </>
    )
  }

  if (!data) return null

  const apiKeyConfigured = data.api_key_configured

  const handleAdded = () => {
    refetch()
  }

  const handleChecked = (newChanges: number, errors: number) => {
    if (errors > 0) {
      toast.error(t('proceedings.checkResultErrors', { newChanges, errors }))
    } else if (newChanges > 0) {
      toast.success(t('proceedings.checkResultChanges', { count: newChanges }))
    } else {
      toast.success(t('proceedings.checkResultNone'))
    }
    refetch()
  }

  // Pre-config: show only banner + header
  if (!apiKeyConfigured) {
    return (
      <div className="space-y-4 p-6">
        <Header />
        <ApiKeyBanner onOpenSettings={() => setSettingsOpen(true)} />
        <SettingsDialog
          open={settingsOpen}
          onOpenChange={setSettingsOpen}
          configured={data?.api_key_configured ?? false}
          maskedKey={data?.api_key_masked ?? ''}
          onSaved={refetch}
        />
      </div>
    )
  }

  return (
    <div className="space-y-4 p-6">
      <Header />

      <AddProceedingForm onAdded={handleAdded} onChecked={handleChecked} />

      {data.proceedings.length === 0 ? (
        <p className="text-muted-foreground text-sm" data-testid="empty-state">
          {t('proceedings.empty')}
        </p>
      ) : (
        <div className="overflow-x-auto rounded-lg border">
          <table className="w-full text-sm" data-testid="proceedings-table">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="px-4 py-3 text-left font-medium">{t('proceedings.colNumber')}</th>
                <th className="px-4 py-3 text-left font-medium">{t('proceedings.colLabel')}</th>
                <th className="px-4 py-3 text-left font-medium">{t('proceedings.colStav')}</th>
                <th className="px-4 py-3 text-left font-medium">{t('proceedings.colOperations')}</th>
                <th className="px-4 py-3 text-left font-medium">{t('proceedings.colUhrada')}</th>
              </tr>
            </thead>
            <tbody>
              {data.proceedings.map((proc) => (
                <ProceedingRow
                  key={proc.uuid}
                  proceeding={proc}
                  onClick={onSelect}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}

      <SettingsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        configured={data.api_key_configured}
        maskedKey={data.api_key_masked}
        onSaved={refetch}
      />
    </div>
  )
}
