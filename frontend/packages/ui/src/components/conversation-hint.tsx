import { ArrowRight, MessageSquareText } from "lucide-react"
import { cn } from "@workspace/ui/lib/utils"

export interface ConversationHintProps {
  lead: string
  prompt: string
  showArrow?: boolean
  className?: string
}

export function ConversationHint({ lead, prompt, showArrow = false, className }: ConversationHintProps) {
  const href = `/?draft=${encodeURIComponent(prompt)}`
  return (
    <div className={cn("border-t pt-6", className)}>
      <p className="text-muted-foreground text-sm">{lead}</p>
      <a
        href={href}
        target="_top"
        data-testid="empty-chat-example"
        className="group hover:bg-accent/50 mt-3 flex items-center gap-3 rounded-lg border p-3 text-left transition-colors"
      >
        <MessageSquareText className="text-primary size-4 shrink-0" />
        <span className="text-muted-foreground flex-1 text-sm italic">{`„${prompt}"`}</span>
        {showArrow && (
          <ArrowRight className="text-muted-foreground size-4 transition-transform group-hover:translate-x-0.5" />
        )}
      </a>
    </div>
  )
}
