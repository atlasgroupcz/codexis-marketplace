import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from '@workspace/ui/components/hover-card'

/** A small logo thumbnail that shows an enlarged preview on hover. */
export function LogoZoom({
  src,
  alt = '',
  className,
}: {
  src: string
  alt?: string
  className?: string
}) {
  return (
    <HoverCard openDelay={80} closeDelay={60}>
      <HoverCardTrigger asChild>
        <img src={src} alt={alt} className={`cursor-zoom-in ${className ?? ''}`} />
      </HoverCardTrigger>
      <HoverCardContent side="right" className="w-auto p-2">
        <img src={src} alt={alt} className="bg-background h-56 w-56 rounded object-contain" />
      </HoverCardContent>
    </HoverCard>
  )
}
