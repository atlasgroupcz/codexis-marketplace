---
uuid: b2400c7f-edab-4722-8222-dbf6722850b0
name: codexis-skolstvi
icon: icon.svg
version: 1.0.0
jurisdictions: [CZ, EU]
description: Use when a Czech legal question concerns schools, education, admissions, pupils, students, assessment, school directors, founders, school boards, special educational needs, educational records, administrative decisions or employment links in education. Standalone education-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "Monitor - Školství"
    summary: "České školské právo, rozhodování škol, práva žáků a role zřizovatelů."
    examplePrompts:
      - "Musí přijetí žáka do ZUŠ proběhnout ve správním řízení?"
      - "Jaký proces platí při odvolání ředitele školy z funkce?"
      - "Prověř pravidla hodnocení, přezkoumání výsledku a opravné zkoušky."
  en:
    displayName: "Monitor - Education"
    summary: "Czech education law, school decisions, student rights and founders' roles."
    examplePrompts:
      - "Must admission to a Czech art school follow administrative procedure?"
      - "What process applies when removing a school director from office?"
      - "Review the rules for assessment, review of results and resit examinations."
  sk:
    displayName: "Monitor - Školstvo"
    summary: "České školské právo, rozhodovanie škôl, práva žiakov a úlohy zriaďovateľov."
    examplePrompts:
      - "Musí prijatie žiaka do českej ZUŠ prebehnúť v správnom konaní?"
      - "Aký proces platí pri odvolaní riaditeľa školy z funkcie?"
      - "Prever pravidlá hodnotenia, preskúmania výsledku a opravnej skúšky."
---

# CODEXIS - Školství

Standalone skill pro české školské právo a související správní a pracovněprávní otázky.

## Operating Assumptions

- Právní zdroje získávej přes `cdx-cli`.
- U školského zákona a prováděcí vyhlášky vždy resolve verzi k rozhodnému školnímu roku a datu úkonu.
- Metodiky MŠMT používej po ověření, že odpovídají typu školy a aktuálnímu znění.
- Nezaměňuj školské rozhodnutí, interní pedagogické opatření a pracovněprávní úkon.

## Research Strategy

1. Urči druh školy, právní formu, zřizovatele, úroveň vzdělávání, dotčenou osobu a roli uživatele.
2. Rozliš školské, správní, pracovněprávní, antidiskriminační a ochranné vrstvy.
3. Sestav stack: školský zákon, druhová vyhláška, správní řád, úprava pedagogických pracovníků a zřizovatelská pravidla.
4. Ověř aktéra, pravomoc, formu, podklady, oznámení, opravný prostředek a soudní ochranu.
5. Navazující evidenci, financování a personální důsledky řeš odděleně.

## Klíčové předpisy

| Předpis | CODEXIS base | Použití |
|---|---|---|
| zákon č. 561/2004 Sb., školský zákon | `cz_law/561/2004` | vzdělávání, školy, rozhodování a zřizovatelé |
| zákon č. 563/2004 Sb. | `cz_law/563/2004` | pedagogičtí pracovníci |
| zákon č. 500/2004 Sb. | `cz_law/500/2004` | správní řízení |
| vyhláška č. 71/2005 Sb. | `cz_law/71/2005` | základní umělecké vzdělávání |
| vyhláška č. 48/2005 Sb. | `cz_law/48/2005` | základní vzdělávání |
| vyhláška č. 13/2005 Sb. | `cz_law/13/2005` | střední vzdělávání |
| vyhláška č. 27/2016 Sb. | `cz_law/27/2016` | podpůrná opatření |
| zákon č. 262/2006 Sb. | `cz_law/262/2006` | zaměstnanci školy a ředitel |

## Hard Rules

- Absence konkrétního typu školy v jednom výčtu sama neprokazuje absenci správní pravomoci.
- Čti obecnou pravomoc ředitele, zvláštní úpravu věci a pravidlo oznamování společně.
- Rozhodnutí vyhovující, rozhodnutí nevyhovující a zveřejnění seznamu mají odlišný procesní režim.
- U ředitele rozliš odvolání z funkce, odvolání proti rozhodnutí a skončení pracovního poměru.
- U žáka odděl školské opatření, správní rozhodnutí, ochranu osobních údajů a bezpečnostní postup.
- Odpověď musí odpovídat přesnému typu školy; pravidlo pro ZŠ nelze automaticky přenést na ZUŠ nebo SŠ.

## Workflow školského právníka

1. **Instituce.** Typ školy, zápis v rejstříku, právní forma a zřizovatel.
2. **Osoba a role.** Uchazeč, žák, zákonný zástupce, ředitel, učitel nebo zřizovatel.
3. **Úkon.** Přijetí, hodnocení, kázeň, podpůrné opatření, personální rozhodnutí nebo interní pravidlo.
4. **Pravomoc a režim.** Obecná i zvláštní ustanovení, správní řád a druhová vyhláška.
5. **Oznámení a obrana.** Zveřejnění, doručení, odůvodnění, poučení, odvolání a přezkum.
6. **Navazující dopad.** Evidence žáka, matrika, financování, pracovněprávní a ochranné důsledky.

## Přijetí do ZUŠ

Při přijetí nebo nepřijetí do základní umělecké školy načti `references/prijeti-do-zus.md`.

## Časté pasti

- Izolované čtení § 165 odst. 2 školského zákona.
- Kategorické vyloučení správního řízení jen proto, že ZUŠ není v jednom výčtu.
- Zveřejnění kladného výsledku bez individuálního rozhodnutí u nepřijatého uchazeče.
- Přenos pravidla z jiného druhu školy.
- Směšování pravomoci ředitele a pravomoci zřizovatele.
- Neúplná odpověď na navazující stručné „ano“, protože nebyl zachován kontext konverzace.

## Struktura odpovědi

1. Přímý závěr a typ školy.
2. Pravomoc a procesní režim.
3. Podklady, forma a rozhodná ustanovení.
4. Oznámení nebo doručení a délka zveřejnění.
5. Odůvodnění, poučení a opravný prostředek.
6. Navazující evidence a praktický postup školy.

## Related references

- Přijetí do ZUŠ: `references/prijeti-do-zus.md`.
