import { useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Image as ImageIcon, Type, Upload } from 'lucide-react'
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
import { addLogo, addText } from '@/lib/api'

const TERRITORIES = ['CZ', 'EU'] as const

function parseClasses(value: string): number[] {
  return Array.from(
    new Set(
      value
        .split(/[^0-9]+/)
        .map((s) => parseInt(s, 10))
        .filter((n) => Number.isInteger(n) && n >= 1 && n <= 45),
    ),
  )
}

function parseList(value: string): string[] {
  return value.split(/[,;\s]+/).map((s) => s.trim()).filter(Boolean)
}

export function AddMarkDialog({
  open,
  onOpenChange,
  onAdded,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onAdded: (uuid?: string) => void
}) {
  const { t } = useTranslation()
  const [tab, setTab] = useState('text')
  const [text, setText] = useState('')
  const [logoText, setLogoText] = useState('')
  const [nice, setNice] = useState('')
  const [vienna, setVienna] = useState('')
  const [territories, setTerritories] = useState<string[]>(['CZ', 'EU'])
  const [owner, setOwner] = useState('')
  const [label, setLabel] = useState('')
  const [logo, setLogo] = useState<{ dataUrl: string; mime: string } | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const fileRef = useRef<HTMLInputElement>(null)

  const reset = () => {
    setText(''); setLogoText(''); setNice(''); setVienna('')
    setTerritories(['CZ', 'EU']); setOwner(''); setLabel(''); setLogo(null)
  }

  const toggleTerritory = (terr: string) =>
    setTerritories((cur) => (cur.includes(terr) ? cur.filter((x) => x !== terr) : [...cur, terr]))

  const onFile = (file: File | undefined) => {
    if (!file) return
    const reader = new FileReader()
    reader.onload = () => setLogo({ dataUrl: String(reader.result), mime: file.type || 'image/png' })
    reader.readAsDataURL(file)
  }

  const finish = (uuid?: string) => {
    toast.success(t('add.success'))
    reset()
    onOpenChange(false)
    onAdded(uuid)
  }
  const handleError = (e: unknown) => toast.error(e instanceof Error ? e.message : String(e))

  const submitText = () => {
    if (!text.trim()) return
    setSubmitting(true)
    addText({
      text: text.trim(),
      nice_classes: parseClasses(nice),
      territories,
      owner_name: owner.trim(),
      label: label.trim(),
    })
      .then((res) => (res.ok ? finish(res.mark?.uuid) : toast.error(res.error ?? t('add.error'))))
      .catch(handleError)
      .finally(() => setSubmitting(false))
  }

  const submitLogo = () => {
    if (!logo) return
    setSubmitting(true)
    addLogo({
      logo_base64: logo.dataUrl,
      mime: logo.mime,
      text: logoText.trim(),
      nice_classes: parseClasses(nice),
      territories,
      vienna_codes: parseList(vienna),
      owner_name: owner.trim(),
      label: label.trim(),
    })
      .then((res) => (res.ok ? finish(res.mark?.uuid) : toast.error(res.error ?? t('add.error'))))
      .catch(handleError)
      .finally(() => setSubmitting(false))
  }

  const filters = (
    <>
      <div className="space-y-1.5">
        <Label htmlFor="nice">{t('add.nice')}</Label>
        <Input id="nice" placeholder="9, 42" value={nice} onChange={(e) => setNice(e.target.value)} disabled={submitting} data-testid="nice-input" />
        <p className="text-muted-foreground text-xs">{t('add.niceHint')}</p>
      </div>
      <div className="space-y-1.5">
        <Label>{t('add.territory')}</Label>
        <div className="flex gap-2">
          {TERRITORIES.map((terr) => (
            <button
              key={terr}
              type="button"
              onClick={() => toggleTerritory(terr)}
              disabled={submitting}
              data-testid={`territory-${terr}`}
              aria-pressed={territories.includes(terr)}
              className={`rounded-md border px-3 py-1.5 text-sm transition-colors ${
                territories.includes(terr) ? 'border-primary bg-secondary text-primary' : 'text-muted-foreground'
              }`}
            >
              {t(`territory.${terr}`)}
            </button>
          ))}
        </div>
      </div>
    </>
  )

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="font-display text-xl font-normal">{t('add.title')}</DialogTitle>
          <DialogDescription>{t('add.description')}</DialogDescription>
        </DialogHeader>

        <Tabs value={tab} onValueChange={setTab} className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="text" data-testid="tab-text"><Type className="size-4" />{t('add.word')}</TabsTrigger>
            <TabsTrigger value="logo" data-testid="tab-logo"><ImageIcon className="size-4" />{t('add.figurative')}</TabsTrigger>
          </TabsList>

          <TabsContent value="text" className="max-h-[60vh] space-y-4 overflow-y-auto pt-2">
            <div className="space-y-1.5">
              <Label htmlFor="text">{t('add.text')}</Label>
              <Input id="text" autoComplete="off" placeholder="CODEXIS" value={text} onChange={(e) => setText(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && submitText()} disabled={submitting} data-testid="text-input" />
            </div>
            {filters}
            <OwnerLabel owner={owner} setOwner={setOwner} label={label} setLabel={setLabel} disabled={submitting} t={t} />
            <DialogFooter>
              <Button variant="outline" onClick={() => onOpenChange(false)} disabled={submitting}>{t('common.cancel')}</Button>
              <Button onClick={submitText} disabled={submitting || !text.trim()} data-testid="submit-text">
                {submitting ? t('add.adding') : t('add.add')}
              </Button>
            </DialogFooter>
          </TabsContent>

          <TabsContent value="logo" className="max-h-[60vh] space-y-4 overflow-y-auto pt-2">
            <div className="space-y-1.5">
              <Label>{t('add.logo')}</Label>
              <input ref={fileRef} type="file" accept="image/*" className="hidden" onChange={(e) => onFile(e.target.files?.[0])} data-testid="logo-input" />
              <button
                type="button"
                onClick={() => fileRef.current?.click()}
                disabled={submitting}
                className="border-input hover:bg-accent/40 flex w-full flex-col items-center gap-2 rounded-lg border border-dashed p-4 transition-colors"
                data-testid="logo-drop"
              >
                {logo ? (
                  <img src={logo.dataUrl} alt="" className="max-h-24 object-contain" />
                ) : (
                  <>
                    <Upload className="text-muted-foreground size-6" />
                    <span className="text-muted-foreground text-sm">{t('add.logoHint')}</span>
                  </>
                )}
              </button>
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="logoText">{t('add.logoText')}</Label>
              <Input id="logoText" autoComplete="off" value={logoText} onChange={(e) => setLogoText(e.target.value)} disabled={submitting} data-testid="logo-text-input" />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="vienna">{t('add.vienna')}</Label>
              <Input id="vienna" placeholder="26.4.1, 27.5" value={vienna} onChange={(e) => setVienna(e.target.value)} disabled={submitting} />
            </div>
            {filters}
            <OwnerLabel owner={owner} setOwner={setOwner} label={label} setLabel={setLabel} disabled={submitting} t={t} />
            <DialogFooter>
              <Button variant="outline" onClick={() => onOpenChange(false)} disabled={submitting}>{t('common.cancel')}</Button>
              <Button onClick={submitLogo} disabled={submitting || !logo} data-testid="submit-logo">
                {submitting ? t('add.adding') : t('add.add')}
              </Button>
            </DialogFooter>
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  )
}

function OwnerLabel({
  owner, setOwner, label, setLabel, disabled, t,
}: {
  owner: string; setOwner: (v: string) => void
  label: string; setLabel: (v: string) => void
  disabled: boolean; t: (k: string) => string
}) {
  return (
    <div className="grid grid-cols-2 gap-3">
      <div className="space-y-1.5">
        <Label htmlFor="owner">{t('add.owner')}</Label>
        <Input id="owner" autoComplete="off" value={owner} onChange={(e) => setOwner(e.target.value)} disabled={disabled} />
      </div>
      <div className="space-y-1.5">
        <Label htmlFor="label">{t('add.label')}</Label>
        <Input id="label" autoComplete="off" placeholder={t('add.labelPlaceholder')} value={label} onChange={(e) => setLabel(e.target.value)} disabled={disabled} data-testid="label-input" />
      </div>
    </div>
  )
}
