/* Custom inline icons for the insolvence app (brand-tuned, not from an icon pack). */

type IconProps = { className?: string }

/** Bell with a small green check badge — "mark notifications as read". */
export function BellCheckIcon({ className }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" className={className} aria-hidden role="img">
      <path
        d="M5.6 16.2c.9-1 1.4-2.3 1.4-3.7V10a5 5 0 0 1 6.2-4.85"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M17 9.8v2.7c0 1.4.5 2.7 1.4 3.7H5.6"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M10 19.5a2.2 2.2 0 0 0 4 0"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="18" cy="6" r="4" fill="#5ea500" />
      <path
        d="m16.3 6.05 1.15 1.2L19.8 4.7"
        stroke="#fff"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

/** Single check — "mark this item read". */
export function CheckIcon({ className }: IconProps) {
  return (
    <svg viewBox="0 0 24 24" fill="none" className={className} aria-hidden role="img">
      <path
        d="m5 12.5 4.5 4.5L19 7"
        stroke="currentColor"
        strokeWidth="1.9"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}
