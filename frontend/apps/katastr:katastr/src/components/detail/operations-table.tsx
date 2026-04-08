import { useTranslation } from 'react-i18next'
import { formatDateTime } from '@/lib/format'
import type { Operace } from '@/lib/schemas'

interface OperationsTableProps {
  operations: Operace[]
}

export function OperationsTable({ operations }: OperationsTableProps) {
  const { t } = useTranslation()

  return (
    <div>
      <h2 className="mb-3 text-lg font-semibold">{t('proceedings.colOperations')}</h2>
      {operations.length === 0 ? (
        <p className="text-muted-foreground text-sm">{t('proceedings.noOperations')}</p>
      ) : (
        <div className="overflow-x-auto rounded-lg border">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="px-4 py-2 text-left font-medium">
                  {t('proceedings.operation')}
                </th>
                <th className="px-4 py-2 text-left font-medium">
                  {t('proceedings.operationDate')}
                </th>
              </tr>
            </thead>
            <tbody>
              {operations.map((op, i) => (
                <tr key={i} className="border-b last:border-b-0">
                  <td className="px-4 py-2">{op.nazev}</td>
                  <td className="px-4 py-2">{formatDateTime(op.datumProvedeni)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
