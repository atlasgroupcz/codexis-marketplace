import { useTranslation } from 'react-i18next'
import { Cell, Pie, PieChart } from 'recharts'
import { ShieldAlert } from 'lucide-react'
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from '@workspace/ui/components/chart'
import type { ChartConfig } from '@workspace/ui/components/chart'
import type { Scores, Tier } from '@/lib/schemas'

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

export function SimilarityDonut({ scores, tier }: { scores: Scores; tier: Tier }) {
  const { t } = useTranslation()
  const pct = Math.round(Math.max(0, Math.min(1, scores.overall)) * 100)
  const data = [
    { key: 'match', label: t('score.overall'), value: pct, fill: TIER_FILL[tier] },
    { key: 'rest', label: '', value: 100 - pct, fill: 'var(--muted)' },
  ]

  const breakdown: Array<{ key: string; value: number }> = [
    { key: 'text_combined', value: scores.text_combined },
    { key: 'text_phonetic', value: scores.text_phonetic },
    { key: 'class_overlap', value: scores.class_overlap },
    { key: 'vienna_overlap', value: scores.vienna_overlap },
  ]
  if (scores.image_phash !== null) breakdown.push({ key: 'image_phash', value: scores.image_phash })

  return (
    <div className="flex items-center gap-4">
      <div className="relative shrink-0">
        <ChartContainer config={chartConfig} className="aspect-square h-[132px]">
          <PieChart>
            <ChartTooltip
              cursor={false}
              content={<ChartTooltipContent hideLabel nameKey="label" />}
            />
            <Pie
              data={data}
              dataKey="value"
              nameKey="label"
              innerRadius={46}
              outerRadius={62}
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
          <ShieldAlert className={`size-5 ${TIER_TEXT[tier]}`} strokeWidth={2} />
          <span className="text-foreground mt-0.5 text-xl font-semibold tabular-nums">{pct}%</span>
          <span className="text-muted-foreground text-[10px]">{t('score.overall')}</span>
        </div>
      </div>

      <dl className="min-w-0 flex-1 space-y-1">
        {breakdown.map((b) => (
          <div key={b.key} className="flex items-center justify-between gap-3 text-xs">
            <dt className="text-muted-foreground truncate">{t(`score.${b.key}`)}</dt>
            <dd className="text-foreground/80 font-mono tabular-nums">
              {Math.round(b.value * 100)}%
            </dd>
          </div>
        ))}
      </dl>
    </div>
  )
}
