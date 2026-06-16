import { useTranslation } from 'react-i18next'
import { Cell, Pie, PieChart } from 'recharts'
import { ShieldAlert } from 'lucide-react'
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from '@workspace/ui/components/chart'
import type { ChartConfig } from '@workspace/ui/components/chart'
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

const chartConfig = {
  value: { label: 'Shoda' },
  match: { label: 'Shoda' },
  rest: { label: '' },
} satisfies ChartConfig

/** Donut gauge of the overall similarity with a large icon + % in the centre. */
export function SimilarityDonut({ overall, tier }: { overall: number; tier: Tier }) {
  const { t } = useTranslation()
  const pct = Math.round(Math.max(0, Math.min(1, overall)) * 100)
  const data = [
    { key: 'match', label: t('score.overall'), value: pct, fill: TIER_FILL[tier] },
    { key: 'rest', label: '', value: 100 - pct, fill: 'var(--muted)' },
  ]

  return (
    <div className="relative">
      <ChartContainer config={chartConfig} className="aspect-square h-[156px]">
        <PieChart>
          <ChartTooltip cursor={false} content={<ChartTooltipContent hideLabel nameKey="label" />} />
          <Pie
            data={data}
            dataKey="value"
            nameKey="label"
            innerRadius={54}
            outerRadius={74}
            startAngle={90}
            endAngle={-270}
            strokeWidth={2}
            isAnimationActive={false}
          >
            {data.map((entry) => (
              <Cell key={entry.key} fill={entry.fill} />
            ))}
          </Pie>
        </PieChart>
      </ChartContainer>
      <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center">
        <ShieldAlert className={`size-10 ${TIER_TEXT[tier]}`} strokeWidth={1.75} />
        <span className="text-foreground mt-1 text-2xl font-semibold tabular-nums">{pct}%</span>
      </div>
    </div>
  )
}
