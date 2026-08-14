// The two panes that show an outcome - the header badge and the timeline's
// closing chip - paint it the same way, so the colours live in one place and a
// palette change lands once.
//
// Only these two are shared. The badge's other colours have a single caller and
// stay in `stav-badge.tsx`, next to the keyword that picks them.
//
// They are here rather than in `lib/stav.ts` because that module reads meaning
// out of ČÚZK's free text and knows nothing about how it is drawn.
export const CLOSED_CLASSES = 'bg-stone-100 text-stone-600'
export const NEGATIVE_CLASSES = 'bg-red-100 text-red-700'
