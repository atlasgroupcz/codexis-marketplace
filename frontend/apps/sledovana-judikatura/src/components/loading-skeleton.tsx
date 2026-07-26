import { Skeleton } from '@workspace/ui/components/skeleton'

export function ListSkeleton() {
  return (
    <div className="space-y-4 p-6">
      {Array.from({ length: 3 }, (_, i) => (
        <div key={i} className="flex items-center gap-4 rounded-xl border p-4">
          <div className="flex-1 space-y-2">
            <Skeleton className="h-5 w-48" />
            <Skeleton className="h-4 w-32" />
          </div>
          <Skeleton className="h-6 w-16" />
          <Skeleton className="h-6 w-16" />
        </div>
      ))}
    </div>
  )
}

export function FeedSkeleton() {
  return (
    <ol className="relative space-y-1 border-l pl-5">
      {Array.from({ length: 5 }, (_, i) => (
        <li key={i} className="space-y-2 px-3 py-2.5">
          <Skeleton className="h-3 w-20" />
          <Skeleton className="h-4 w-3/4" />
          <Skeleton className="h-3 w-1/2" />
        </li>
      ))}
    </ol>
  )
}

/** @deprecated Use ListSkeleton instead */
export const LoadingSkeleton = ListSkeleton
