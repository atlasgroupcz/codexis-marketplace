import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Building2, User } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@workspace/ui/components/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@workspace/ui/components/tabs'
import { Input } from '@workspace/ui/components/input'
import { Button } from '@workspace/ui/components/button'
import { Label } from '@workspace/ui/components/label'
import { toast } from '@workspace/ui/components/sonner'
import { addCompany, addPerson } from '@/lib/api'

interface AddSubjectDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onAdded: (uuid?: string) => void
}

export function AddSubjectDialog({ open, onOpenChange, onAdded }: AddSubjectDialogProps) {
  const { t } = useTranslation()
  const [tab, setTab] = useState('firma')
  const [ico, setIco] = useState('')
  const [prijmeni, setPrijmeni] = useState('')
  const [jmeno, setJmeno] = useState('')
  const [datum, setDatum] = useState('')
  const [label, setLabel] = useState('')
  const [submitting, setSubmitting] = useState(false)

  const reset = () => {
    setIco('')
    setPrijmeni('')
    setJmeno('')
    setDatum('')
    setLabel('')
  }

  const finish = (uuid?: string) => {
    toast.success(t('add.success'))
    reset()
    onOpenChange(false)
    onAdded(uuid)
  }

  const handleError = (e: unknown) => {
    toast.error(e instanceof Error ? e.message : String(e))
  }

  const submitCompany = () => {
    if (!ico.trim()) return
    setSubmitting(true)
    addCompany(ico.trim(), label.trim())
      .then((res) => (res.ok ? finish(res.subject?.uuid) : toast.error(res.error ?? t('add.error'))))
      .catch(handleError)
      .finally(() => setSubmitting(false))
  }

  const submitPerson = () => {
    if (!prijmeni.trim() || !datum) return
    setSubmitting(true)
    addPerson(prijmeni.trim(), jmeno.trim(), datum, label.trim())
      .then((res) => (res.ok ? finish(res.subject?.uuid) : toast.error(res.error ?? t('add.error'))))
      .catch(handleError)
      .finally(() => setSubmitting(false))
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="font-display text-xl font-normal">{t('add.title')}</DialogTitle>
          <DialogDescription>{t('add.description')}</DialogDescription>
        </DialogHeader>

        <Tabs value={tab} onValueChange={setTab} className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="firma" data-testid="tab-firma">
              <Building2 className="size-4" />
              {t('add.company')}
            </TabsTrigger>
            <TabsTrigger value="osoba" data-testid="tab-osoba">
              <User className="size-4" />
              {t('add.person')}
            </TabsTrigger>
          </TabsList>

          <TabsContent value="firma" className="space-y-4 pt-2">
            <div className="space-y-1.5">
              <Label htmlFor="ico">{t('add.ico')}</Label>
              <Input
                id="ico"
                inputMode="numeric"
                autoComplete="off"
                placeholder="04863313"
                className="font-mono"
                value={ico}
                onChange={(e) => setIco(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && submitCompany()}
                disabled={submitting}
                data-testid="ico-input"
              />
              <p className="text-muted-foreground text-xs">{t('add.icoHint')}</p>
            </div>
            <LabelField label={label} setLabel={setLabel} disabled={submitting} t={t} />
            <DialogFooter>
              <Button variant="outline" onClick={() => onOpenChange(false)} disabled={submitting}>
                {t('common.cancel')}
              </Button>
              <Button onClick={submitCompany} disabled={submitting || !ico.trim()} data-testid="submit-company">
                {submitting ? t('add.adding') : t('add.add')}
              </Button>
            </DialogFooter>
          </TabsContent>

          <TabsContent value="osoba" className="space-y-4 pt-2">
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1.5">
                <Label htmlFor="prijmeni">{t('add.surname')}</Label>
                <Input
                  id="prijmeni"
                  autoComplete="off"
                  placeholder="Novák"
                  value={prijmeni}
                  onChange={(e) => setPrijmeni(e.target.value)}
                  disabled={submitting}
                  data-testid="surname-input"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="jmeno">{t('add.givenName')}</Label>
                <Input
                  id="jmeno"
                  autoComplete="off"
                  placeholder="Jan"
                  value={jmeno}
                  onChange={(e) => setJmeno(e.target.value)}
                  disabled={submitting}
                  data-testid="given-name-input"
                />
              </div>
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="datum">{t('add.birthDate')}</Label>
              <Input
                id="datum"
                type="date"
                value={datum}
                onChange={(e) => setDatum(e.target.value)}
                disabled={submitting}
                data-testid="birthdate-input"
              />
              <p className="text-muted-foreground text-xs">{t('add.personHint')}</p>
            </div>
            <LabelField label={label} setLabel={setLabel} disabled={submitting} t={t} />
            <DialogFooter>
              <Button variant="outline" onClick={() => onOpenChange(false)} disabled={submitting}>
                {t('common.cancel')}
              </Button>
              <Button
                onClick={submitPerson}
                disabled={submitting || !prijmeni.trim() || !datum}
                data-testid="submit-person"
              >
                {submitting ? t('add.adding') : t('add.add')}
              </Button>
            </DialogFooter>
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  )
}

function LabelField({
  label,
  setLabel,
  disabled,
  t,
}: {
  label: string
  setLabel: (v: string) => void
  disabled: boolean
  t: (k: string) => string
}) {
  return (
    <div className="space-y-1.5">
      <Label htmlFor="label">{t('add.label')}</Label>
      <Input
        id="label"
        autoComplete="off"
        placeholder={t('add.labelPlaceholder')}
        value={label}
        onChange={(e) => setLabel(e.target.value)}
        disabled={disabled}
        data-testid="label-input"
      />
    </div>
  )
}
