import type { ChartConfig } from '@workspace/ui/components/chart'
import { ChartContainer } from '@workspace/ui/components/chart'
import { ShieldAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Cell, Pie, PieChart } from 'recharts'
import type { Tier } from '@/lib/schemas'

const TIER_FILL: Record<Tier, string> = {
  high: '#ef4444',
  medium: '#f59e0b',
  low: '#64748b',
}
const TIER_TEXT: Record<Tier, string> = {
  high: 'text-red-500',
  medium: 'text-amber-500',
  low: 'text-slate-400',
}

const chartConfig = { value: { label: 'Shoda' } } satisfies ChartConfig

/** Light donut gauge of the overall similarity: icon in the centre, % below it. */
export function SimilarityDonut({ overall, tier }: { overall: number; tier: Tier }) {
  const { t } = useTranslation()
  const pct = Math.round(Math.max(0, Math.min(1, overall)) * 100)
  const data = [
    { key: 'match', value: pct, fill: TIER_FILL[tier] },
    { key: 'rest', value: 100 - pct, fill: 'var(--muted)' },
  ]

  return (
    <div className="flex w-[104px] flex-col items-center">
      <div className="relative size-[104px]">
        <ChartContainer config={chartConfig} className="size-full">
          <PieChart>
            <Pie
              data={data}
              dataKey="value"
              nameKey="key"
              innerRadius={38}
              outerRadius={50}
              startAngle={90}
              endAngle={-270}
              strokeWidth={0}
              isAnimationActive={false}
            >
              {data.map((entry) => (
                <Cell key={entry.key} fill={entry.fill} />
              ))}
            </Pie>
          </PieChart>
        </ChartContainer>
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <ShieldAlert className={`size-7 ${TIER_TEXT[tier]}`} strokeWidth={1.75} />
        </div>
      </div>
      <span className={`mt-1 text-xl font-semibold tabular-nums ${TIER_TEXT[tier]}`}>{pct}%</span>
      <span className="text-muted-foreground text-[10px] leading-none">{t('score.overall')}</span>
    </div>
  )
}
