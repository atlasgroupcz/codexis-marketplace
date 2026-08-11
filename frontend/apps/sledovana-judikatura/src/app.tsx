import type { NotifyValues } from '@workspace/ui/components/notify-settings'
import { NotifySettings } from '@workspace/ui/components/notify-settings'
import { toast } from '@workspace/ui/components/sonner'
import { useQueryState } from 'nuqs'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { JudikaturaHome } from '@/components/judikatura-home'
import { ReportView } from '@/components/report-view'
import { TopicDetail } from '@/components/topic-detail'
import { fetchNotify, saveNotify } from '@/lib/api'
import { reportParser, uuidParser, viewParser } from '@/lib/url-state'

const emptyNotify: NotifyValues = { email: false, inApp: false, recipients: [] }

export function App() {
  const { t } = useTranslation()
  const [view, setView] = useQueryState('view', viewParser)
  const [uuid, setUuid] = useQueryState('uuid', uuidParser)
  const [reportId, setReportId] = useQueryState('report', reportParser)

  const [settingsOpen, setSettingsOpen] = useState(false)
  const [notify, setNotify] = useState<NotifyValues>(emptyNotify)
  const [userEmail, setUserEmail] = useState('')
  const [savingNotify, setSavingNotify] = useState(false)

  useEffect(() => {
    fetchNotify()
      .then(({ settings, userEmail: email }) => {
        setNotify(settings)
        setUserEmail(email)
      })
      .catch(() => {
        // notify settings are best-effort on mount; the dialog still opens with defaults
      })
  }, [])

  const handleSaveNotify = async (values: NotifyValues) => {
    setSavingNotify(true)
    try {
      await saveNotify(values)
      setNotify(values)
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : String(e))
    } finally {
      setSavingNotify(false)
    }
  }

  const notifyLabels = {
    title: t('judikatura.notify.title'),
    description: t('judikatura.notify.description'),
    save: t('judikatura.notify.save'),
    cancel: t('judikatura.cancel'),
    emailLabel: t('judikatura.notify.emailLabel'),
    emailDescription: t('judikatura.notify.emailDescription'),
    yourAddressLabel: t('judikatura.notify.yourAddressLabel'),
    copyToLabel: t('judikatura.notify.copyToLabel'),
    addRecipient: t('judikatura.notify.addRecipient'),
    removeRecipient: t('judikatura.notify.removeRecipient'),
    addPlaceholder: t('judikatura.notify.addPlaceholder'),
    invalidEmail: t('judikatura.notify.invalidEmail'),
    ownAddress: t('judikatura.notify.ownAddress'),
    duplicate: t('judikatura.notify.duplicate'),
    inAppLabel: t('judikatura.notify.inAppLabel'),
    inAppDescription: t('judikatura.notify.inAppDescription'),
  }

  const onOpenSettings = () => setSettingsOpen(true)

  const handleSelectReport = (topicUuid: string, rid: string) => {
    void setUuid(topicUuid)
    void setReportId(rid)
    void setView('report')
  }

  const handleOpenTopic = (topicUuid: string) => {
    void setUuid(topicUuid)
    void setReportId(null)
    void setView('detail')
  }

  const handleBackToList = () => {
    void setView('list')
    void setUuid(null)
    void setReportId(null)
  }

  const screen =
    view === 'report' && uuid && reportId ? (
      <ReportView
        uuid={uuid}
        reportId={reportId}
        onBack={handleBackToList}
        onOpenSettings={onOpenSettings}
      />
    ) : view === 'detail' && uuid ? (
      <TopicDetail
        uuid={uuid}
        onBack={handleBackToList}
        onSelectReport={(rid) => handleSelectReport(uuid, rid)}
        onOpenSettings={onOpenSettings}
      />
    ) : (
      <JudikaturaHome
        onSelectReport={handleSelectReport}
        onOpenTopic={handleOpenTopic}
        onOpenSettings={onOpenSettings}
      />
    )

  return (
    <>
      {screen}
      <NotifySettings
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        values={notify}
        userEmail={userEmail}
        labels={notifyLabels}
        saving={savingNotify}
        onSave={(v) => void handleSaveNotify(v)}
      />
    </>
  )
}
