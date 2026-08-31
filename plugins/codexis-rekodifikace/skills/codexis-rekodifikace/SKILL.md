---
uuid: 6caec9ec-a2ec-4e10-bb21-acf94cc7df54
name: codexis-rekodifikace
icon: icon.svg
version: 1.0.0
jurisdictions: [CZ]
description: Use when a Czech private-law question turns on recodification, transitional provisions, old versus new Civil Code, legal relationships created before a reform, continuing effects, later legal facts, limitation periods, transformed legal persons or retroactivity. Standalone temporal-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "Monitor - Rekodifikace"
    summary: "Přechodná ustanovení a časové účinky rekodifikace českého soukromého práva."
    examplePrompts:
      - "Které právo se použije na smlouvu uzavřenou před rekodifikací a její pozdější porušení?"
      - "Posuď promlčení nároku vzniklého před změnou občanského zákoníku."
      - "Jak se rekodifikace dotkla právní formy a jednání starší právnické osoby?"
  en:
    displayName: "Monitor - Recodification"
    summary: "Transitional provisions and temporal effects of Czech private-law recodification."
    examplePrompts:
      - "Which law governs a contract made before recodification and a later breach?"
      - "Assess limitation of a claim created before the Civil Code changed."
      - "How did recodification affect an older legal person's form and representation?"
  sk:
    displayName: "Monitor - Rekodifikácia"
    summary: "Prechodné ustanovenia a časové účinky rekodifikácie českého súkromného práva."
    examplePrompts:
      - "Ktoré právo sa použije na zmluvu uzavretú pred rekodifikáciou a jej neskoršie porušenie?"
      - "Posúď premlčanie nároku vzniknutého pred zmenou občianskeho zákonníka."
      - "Ako rekodifikácia ovplyvnila právnu formu a konanie staršej právnickej osoby?"
---

# CODEXIS — Rekodifikace

Standalone skill pro časové střety starého a nového českého soukromého práva.

## Operating Assumptions

- Pro každou starou a novou normu používej `cdx-cli get .../versions`.
- Neporovnávej jen současný a poslední starý text; načti i přechodná a doprovodná ustanovení.
- Starý a nový právní pojem nepovažuj za totožný jen kvůli podobnému názvu.
- U judikatury ověř, který časový režim a který konkrétní přechod řešila.

## Research Strategy

1. Vytvoř časovou osu vzniku vztahu, nabytí práva, pokračujících účinků, porušení, uplatnění nároku, řízení a rozhodnutí.
2. Každou událost klasifikuj samostatně.
3. Identifikuj starou normu, novou normu, obecné přechodné ustanovení a zvláštní přechodné pravidlo.
4. Použij zvláštní přechodné pravidlo před obecným.
5. Odděl hmotné právo, právní účinky, promlčení, proces, důkazy a rejstříkové následky.

## Klíčové předpisy

| Předpis | CODEXIS base | Použití |
|---|---|---|
| zákon č. 89/2012 Sb. | `cz_law/89/2012` | občanský zákoník a jeho přechody |
| zákon č. 90/2012 Sb. | `cz_law/90/2012` | obchodní korporace |
| zákon č. 292/2013 Sb. | `cz_law/292/2013` | zvláštní řízení soudní |
| zákon č. 304/2013 Sb. | `cz_law/304/2013` | veřejné rejstříky |
| zákon č. 303/2013 Sb. | `cz_law/303/2013` | doprovodné změny rekodifikace |
| zákon č. 40/1964 Sb. | `cz_law/40/1964` | dřívější občanský zákoník |
| zákon č. 513/1991 Sb. | `cz_law/513/1991` | dřívější obchodní zákoník |

## Hard Rules

- Jediný právní vztah může mít vznik, obsah, porušení i proces podle různých časových pravidel.
- § 3028 až 3079 občanského zákoníku necituj jako blok; najdi konkrétní použitelné ustanovení.
- Pravou a nepravou retroaktivitu rozliš až po identifikaci zákonného přechodu.
- Promlčení a prekluzi řeš odděleně od vzniku a obsahu práva.
- Terminologickou náhradu neprezentuj jako věcnou kontinuitu bez srovnání skutkových podstat.
- Při absenci doménového feedbacku nevytvářej kategorický výklad bez dohledaného přechodného pravidla a judikatury.

## Workflow temporal analysis

1. **Events.** Každá právně relevantní událost s přesným datem.
2. **Old law.** Starý institut, jeho podmínky a účinky.
3. **New law.** Nový institut a rozdíl proti starému.
4. **Transition.** Zvláštní a obecné přechodné ustanovení.
5. **Constitutional check.** Nabytá práva, legitimní očekávání a zákaz pravé retroaktivity.
6. **Procedure.** Zahájení řízení, příslušnost, dokazování a opravné prostředky.
7. **Case law.** Rozhodnutí k přesnému přechodnému ustanovení.

## Časté pasti

- Jedno datum a jeden zákon pro celý dlouhodobý vztah.
- Použití § 3028 jako univerzální odpovědi.
- Směšování staré a nové terminologie bez věcného srovnání.
- Automatický restart promlčecí doby k 1. 1. 2014.
- Použití dnešní judikatury bez ověření, že vykládá rozhodné staré právo.
- Přehlédnutí zvláštního přechodu v jiném zákoně.

## Struktura odpovědi

1. Dated timeline.
2. Právní kvalifikace každé události.
3. Staré a nové pravidlo.
4. Konkrétní přechodné ustanovení.
5. Hmotněprávní, promlčecí, procesní a rejstříkový následek odděleně.
6. Judikatura, ústavní limit a zbývající nejistota.
