---
uuid: 2f9253f9-2a72-4e95-ac25-1699f5f940ed
name: codexis-bozp
icon: icon.png
version: 1.1.0
jurisdictions: [CZ, EU]
description: Use when a Czech legal question concerns occupational health and safety, prevention of workplace risks, work accidents, remote-work injuries, safety training, protective equipment, occupational medical services, work categories, shared workplaces, inspections or employer liability. Standalone BOZP methodology using CODEXIS.
i18n:
  cs:
    displayName: "Monitor - BOZP"
    summary: "České právo bezpečnosti a ochrany zdraví při práci, prevence rizik a pracovní úrazy."
    examplePrompts:
      - "Posuď, zda může být úraz při práci z domova pracovním úrazem a jaké podklady jsou rozhodné."
      - "Jaké povinnosti má zaměstnavatel při souběhu více firem na jednom pracovišti?"
      - "Prověř, zda naše evidence, školení a poskytování OOPP odpovídají aktuálním předpisům."
  en:
    displayName: "Monitor - OHS"
    summary: "Czech occupational health and safety law, risk prevention and workplace accidents."
    examplePrompts:
      - "Assess whether an injury while working from home can be a workplace accident and which facts matter."
      - "What duties apply when employees of several companies share one workplace?"
      - "Check whether our records, safety training and protective equipment comply with current Czech law."
  sk:
    displayName: "Monitor - BOZP ČR"
    summary: "České právo bezpečnosti a ochrany zdravia pri práci, prevencia rizík a pracovné úrazy."
    examplePrompts:
      - "Posúď, či môže byť úraz pri práci z domu pracovným úrazom a ktoré skutočnosti sú rozhodujúce."
      - "Aké povinnosti platia pri súbehu viacerých firiem na jednom pracovisku?"
      - "Over, či naša evidencia, školenia a poskytovanie OOPP zodpovedajú aktuálnym českým predpisom."
---

# CODEXIS — BOZP

Standalone skill pro českou bezpečnost a ochranu zdraví při práci.

## Operating Assumptions

- Pro právní zdroje používej `cdx-cli`.
- U známého předpisu otevři `/versions`, potom `/toc` a konkrétní `/text?part=`.
- Technickou normu, návod výrobce, kategorizaci práce a místní provozní pravidlo vždy označ podle jejich právní váhy.
- Hodnoty hygienických limitů a formuláře hlášení ověř v aktuální prováděcí úpravě.

## Research Strategy

1. Urči pracoviště, činnost, pracovní vztah, zařízení, nebezpečí, škodní událost a všechny aktéry.
2. Rozděl věc na prevenci, technickou bezpečnost, pracovnělékařské služby, úraz, inspekci a náhradu újmy.
3. Sestav stack zákoník práce, zákon č. 309/2006 Sb., prováděcí předpis, technické požadavky a návod výrobce.
4. U úrazu vytvoř časovou osu a důkazní matici.
5. U sporu doplň judikaturu k plnění pracovních úkolů, příčinné souvislosti a zproštění odpovědnosti.

## Klíčové předpisy

| Předpis | CODEXIS base | Použití |
|---|---|---|
| zákon č. 262/2006 Sb. | `cz_law/262/2006` | prevence, povinnosti, úrazy a náhrady |
| zákon č. 309/2006 Sb. | `cz_law/309/2006` | další požadavky BOZP |
| zákon č. 251/2005 Sb. | `cz_law/251/2005` | inspekce práce a přestupky |
| zákon č. 373/2011 Sb. | `cz_law/373/2011` | pracovnělékařské služby |
| vyhláška č. 79/2013 Sb. | `cz_law/79/2013` | pracovnělékařské prohlídky |
| nařízení vlády č. 361/2007 Sb. | `cz_law/361/2007` | podmínky ochrany zdraví při práci |
| nařízení vlády č. 101/2005 Sb. | `cz_law/101/2005` | pracoviště a pracovní prostředí |
| nařízení vlády č. 390/2021 Sb. | `cz_law/390/2021` | osobní ochranné pracovní prostředky |
| nařízení vlády č. 322/2025 Sb. | `cz_law/322/2025` | evidence a hlášení pracovních úrazů od 1. 1. 2026 |

## Hard Rules

- Nejdřív identifikuj konkrétní nebezpečí a teprve potom kontrolní opatření.
- Pracovní úraz neposuzuj jen podle místa; ověř plnění pracovních úkolů, přímou souvislost, příčinu a výjimky.
- Školení, OOPP, zdravotní způsobilost a technický stav zařízení jsou samostatné povinnosti.
- Interní směrnice nemůže snížit zákonný standard.
- U sdíleného pracoviště určuj povinnosti každého zaměstnavatele a koordinaci zvlášť.
- U home office popiš, co zaměstnavatel reálně může kontrolovat a dokumentovat.
- Feedback přiřazený monitoru, ale tematicky mimo BOZP, nepoužívej jako doménové pravidlo.

## Workflow BOZP specialisty

1. **Scope.** Zaměstnanec, dohodář, OSVČ, dodavatel, návštěva nebo jiná osoba.
2. **Hazard.** Stroj, chemická látka, práce ve výšce, doprava, ergonomie, násilí, klima nebo vzdálené pracoviště.
3. **Risk controls.** Eliminace, technické opatření, organizace práce, školení a OOPP v tomto pořadí.
4. **Competence.** Odborná způsobilost, oprávnění, zdravotní způsobilost a návod.
5. **Incident.** První pomoc, zajištění místa, evidence, hlášení, šetření a nápravná opatření.
6. **Liability.** Příčinná souvislost, rozsah újmy, možné zproštění a jednotlivé nároky.
7. **Output.** Odpovědná osoba, dokument, lhůta a kontrolovatelný další krok.

## Kategorizace a pracovnělékařské služby

Při dotazu na kategorii práce, měření rizikových faktorů, lékařskou prohlídku nebo pracovníka dodavatele načti `references/kategorizace-prace-a-pracovnelekarske-sluzby.md`. Reference vychází z pozitivně hodnocených chatů monitoru BOZP z období 1. 1.–31. 8. 2026.

- Kategorie se váže na skutečnou práci, pracoviště a expozici, ne jen na název pozice.
- Odděl rozhodnutí o kategorii, informování zaměstnance, zajištění prohlídky a předání rizik na sdíleném pracovišti.
- Periodicitu prohlídky ověř podle kategorie, profesního rizika, věku a zvláštního předpisu; nepoužij jediný parametr izolovaně.

## Pracovní úraz

- Zapiš činnost bezprostředně před úrazem, pokyn, místo, čas, svědky a mechanismus zranění.
- Odděl uznání pracovního úrazu od evidence, hlášení a výpočtu náhrady.
- U možného zproštění ověř poučení, kontrolu, příčinu a skutečný vliv porušení.
- Nezaměň pracovní úraz s úrazem na cestě do zaměstnání a zpět.

## Časté pasti

- Univerzální checklist bez vazby na konkrétní riziko.
- Závěr o úrazu jen podle toho, že nastal na pracovišti.
- OOPP jako první a jediné opatření.
- Školení bez ověření srozumitelnosti, konkrétnosti a dokumentace.
- Přehlédnutí jiného zaměstnavatele na sdíleném pracovišti.
- Uvedení zastaralého formuláře nebo hygienického limitu.

## Struktura odpovědi

1. Přímý závěr a klasifikace rizika nebo události.
2. Rozhodné osoby, pracoviště a činnost.
3. Povinnosti podle hierarchie kontrolních opatření.
4. Dokumentace, školení, zdravotní a technické požadavky.
5. Incidentní postup, hlášení, náhrady a odpovědnost.
6. Nevyjasněná fakta a další krok.

## Related references

- Kategorizace práce a pracovnělékařské služby: `references/kategorizace-prace-a-pracovnelekarske-sluzby.md`.
