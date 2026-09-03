---
uuid: 287ca8f3-7329-48e9-b948-3d2249d8b5a7
type: COMMAND
title: Hlídač SID dokumentů
description: Denní kontrola nových notifikací a čekajících potvrzení v SID.
command: sid-cli check --notify
schedule:
  repeat: DAILY
  dayPart: MORNING
enabled: true
notify-on-success: false
notify-on-failure: false
i18n:
  cs:
    title: "Hlídač SID dokumentů"
    description: "Denní kontrola nových notifikací a čekajících potvrzení v SID."
  en:
    title: "SID documents watchdog"
    description: "Daily check of new SID notifications and pending read-confirmations."
  sk:
    title: "Strážca SID dokumentov"
    description: "Denná kontrola nových notifikácií a čakajúcich potvrdení v SID."
---
