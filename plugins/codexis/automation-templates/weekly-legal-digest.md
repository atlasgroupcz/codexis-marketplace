---
uuid: ecf1ce9e-4d07-4dac-a212-bf0c80305403
type: AGENT
title: Weekly legal digest on a topic
description: Every week, summarize new legislation, case law and commentary on a topic of your choice.
skill-full-names:
  - codexis:codexis
cron: 0 7 * * 1
max-turns: 20
enabled: false
notify-on-success: true
notify-on-failure: true
i18n:
  cs:
    title: "Týdenní právní rešerše k tématu"
    description: "Každý týden shrne novou legislativu, judikaturu a komentáře k tématu dle vašeho výběru."
  en:
    title: "Weekly legal digest on a topic"
    description: "Every week, summarize new legislation, case law and commentary on a topic of your choice."
  sk:
    title: "Týždenná právna rešerš k téme"
    description: "Každý týždeň zhrnie novú legislatívu, judikatúru a komentáre k téme podľa vášho výberu."
---
Sleduji za vás vývoj k tématu: «UPRAVTE — např. veřejné zakázky, GDPR, pracovní právo».

Pomocí dovednosti CODEXIS projdi za uplynulý týden nový vývoj k tomuto tématu a sestav stručný přehled:

1. **Legislativa** — nové nebo novelizované předpisy, účinnost změn, klíčový dopad.
2. **Judikatura** — nová relevantní rozhodnutí soudů (spisová značka, soud, datum, právní věta v jedné větě).
3. **Komentáře a odborná literatura** — nové výklady nebo stanoviska, pokud jsou relevantní.

Pravidla:
- U každého bodu uveď konkrétní zdroj (číslo předpisu / spisovou značku) tak, aby šel dohledat v CODEXIS.
- Pokud se za dané období nic podstatného nestalo, napiš to jednou větou a negeneruj balast.
- Piš česky, věcně, bez vaty. Cílem je report k rychlému přečtení, ne esej.
