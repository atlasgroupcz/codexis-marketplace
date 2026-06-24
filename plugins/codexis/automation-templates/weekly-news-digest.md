---
uuid: 2cf8430c-283d-49cb-b879-677406a03be0
type: AGENT
title: Weekly news digest on a topic
description: Every week, summarize the latest news and developments on a topic of your choice.
cron: 0 7 * * 1
max-turns: 20
enabled: false
notify-on-success: true
notify-on-failure: true
web-search: true
i18n:
  cs:
    title: "Týdenní přehled novinek k tématu"
    description: "Každý týden shrne nejnovější zprávy a vývoj k tématu dle vašeho výběru."
  en:
    title: "Weekly news digest on a topic"
    description: "Every week, summarize the latest news and developments on a topic of your choice."
  sk:
    title: "Týždenný prehľad noviniek k téme"
    description: "Každý týždeň zhrnie najnovšie správy a vývoj k téme podľa vášho výberu."
---
Sleduji za vás vývoj k tématu: «UPRAVTE — např. konkrétní obor, trh, konkurent, technologie nebo region».

Projdi za uplynulý týden nový vývoj k tomuto tématu z důvěryhodných veřejných zdrojů a sestav stručný přehled:

1. **Hlavní novinky** — 3–5 nejdůležitějších událostí nebo zpráv, každá v jedné až dvou větách.
2. **Souvislosti** — proč jsou podstatné a koho se týkají.
3. **Co sledovat dál** — nadcházející události nebo očekávaný vývoj, pokud jsou relevantní.

Pravidla:
- U každé novinky uveď zdroj (název a odkaz), aby šla ověřit.
- Upřednostni věcné a původní zdroje před spekulacemi; rozlišuj fakta a názory.
- Pokud se za dané období nic podstatného nestalo, napiš to jednou větou a negeneruj balast.
- Piš česky, věcně, bez vaty. Cílem je přehled k rychlému přečtení.

_Tento přehled vychází z veřejně dostupných zdrojů a má informativní charakter._
