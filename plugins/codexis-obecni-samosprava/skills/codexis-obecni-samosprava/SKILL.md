---
uuid: a892e30a-aa51-4e58-8b13-979d9bd897c1
name: codexis-obecni-samosprava
icon: icon.webp
version: 1.0.0
jurisdictions: [CZ]
description: Use when a Czech legal question concerns municipalities, municipal councils, boards, mayors, municipal offices, independent or delegated powers, municipal property, local ordinances, elections, access to information, contributory organizations or municipal administrative procedure. Standalone municipal-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "Monitor - Obecní samospráva"
    summary: "Právo obcí, jejich orgánů, majetku a výkonu samostatné i přenesené působnosti."
    examplePrompts:
      - "Který orgán obce musí schválit převod obecního pozemku a jaké kroky tomu předcházejí?"
      - "Prověř platnost usnesení rady obce a navazující smlouvy."
      - "Jaké povinnosti má obec a registrační úřad před komunálními volbami?"
  en:
    displayName: "Monitor - Municipal Government"
    summary: "Czech municipal bodies, property and independent or delegated government powers."
    examplePrompts:
      - "Which municipal body must approve the transfer of municipal land and what steps come first?"
      - "Review the validity of a municipal board resolution and the resulting contract."
      - "What duties apply to a municipality and registration office before local elections?"
  sk:
    displayName: "Monitor - Obecná samospráva"
    summary: "České právo obcí, ich orgánov, majetku a samostatnej aj prenesenej pôsobnosti."
    examplePrompts:
      - "Ktorý orgán obce musí schváliť prevod obecného pozemku a aké kroky tomu predchádzajú?"
      - "Prever platnosť uznesenia rady obce a nadväzujúcej zmluvy."
      - "Aké povinnosti má obec a registračný úrad pred komunálnymi voľbami?"
---

# CODEXIS — Obecní samospráva

Standalone skill pro české obecní právo. Řeší kompetence orgánů obce, samostatnou a přenesenou působnost, majetek, místní normotvorbu, volby a správní postupy.

## Operating Assumptions

- Pro právní zdroje používej `cdx-cli`; je nainstalovaný a přihlášený.
- Známý předpis otevírej přes `cdx-cli get cdx://cz_law/<num>/<year>/versions`.
- Konkrétní znění čti přes `cdx-cli get 'cdx://doc/<versionId>/text?part=<elementId>'`.
- Nehádej `docId`, datum účinnosti ani příslušný orgán.
- Oficiální metodiky MV, volební formuláře a informace úřadů používej jako provozní doplněk, ne jako náhradu zákona.

## Research Strategy

1. Urči datum, roli uživatele, obec, konkrétní orgán a požadovaný úkon.
2. Rozliš samostatnou působnost, přenesenou působnost a soukromoprávní jednání obce.
3. Sestav kompetenční řetězec: kdo připravuje, rozhoduje, podepisuje, zveřejňuje, doručuje a přezkoumává.
4. Otevři znění každého předpisu účinné k rozhodnému datu a projdi související i přechodná ustanovení.
5. Teprve potom posuzuj platnost úkonu, procesní následek a praktický další krok.

## Data Sources

| Code | Použití |
|---|---|
| `CR` | zákony, vyhlášky a nařízení |
| `JD` | kompetence, neplatnost, volby, informace a správní přezkum |
| `COMMENT` | výklad vztahu obecního, správního a soukromého práva |
| `LT` | odborné rozbory a metodický kontext |
| `ALL` | pouze orientační hledání zdroje |

## Klíčové předpisy

| Předpis | CODEXIS base | Použití |
|---|---|---|
| zákon č. 128/2000 Sb., o obcích | `cz_law/128/2000` | orgány, působnost, majetek, vyhlášky |
| zákon č. 491/2001 Sb. | `cz_law/491/2001` | volby do zastupitelstev obcí |
| zákon č. 500/2004 Sb. | `cz_law/500/2004` | správní postup obecního úřadu |
| zákon č. 106/1999 Sb. | `cz_law/106/1999` | svobodný přístup k informacím |
| zákon č. 565/1990 Sb. | `cz_law/565/1990` | místní poplatky |
| zákon č. 250/2000 Sb. | `cz_law/250/2000` | rozpočtová pravidla územních rozpočtů |
| zákon č. 89/2012 Sb. | `cz_law/89/2012` | smlouvy, vlastnictví a právní jednání obce |
| zákon č. 134/2016 Sb. | `cz_law/134/2016` | veřejné zakázky obce |

## Hard Rules

- Obec, zastupitelstvo, rada, starosta a obecní úřad nejsou zaměnitelné subjekty.
- U historického dotazu nejprve resolve verzi k požadovanému datu. Aktuální znění nesmí sloužit jako důkaz pro dřívější povinnost.
- Rozhodnutí orgánu, podpis smlouvy, zveřejnění záměru a převod vlastnického práva jsou samostatné kroky.
- U lhůty uveď počátek, pravidlo počítání, konec a případný posun; nepřebírej pouze výsledek kalkulátoru.
- U formuláře ověř, kdo jej podává, komu, v jaké lhůtě a podle kterého znění volební úpravy.
- Doporučený governance postup nikdy neoznačuj jako zákonnou povinnost bez konkrétní opory.

## Workflow obecního právníka

1. **Role a působnost.** Zapiš, zda uživatel jedná za obec jako právnickou osobu, její orgán, správní orgán nebo registrační úřad.
2. **Kompetence.** Najdi vyhrazenou pravomoc a případnou možnost svěření nebo delegace.
3. **Proces.** Ověř přípravu, zveřejnění, usnášeníschopnost, hlasování, zápis, podpis, doručení a přezkum.
4. **Hmotné právo.** U smluv a majetku přidej občanský zákoník; u poplatků daňový řád; u zakázek ZZVZ.
5. **Časový řez.** Všechny citované paragrafy musí patřit ke stejnému rozhodnému období.
6. **Následky vad.** Odděl interní odpovědnost, veřejnoprávní přezkum a soukromoprávní platnost.
7. **Výstup podle role.** Jiný další krok dostane obec, úředník, občan, žadatel a kontrolní orgán.

## Majetek obce

- Identifikuj věc, právní titul a orgán s vyhrazenou kompetencí.
- Ověř záměr, dobu a způsob zveřejnění, přesné označení předmětu a zákonnou výjimku.
- Porovnej schválené podmínky s podepsanou smlouvou.
- Podpis a oprávnění jednat prověř odděleně od předchozího schválení.
- U nemovitosti odděl smlouvu od vkladu do katastru.

## Volby a historická znění

Při komunálních volbách načti `references/historicka-zneni-a-komunalni-volby.md`. Reference vychází z feedbacku, v němž byla povinnost podle znění účinného v roce 2026 chybně přenesena do období končícího 31. 12. 2025.

## Časté pasti

- Odpověď podle aktuálního znění na výslovně historický dotaz.
- Přenos povinnosti obce na registrační úřad nebo opačně.
- Závěr o platnosti smlouvy jen z usnesení orgánu.
- Směšování samostatné a přenesené působnosti.
- Nesprávný výpočet měsíční nebo procesní lhůty.
- Změna struktury uživatelova dokumentu, i když žádal zachovat názvy a pořadí.
- Odpověď žadateli, když uživatel jedná za obec nebo správní orgán.

## Struktura odpovědi

1. Přímý závěr a příslušný orgán.
2. Role uživatele a druh působnosti.
3. Rozhodné datum a použitá verze.
4. Kompetenční a procesní řetězec.
5. Povinné dokumenty, zveřejnění, doručení a lhůty.
6. Následky vady a praktický další krok.
7. Citace v pořadí zákon, prováděcí předpis, judikatura, metodika.

## Related references

- Historická znění a komunální volby: `references/historicka-zneni-a-komunalni-volby.md`.
