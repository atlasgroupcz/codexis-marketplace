/* Branded spot illustrations for empty states - CODEXIS green, editorial line+fill. */

type ArtProps = { className?: string }

/** "We watch case law for your topics" - scales of justice under a radar sweep. */
export function WatchCaseLawArt({ className }: ArtProps) {
  return (
    <svg viewBox="0 0 200 150" fill="none" className={className} aria-hidden role="img">
      <defs>
        <linearGradient id="lawsweep" x1="100" y1="72" x2="150" y2="36" gradientUnits="userSpaceOnUse">
          <stop stopColor="#5ea500" stopOpacity="0.38" />
          <stop offset="1" stopColor="#5ea500" stopOpacity="0" />
        </linearGradient>
      </defs>
      <circle cx="100" cy="72" r="60" fill="#f4f7f0" />
      <circle cx="100" cy="72" r="60" stroke="#dfe8d1" />
      <path d="M100 72V18A54 54 0 0 1 146 44Z" fill="url(#lawsweep)" />
      {/* scales of justice */}
      <g stroke="#3f7000" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" fill="none">
        <path d="M100 40v68" />
        <path d="M78 118h44" />
        <path d="M70 52h60" />
        <path d="M70 52l-12 22M70 52l12 22" />
        <path d="M124 52l-12 22M124 52l12 22" />
      </g>
      <path d="M52 74a18 8 0 0 0 36 0Z" fill="#e7f0d8" stroke="#5ea500" strokeWidth="2" />
      <path d="M106 74a18 8 0 0 0 36 0Z" fill="#e7f0d8" stroke="#5ea500" strokeWidth="2" />
      <circle cx="100" cy="40" r="4.5" fill="#5ea500" />
      <circle cx="139" cy="46" r="8" fill="#5ea500" fillOpacity="0.16" />
      <circle cx="139" cy="46" r="3.5" fill="#5ea500" />
    </svg>
  )
}

/** "No new reports yet" - a report sheet at rest with a green check. */
export function NoReportsArt({ className }: ArtProps) {
  return (
    <svg viewBox="0 0 200 150" fill="none" className={className} aria-hidden role="img">
      <ellipse cx="100" cy="130" rx="52" ry="7" fill="#eef3e6" />
      <circle cx="96" cy="68" r="50" fill="#f4f7f0" />
      <rect x="70" y="32" width="56" height="70" rx="6" fill="#ffffff" stroke="#e6e6e0" strokeWidth="2" />
      <rect x="70" y="32" width="56" height="16" rx="6" fill="#f4f7f0" />
      <path
        d="M80 60h36M80 70h36M80 80h22"
        stroke="#deded7"
        strokeWidth="3"
        strokeLinecap="round"
      />
      <circle cx="134" cy="96" r="16" fill="#5ea500" />
      <path
        d="m127 96 4.4 4.6 9-9.4"
        stroke="#ffffff"
        strokeWidth="2.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}
