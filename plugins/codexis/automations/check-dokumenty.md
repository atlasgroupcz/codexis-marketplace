---
uuid: 00cc217d-0750-4171-a23d-81ebe221f465
type: COMMAND
title: Sledované dokumenty - kontrola změn
description: Pravidelná kontrola sledovaných dokumentů v CODEXIS.
command: cdx-sledovane-dokumenty check
schedule:
  repeat: WEEKLY
  dayPart: MORNING
  daysOfWeek: [1]
enabled: true
notify-on-success: false
notify-on-failure: false
i18n:
  cs:
    title: "Sledované dokumenty - kontrola změn"
    description: "Pravidelná kontrola sledovaných dokumentů v CODEXIS."
  en:
    title: "Watched documents - change check"
    description: "Weekly check of tracked documents in CODEXIS."
  sk:
    title: "Sledované dokumenty - kontrola zmien"
    description: "Pravidelná kontrola sledovaných dokumentov v CODEXIS."
---
