import { Skeleton } from '@workspace/ui/components/skeleton'

/** Placeholder rows that mirror the real subject list. */
export function ListSkeleton() {
  return (
    <div>
      {Array.from({ length: 6 }).map((_, i) => (
        <div key={i} className="flex items-center gap-2.5 border-b px-3 py-2.5">
          <Skeleton className="size-2 shrink-0 rounded-full" />
          <div className="flex-1 space-y-1.5">
            <Skeleton className="h-3.5 w-36" />
            <Skeleton className="h-3 w-24" />
          </div>
          <Skeleton className="h-3.5 w-7" />
        </div>
      ))}
    </div>
  )
}

/** Placeholder that mirrors the real detail: header, tabs, and document rows. */
export function DetailSkeleton() {
  return (
    <div className="space-y-6 p-4 sm:p-6">
      <div className="space-y-2.5">
        <Skeleton className="h-6 w-56" />
        <Skeleton className="h-3.5 w-80" />
        <Skeleton className="h-3.5 w-64" />
      </div>
      <Skeleton className="h-6 w-44" />
      <div className="space-y-5">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="grid grid-cols-[32px_1fr_auto] items-start gap-3.5">
            <Skeleton className="size-8 shrink-0 rounded-lg" />
            <div className="space-y-2">
              <Skeleton className="h-3.5 w-3/4" />
              <Skeleton className="h-3 w-40" />
            </div>
            <Skeleton className="h-[30px] w-24 rounded-lg" />
          </div>
        ))}
      </div>
    </div>
  )
}
