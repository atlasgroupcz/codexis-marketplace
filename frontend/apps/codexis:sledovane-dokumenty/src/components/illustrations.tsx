/* Branded spot illustrations for empty states — CODEXIS green, editorial line+fill. */

type ArtProps = { className?: string }

/** "We watch your laws for changes" — a document under a radar sweep. */
export function WatchDocsArt({ className }: ArtProps) {
  return (
    <svg viewBox="0 0 200 150" fill="none" className={className} aria-hidden role="img">
      <defs>
        <linearGradient id="docsweep" x1="100" y1="75" x2="150" y2="38" gradientUnits="userSpaceOnUse">
          <stop stopColor="#5ea500" stopOpacity="0.4" />
          <stop offset="1" stopColor="#5ea500" stopOpacity="0" />
        </linearGradient>
      </defs>
      <circle cx="100" cy="75" r="60" fill="#f4f7f0" />
      <circle cx="100" cy="75" r="60" stroke="#dfe8d1" />
      <circle cx="100" cy="75" r="38" stroke="#e7eedb" />
      {/* document sheet with folded corner */}
      <path
        d="M74 38h36l16 16v58a4 4 0 0 1-4 4H74a4 4 0 0 1-4-4V42a4 4 0 0 1 4-4Z"
        fill="#ffffff"
        stroke="#e6e6e0"
        strokeWidth="2"
      />
      <path d="M110 38v16h16" stroke="#e6e6e0" strokeWidth="2" strokeLinejoin="round" />
      <path
        d="M80 64h28M80 74h36M80 84h36M80 94h22"
        stroke="#deded7"
        strokeWidth="3"
        strokeLinecap="round"
      />
      {/* radar sweep + watch dot */}
      <path d="M100 75V30A45 45 0 0 1 139 52Z" fill="url(#docsweep)" />
      <circle cx="100" cy="75" r="4" fill="#3f7000" />
      <circle cx="133" cy="50" r="8" fill="#5ea500" fillOpacity="0.16" />
      <circle cx="133" cy="50" r="3.5" fill="#5ea500" />
    </svg>
  )
}

/** "Nothing changed — you're covered" — a document at rest with a green check. */
export function NoChangesArt({ className }: ArtProps) {
  return (
    <svg viewBox="0 0 200 150" fill="none" className={className} aria-hidden role="img">
      <ellipse cx="100" cy="130" rx="52" ry="7" fill="#eef3e6" />
      <circle cx="96" cy="68" r="50" fill="#f4f7f0" />
      <rect
        x="72"
        y="34"
        width="52"
        height="68"
        rx="6"
        fill="#ffffff"
        stroke="#e6e6e0"
        strokeWidth="2"
        transform="rotate(-6 98 68)"
      />
      <path
        d="M83 52h30M81 62h34M82 72h24"
        stroke="#deded7"
        strokeWidth="3"
        strokeLinecap="round"
        transform="rotate(-6 98 68)"
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
