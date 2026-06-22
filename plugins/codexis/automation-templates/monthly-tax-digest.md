---
uuid: 69879057-cba0-4c62-b285-af413e48a64e
type: AGENT
title: Monthly Czech tax digest
description: Each month, summarize recent Czech tax-law changes and upcoming filing deadlines.
skill-full-names:
  - codexis:codexis-dane
cron: 0 7 1 * *
max-turns: 20
enabled: false
notify-on-success: true
notify-on-failure: true
i18n:
  cs:
    title: "Měsíční daňový přehled"
    description: "Každý měsíc shrne aktuální změny v české daňové legislativě a nadcházející termíny."
  en:
    title: "Monthly Czech tax digest"
    description: "Each month, summarize recent Czech tax-law changes and upcoming filing deadlines."
  sk:
    title: "Mesačný daňový prehľad"
    description: "Každý mesiac zhrnie aktuálne zmeny v českej daňovej legislatíve a nadchádzajúce termíny."
---
Pomocí dovednosti pro české daně sestav měsíční daňový přehled za uplynulý měsíc:

1. **Změny v legislativě** — novelizace daňových zákonů a vyhlášek (DPH, daň z příjmů, daňový řád, spotřební daně, daň z nemovitých věcí), s účinností a stručným dopadem pro plátce/poplatníka.
2. **Judikatura a výkladová stanoviska** — nová významná rozhodnutí nebo stanoviska finanční správy s praktickým dopadem.
3. **Nadcházející termíny** — daňové povinnosti splatné v následujícím měsíci (přiznání, zálohy, kontrolní hlášení) s konkrétními daty.

Pravidla:
- Zaměř se na obecně použitelné novinky; nepředpokládej konkrétní situaci jednoho poplatníka.
- U každého bodu uveď číslo předpisu / paragraf nebo spisovou značku dohledatelnou v CODEXIS.
- Pokud se za období nic podstatného nezměnilo, napiš to a uveď jen termíny.
- Piš česky, věcně a stručně.
