import { AppHeader, HeaderGear } from '@workspace/ui/components/app-header'
import { Button } from '@workspace/ui/components/button'
import type { NotifyLabels } from '@workspace/ui/components/notify-settings'
import { NotifySettings } from '@workspace/ui/components/notify-settings'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { useNotify } from '@workspace/ui/hooks/use-notify'
import { Plus, RefreshCw, ShieldAlert } from 'lucide-react'
import { useQueryState } from 'nuqs'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { AddSubjectDialog } from '@/components/add-subject-dialog'
import { EmptySubjects, NoSelection } from '@/components/empty-state'
import { ErrorMessage } from '@/components/error-message'
import { DetailSkeleton, ListSkeleton } from '@/components/loading-skeleton'
import { SubjectDetail } from '@/components/subject-detail'
import { SubjectList } from '@/components/subject-list'
import { useOverview } from '@/hooks/use-overview'
import { checkAll, fetchNotify, saveNotify } from '@/lib/api'
import type { NotifyValues } from '@/lib/schemas'
import { uuidParser } from '@/lib/url-state'

const emptyNotifyValues: NotifyValues = { email: false, inApp: false, recipients: [] }

export function App() {
  const { t, i18n } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const { data: notifyData, refetch: refetchNotify } = useNotify(fetchNotify)
  const [selectedUuid, setSelectedUuid] = useQueryState('uuid', uuidParser)
  const [addOpen, setAddOpen] = useState(false)
  const [checking, setChecking] = useState(false)
  const [detailNonce, setDetailNonce] = useState(0)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [notifySaving, setNotifySaving] = useState(false)

  const subjects = data?.subjects ?? []

  const notifyLabels: NotifyLabels = {
    title: t('notify.title'),
    description: t('notify.description'),
    save: t('notify.save'),
    cancel: t('common.cancel'),
    emailLabel: t('notify.emailLabel'),
    emailDescription: t('notify.emailDescription'),
    yourAddressLabel: t('notify.yourAddressLabel'),
    copyToLabel: t('notify.copyToLabel'),
    addRecipient: t('notify.addRecipient'),
    removeRecipient: t('notify.removeRecipient'),
    addPlaceholder: t('notify.addPlaceholder'),
    invalidEmail: t('notify.invalidEmail'),
    ownAddress: t('notify.ownAddress'),
    duplicate: t('notify.duplicate'),
    inAppLabel: t('notify.inAppLabel'),
    inAppDescription: t('notify.inAppDescription'),
  }

  const handleSaveNotify = (values: NotifyValues) => {
    setNotifySaving(true)
    saveNotify(values)
      .then(() => refetchNotify())
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setNotifySaving(false))
  }

  const handleCheckAll = () => {
    setChecking(true)
    checkAll()
      .then((res) => {
        if (res.errors.length > 0) {
          toast.error(t('app.checkErrors', { count: res.errors.length }))
        } else if (res.new_changes > 0) {
          toast.success(t('app.checkChanges', { count: res.new_changes }))
        } else {
          toast.success(t('app.checkNone'))
        }
        refetch()
        setDetailNonce((n) => n + 1)
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setChecking(false))
  }

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="app-root">
      <AppHeader
        brand={{ icon: ShieldAlert, title: t('app.title'), subtitle: t('app.subtitle') }}
        docs={{ app: 'insolvence', lang: i18n.language, label: t('help.docs') }}
        actions={
          <>
            {subjects.length > 0 && (
              <Button
                variant="outline"
                size="sm"
                onClick={handleCheckAll}
                disabled={checking}
                data-testid="check-all"
              >
                <RefreshCw className={`size-4 ${checking ? 'animate-spin' : ''}`} />
                <span className="hidden sm:inline">{t('app.checkAll')}</span>
              </Button>
            )}
            <Button size="sm" onClick={() => setAddOpen(true)} data-testid="add-subject">
              <Plus className="size-4" />
              <span className="hidden sm:inline">{t('app.add')}</span>
            </Button>
            <HeaderGear onClick={() => setSettingsOpen(true)} title={t('notify.title')} />
          </>
        }
      />

      <main className="min-h-0 flex-1">
        {loading && !data ? (
          <div className="grid h-full min-h-0 md:grid-cols-[320px_1fr]">
            <div className="border-r">
              <ListSkeleton />
            </div>
            <div className="hidden md:block">
              <DetailSkeleton />
            </div>
          </div>
        ) : error ? (
          <div className="p-6">
            <ErrorMessage message={error.message} onRetry={refetch} />
          </div>
        ) : subjects.length === 0 ? (
          <EmptySubjects onAdd={() => setAddOpen(true)} />
        ) : (
          <div className="grid h-full min-h-0 md:grid-cols-[320px_1fr]">
            <div className={`${selectedUuid ? 'hidden md:block' : 'block'} min-h-0 border-r`}>
              <SubjectList
                subjects={subjects}
                selectedUuid={selectedUuid}
                onSelect={(uuid) => void setSelectedUuid(uuid)}
              />
            </div>
            <div className={`${selectedUuid ? 'block' : 'hidden md:block'} min-h-0`}>
              {selectedUuid ? (
                <SubjectDetail
                  key={`${selectedUuid}:${detailNonce}`}
                  uuid={selectedUuid}
                  onBack={() => void setSelectedUuid(null)}
                  onRemoved={() => {
                    void setSelectedUuid(null)
                    refetch()
                  }}
                  onChanged={refetch}
                />
              ) : (
                <NoSelection />
              )}
            </div>
          </div>
        )}
      </main>

      <AddSubjectDialog
        open={addOpen}
        onOpenChange={setAddOpen}
        onAdded={(uuid) => {
          refetch()
          if (uuid) void setSelectedUuid(uuid)
        }}
      />
      <NotifySettings
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        values={notifyData?.settings ?? emptyNotifyValues}
        userEmail={notifyData?.userEmail ?? ''}
        labels={notifyLabels}
        saving={notifySaving}
        onSave={handleSaveNotify}
      />
      <Toaster position="bottom-right" />
    </div>
  )
}
