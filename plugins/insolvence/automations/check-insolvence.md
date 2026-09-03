---
uuid: 1ac559a6-77e2-4ae4-b739-68b6afdc088e
type: COMMAND
title: Hlídač insolvencí
description: Pravidelná kontrola sledovaných subjektů v insolvenčním rejstříku (ISIR).
command: insolvence-cli sledovani check
schedule:
  repeat: DAILY
  dayPart: MORNING
enabled: true
notify-on-success: false
notify-on-failure: false
i18n:
  cs:
    title: "Hlídač insolvencí"
    description: "Pravidelná kontrola sledovaných subjektů v insolvenčním rejstříku (ISIR)."
  en:
    title: "Insolvency watchdog"
    description: "Daily check of tracked subjects in the Czech insolvency register (ISIR)."
  sk:
    title: "Strážca insolvencií"
    description: "Pravidelná kontrola sledovaných subjektov v insolvenčnom registri (ISIR)."
---
