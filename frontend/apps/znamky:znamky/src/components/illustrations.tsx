/* Branded spot illustrations for empty states - CODEXIS green, editorial line+fill. */

type ArtProps = { className?: string }

/** "We watch the ISIR register for you" - a radar sweeping over the register. */
export function WatchRegisterArt({ className }: ArtProps) {
  return (
    <svg viewBox="0 0 200 150" fill="none" className={className} aria-hidden role="img">
      <defs>
        <linearGradient id="sweep" x1="100" y1="75" x2="158" y2="40" gradientUnits="userSpaceOnUse">
          <stop stopColor="#5ea500" stopOpacity="0.45" />
          <stop offset="1" stopColor="#5ea500" stopOpacity="0" />
        </linearGradient>
      </defs>
      <circle cx="100" cy="75" r="60" fill="#f4f7f0" />
      <circle cx="100" cy="75" r="60" stroke="#dfe8d1" />
      <circle cx="100" cy="75" r="40" stroke="#dfe8d1" />
      <circle cx="100" cy="75" r="20" stroke="#dfe8d1" />
      <path d="M100 75V15A60 60 0 0 1 152 45Z" fill="url(#sweep)" />
      <line x1="40" y1="75" x2="160" y2="75" stroke="#e7eedb" />
      <line x1="100" y1="15" x2="100" y2="135" stroke="#e7eedb" />
      <circle cx="100" cy="75" r="4.5" fill="#3f7000" />
      <circle cx="134" cy="49" r="9" fill="#5ea500" fillOpacity="0.16" />
      <circle cx="134" cy="49" r="4" fill="#5ea500" />
      <circle cx="70" cy="104" r="3" fill="#b3b3ad" />
      <circle cx="124" cy="100" r="3" fill="#b3b3ad" />
    </svg>
  )
}

/** "No insolvency proceedings" - a reassuring shield with a check. */
export function NoProceedingsArt({ className }: ArtProps) {
  return (
    <svg viewBox="0 0 200 150" fill="none" className={className} aria-hidden role="img">
      <ellipse cx="100" cy="132" rx="56" ry="8" fill="#eef3e6" />
      <rect
        x="54"
        y="34"
        width="50"
        height="66"
        rx="6"
        fill="#ffffff"
        stroke="#e6e6e0"
        transform="rotate(-9 79 67)"
      />
      <path d="M68 52h22M66 62h26M67 72h18" stroke="#deded7" strokeWidth="3" strokeLinecap="round" />
      <path
        d="M104 30c12.5 5.2 22.5 7 30 7v25c0 23-13.5 35.8-30 41.5C87.5 97.8 74 85 74 62V37c7.5 0 17.5-1.8 30-7Z"
        fill="#e7f0d8"
        stroke="#5ea500"
        strokeWidth="2.5"
        strokeLinejoin="round"
      />
      <path
        d="M90 60.5 99 69l16-17"
        stroke="#3f7000"
        strokeWidth="3.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

/** "Pick a subject from the list" - a ledger with a highlighted row. */
export function PickSubjectArt({ className }: ArtProps) {
  return (
    <svg viewBox="0 0 200 150" fill="none" className={className} aria-hidden role="img">
      <rect x="38" y="30" width="124" height="90" rx="10" fill="#ffffff" stroke="#e6e6e0" />
      <rect x="38" y="30" width="124" height="22" rx="10" fill="#f6f6f4" />
      <rect x="38" y="46" width="124" height="6" fill="#f6f6f4" />
      <circle cx="52" cy="41" r="3.5" fill="#cfe3ad" />
      <rect x="62" y="38.5" width="40" height="5" rx="2.5" fill="#e2e2db" />
      <rect x="48" y="62" width="104" height="14" rx="5" fill="#f1f1ed" />
      <rect x="60" y="67" width="56" height="4" rx="2" fill="#dcdcd5" />
      <rect x="46" y="82" width="108" height="18" rx="6" fill="#f4f7f0" stroke="#cfe3ad" />
      <circle cx="58" cy="91" r="4" fill="#5ea500" />
      <rect x="70" y="86" width="50" height="4.5" rx="2.25" fill="#88b94d" />
      <rect x="70" y="93" width="34" height="3.5" rx="1.75" fill="#bcd79a" />
      <path
        d="M150 96l4 18 5-7 8 1z"
        fill="#0a0a0a"
        stroke="#ffffff"
        strokeWidth="2"
        strokeLinejoin="round"
      />
    </svg>
  )
}

/** "No changes yet, you're being watched" - a bell at rest with a green check. */
export function NoChangesArt({ className }: ArtProps) {
  return (
    <svg viewBox="0 0 200 150" fill="none" className={className} aria-hidden role="img">
      <ellipse cx="98" cy="128" rx="46" ry="7" fill="#eef3e6" />
      <circle cx="96" cy="66" r="50" fill="#f4f7f0" />
      <g
        transform="translate(52 20) scale(3.7)"
        fill="none"
        stroke="#b4b4ad"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9" />
        <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0" />
      </g>
      <circle cx="138" cy="98" r="15" fill="#5ea500" />
      <path
        d="m131 98 4.2 4.4 8.4-9"
        stroke="#ffffff"
        strokeWidth="2.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}
