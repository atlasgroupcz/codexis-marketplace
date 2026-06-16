import { useTranslation } from 'react-i18next'
import { Info, Landmark, ShieldAlert, ShieldCheck } from 'lucide-react'
import { Badge } from '@workspace/ui/components/badge'
import { Card, CardContent } from '@workspace/ui/components/card'
import { ErrorMessage } from './error-message'
import { useDph } from '@/hooks/use-dph'
import { formatDate } from '@/lib/format'

/** VAT-payer reliability + published bank accounts (MF registry), for a company. */
export function DphPanel({ ico }: { ico: string }) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useDph(ico)

  if (loading && !data) {
    return (
      <div className="space-y-3">
        <div className="bg-muted h-16 animate-pulse rounded-xl" />
        <div className="bg-muted h-10 animate-pulse rounded-lg" />
      </div>
    )
  }
  if (error) return <ErrorMessage message={error.message} onRetry={refetch} />
  if (!data) return null

  const status = !data.found || !data.je_platce ? 'none' : data.nespolehlivy ? 'bad' : 'ok'

  return (
    <div className="space-y-3">
      <Card className="gap-0 py-0">
        <CardContent className="flex items-start gap-3 p-4">
          {status === 'bad' ? (
            <ShieldAlert className="mt-0.5 size-5 shrink-0 text-red-600" />
          ) : status === 'ok' ? (
            <ShieldCheck className="text-brand-700 mt-0.5 size-5 shrink-0" />
          ) : (
            <Info className="text-muted-foreground mt-0.5 size-5 shrink-0" />
          )}
          <div className="min-w-0">
            <p
              className={`text-sm font-medium ${
                status === 'bad' ? 'text-red-700' : status === 'ok' ? 'text-brand-700' : ''
              }`}
            >
              {status === 'bad'
                ? t('dph.unreliable')
                : status === 'ok'
                  ? t('dph.reliable')
                  : t('dph.notPayer')}
            </p>
            {status === 'bad' && data.nespolehlivy_od && (
              <p className="text-muted-foreground mt-0.5 text-xs">
                {t('dph.unreliableSince', { date: formatDate(data.nespolehlivy_od) })}
              </p>
            )}
            {data.cislo_fu && (
              <p className="text-muted-foreground mt-0.5 text-xs">
                {t('dph.taxOffice')}: {data.cislo_fu}
              </p>
            )}
          </div>
        </CardContent>
      </Card>

      {data.je_platce && (
        <div>
          <p className="text-foreground mb-2 text-sm font-medium">{t('dph.accounts')}</p>
          {data.ucty.length === 0 ? (
            <p className="text-muted-foreground text-xs">{t('dph.noAccounts')}</p>
          ) : (
            <div className="space-y-2">
              {data.ucty.map((u, i) => (
                <Card key={`${u.ucet}-${i}`} className="gap-0 py-0">
                  <CardContent className="flex flex-wrap items-center gap-x-3 gap-y-1 p-3">
                    <Landmark className="text-muted-foreground size-4 shrink-0" />
                    <span className="font-mono text-sm">{u.ucet}</span>
                    {u.iban && (
                      <Badge variant="outline" className="text-[10px]">
                        IBAN
                      </Badge>
                    )}
                    {u.datum_zverejneni && (
                      <span className="text-muted-foreground ml-auto text-xs">
                        {t('dph.publishedOn', { date: formatDate(u.datum_zverejneni) })}
                      </span>
                    )}
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      )}

      <p className="text-muted-foreground text-xs">{t('dph.source')}</p>
    </div>
  )
}
