---
uuid: 5a416920-3f3e-4075-83cf-5db20ae55800
name: codexis-personalistika
icon: icon.svg
version: 1.0.0
jurisdictions: [CZ, EU]
description: Use when a Czech legal question concerns employment, HR, hiring, contracts, termination, redundancy, working time, leave, wages, payroll, health or social insurance, discrimination, employee representation, remote work, obstacles to work or personnel records. Standalone employment-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "Monitor - Personalistika"
    summary: "České pracovní právo, mzdy, odvody a personální agenda."
    examplePrompts:
      - "Prověř výpověď pro nadbytečnost a rizika při výběru konkrétního zaměstnance."
      - "Musí invalidní zaměstnanec při neplaceném volnu doplatit zdravotní pojištění do minima?"
      - "Navrhni správný postup při změně pracovní doby a místa výkonu práce."
  en:
    displayName: "Monitor - HR"
    summary: "Czech employment law, payroll, insurance and HR administration."
    examplePrompts:
      - "Review a redundancy dismissal and the risks in selecting a particular employee."
      - "Must an employee with a disability top up health insurance during unpaid leave?"
      - "Propose a compliant process for changing working time and place of work."
  sk:
    displayName: "Monitor - Personalistika ČR"
    summary: "České pracovné právo, mzdy, odvody a personálna agenda."
    examplePrompts:
      - "Prever výpoveď pre nadbytočnosť a riziká pri výbere konkrétneho zamestnanca."
      - "Musí invalidný zamestnanec pri neplatenom voľne doplatiť zdravotné poistenie do minima?"
      - "Navrhni správny postup pri zmene pracovného času a miesta výkonu práce."
---

# CODEXIS — Personalistika

Standalone skill pro české pracovní právo, personální agendu, mzdy a pojistné.

## Operating Assumptions

- Právní zdroje získávej přes `cdx-cli`.
- U známého zákona začni `/versions`, u konkrétního ustanovení pokračuj přes `/toc` a `/text?part=`.
- Částky, limity, redukční hranice a minimální základy vždy ověř k rozhodnému období.
- Materiály MPSV, ČSSZ a zdravotních pojišťoven označ jako provozní výklad.

## Research Strategy

1. Identifikuj strany, skutečný typ práce, pracovněprávní vztah, roli uživatele a rozhodná data.
2. Rozděl dotaz na pracovní právo, mzdu, daň, sociální pojištění a zdravotní pojištění.
3. Pro každou vrstvu otevři zvláštní předpis a jeho prováděcí úpravu.
4. Ověř vznik povinnosti, výjimky, proces, dokument, lhůtu a následek vady.
5. U skončení, diskriminace, úrazu nebo sporu doplň judikaturu a důkazní otázky.

## Klíčové předpisy

| Předpis | CODEXIS base | Použití |
|---|---|---|
| zákon č. 262/2006 Sb., zákoník práce | `cz_law/262/2006` | pracovní poměr, DPP/DPČ, mzda, doba, dovolená, skončení |
| zákon č. 435/2004 Sb. | `cz_law/435/2004` | zaměstnanost a cizinci |
| zákon č. 187/2006 Sb. | `cz_law/187/2006` | nemocenské pojištění |
| zákon č. 589/1992 Sb. | `cz_law/589/1992` | pojistné na sociální zabezpečení |
| zákon č. 592/1992 Sb. | `cz_law/592/1992` | pojistné na veřejné zdravotní pojištění |
| zákon č. 48/1997 Sb. | `cz_law/48/1997` | veřejné zdravotní pojištění a státní pojištěnci |
| zákon č. 586/1992 Sb. | `cz_law/586/1992` | zdanění mzdy a benefitů |
| zákon č. 373/2011 Sb. | `cz_law/373/2011` | pracovnělékařské služby |

## Hard Rules

- Pracovněprávní, daňový, sociální a zdravotněpojistný výsledek odvozuj samostatně.
- „Invalidní“ není totéž jako „příjemce invalidního důchodu“; ověř právně rozhodný status.
- U zdravotního pojištění nejdřív prověř výjimky z minimálního vyměřovacího základu.
- U skončení pracovního poměru rozliš důvod, formu, skutkové vymezení, doručení, ochrannou dobu a nároky.
- U benefitů ověř současně daň zaměstnance, náklad zaměstnavatele a pojistné.
- Neuvedený limit nebo částku nedoplňuj z paměti.

## Workflow personalisty

1. **Vztah.** Pracovní poměr, DPP, DPČ, agenturní práce, statutární funkce nebo samostatná činnost.
2. **Čas.** Vznik, změna, rozhodné období, doručení, konec a navazující lhůty.
3. **Pracovní právo.** Povinnosti, forma dokumentu, projednání a následky vady.
4. **Mzda a náhrady.** Základ, příplatky, náhrady, překážky a dovolená.
5. **Daň a pojistné.** Každý režim posuď podle vlastního zákona a výjimek.
6. **Evidence.** Osobní spis, mzdové podklady, hlášení a retenční povinnosti.
7. **Praktický výstup.** Odděl kroky zaměstnavatele, zaměstnance a příslušné instituce.

## Zdravotní pojištění a invalidní důchod

Při dotazu na minimum zdravotního pojištění načti `references/invalidni-duchod-a-minimum-zdravotniho-pojisteni.md`.

## Časté pasti

- Závěr podle výše mzdy bez ověření statusu státního pojištěnce.
- Směšování invalidity s pobíráním invalidního důchodu.
- Zaměnění účetního, daňového a pojistného zachycení benefitu.
- Výpověď bez přesného skutkového důvodu nebo bez kontroly ochranné doby.
- Odpověď jen zaměstnavateli bez kroků a nároků zaměstnance.
- Použití aktuálního limitu na starší mzdové období.

## Struktura odpovědi

1. Přímý závěr.
2. Typ vztahu, role a rozhodné období.
3. Pracovněprávní pravidlo a dokumenty.
4. Mzdový, daňový, sociální a zdravotní dopad odděleně.
5. Lhůty, evidence a praktické kroky každé strany.
6. Předpoklady, které mohou závěr změnit.

## Related references

- Invalidní důchod a minimum zdravotního pojištění: `references/invalidni-duchod-a-minimum-zdravotniho-pojisteni.md`.
