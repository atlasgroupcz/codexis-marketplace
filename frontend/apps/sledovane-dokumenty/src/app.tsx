import type { NotifyValues } from '@workspace/ui/components/notify-settings'
import { NotifySettings } from '@workspace/ui/components/notify-settings'
import { toast } from '@workspace/ui/components/sonner'
import { useQueryState } from 'nuqs'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { DocumentDetail } from '@/components/document-detail'
import { DocumentsHome } from '@/components/documents-home'
import { ProjectDetail } from '@/components/project-detail'
import { ProjectsHome } from '@/components/projects-home'
import { fetchNotify, saveNotify } from '@/lib/api'
import { sectionParser, uuidParser, viewParser, watchParser } from '@/lib/url-state'

const defaultNotify: NotifyValues = { email: true, inApp: true, recipients: [] }

export function App() {
  const { t } = useTranslation()
  const [section, setSection] = useQueryState('section', sectionParser)
  const [view, setView] = useQueryState('view', viewParser)
  const [uuid, setUuid] = useQueryState('uuid', uuidParser)
  const [watch, setWatch] = useQueryState('watch', watchParser)

  const [settingsOpen, setSettingsOpen] = useState(false)
  const [notify, setNotify] = useState<NotifyValues>(defaultNotify)
  const [userEmail, setUserEmail] = useState('')
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    fetchNotify()
      .then(({ settings, userEmail: email }) => {
        setNotify(settings)
        setUserEmail(email)
      })
      .catch(() => {
        // notify settings are non-critical; leave defaults on failure
      })
  }, [])

  const changeSection = (next: 'documents' | 'projects') => {
    void setSection(next)
  }

  const openSettings = () => setSettingsOpen(true)

  const handleSaveNotify = (values: NotifyValues) => {
    setSaving(true)
    saveNotify(values)
      .then(() => setNotify(values))
      .catch((err: unknown) => toast.error(err instanceof Error ? err.message : String(err)))
      .finally(() => setSaving(false))
  }

  const settingsDialog = (
    <NotifySettings
      open={settingsOpen}
      onOpenChange={setSettingsOpen}
      values={notify}
      userEmail={userEmail}
      labels={{
        title: t('notify.title'),
        description: t('notify.description'),
        save: t('notify.save'),
        cancel: t('notify.cancel'),
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
      }}
      saving={saving}
      onSave={handleSaveNotify}
    />
  )

  if (section === 'projects') {
    if (watch) {
      return (
        <>
          <ProjectDetail
            watchId={watch}
            onBack={() => void setWatch(null)}
            onOpenSettings={openSettings}
          />
          {settingsDialog}
        </>
      )
    }
    return (
      <>
        <ProjectsHome
          section={section}
          onChangeSection={changeSection}
          onSelectWatch={(id) => void setWatch(id)}
          onOpenSettings={openSettings}
        />
        {settingsDialog}
      </>
    )
  }

  const handleSelectDocument = (docUuid: string) => {
    void setView('detail')
    void setUuid(docUuid)
  }

  const handleBack = () => {
    void setView('list')
    void setUuid(null)
  }

  if (view === 'detail' && uuid) {
    return (
      <>
        <DocumentDetail uuid={uuid} onBack={handleBack} onOpenSettings={openSettings} />
        {settingsDialog}
      </>
    )
  }

  return (
    <>
      <DocumentsHome
        section={section}
        onChangeSection={changeSection}
        onSelectDocument={handleSelectDocument}
        onOpenSettings={openSettings}
      />
      {settingsDialog}
    </>
  )
}
