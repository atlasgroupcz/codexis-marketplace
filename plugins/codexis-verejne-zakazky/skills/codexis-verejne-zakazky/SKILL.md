---
uuid: 6a3301ef-581b-45ce-af55-50234bffa5f8
name: codexis-verejne-zakazky
icon: icon.png
version: 1.0.0
jurisdictions: [CZ, EU]
description: Use when a Czech or EU legal question concerns public procurement, contracting authorities, estimated value, procurement procedures, tender conditions, qualification, bids, clarification, evaluation, deadlines, site visits, objections, review, contract award or modification under ZZVZ. Standalone procurement methodology using CODEXIS.
i18n:
  cs:
    displayName: "Monitor - Veřejné zakázky"
    summary: "České a evropské právo veřejných zakázek, zadávací postupy a změny smluv."
    examplePrompts:
      - "Lze po lhůtě doplnit kvalifikaci nebo novou referenční zakázku?"
      - "Prověř lhůtu pro podání nabídek a termín prohlídky místa plnění."
      - "Posuď, zda lze po zadání změnit cenu nebo rozsah smlouvy."
  en:
    displayName: "Monitor - Public Procurement"
    summary: "Czech and EU public procurement law, tender procedures and contract changes."
    examplePrompts:
      - "Can qualification or a new reference contract be supplemented after the bid deadline?"
      - "Review the bid deadline and scheduled site visit."
      - "Assess whether the price or scope of an awarded contract may be changed."
  sk:
    displayName: "Monitor - Verejné obstarávanie"
    summary: "České a európske právo verejného obstarávania, postupy a zmeny zmlúv."
    examplePrompts:
      - "Možno po lehote doplniť kvalifikáciu alebo novú referenčnú zákazku?"
      - "Prever lehotu na predkladanie ponúk a termín obhliadky miesta plnenia."
      - "Posúď, či možno po zadaní zmeniť cenu alebo rozsah zmluvy."
---

# CODEXIS - Veřejné zakázky

Standalone skill pro české a evropské právo veřejných zakázek.

## Operating Assumptions

- Pro CODEXIS používej `cdx-cli search`, `cdx-cli get` a `cdx-cli schema`.
- Známý předpis otevři přes `/versions`; konkrétní paragraf přes `/toc` a `/text?part=`.
- ÚOHS, VVZ, TED a metodiky používej jako doplňkové oficiální zdroje.
- Nehádej procesní fázi, druh řízení, limit ani účinek uveřejnění.

## Research Strategy

1. Urči zadavatele, předmět, předpokládanou hodnotu, režim, druh řízení, fázi a rozhodná data.
2. Označ každý závěr jako povinnost, možnost, vyhrazenou volbu zadavatele, doporučení nebo smluvní požadavek.
3. Sestav normativní stack: zásady, obecné pravidlo, zvláštní pravidlo fáze, výjimka, uveřejnění a přezkum.
4. Ověř aktuální nebo historické znění ZZVZ a odpovídající unijní úpravy.
5. U judikatury porovnej skutkový stav, fázi řízení, výrok a pozdější vývoj.

## Data Sources

| Code | Použití |
|---|---|
| `CR` | ZZVZ, správní a procesní předpisy |
| `EU` | zadávací směrnice a přímo použitelné předpisy |
| `JD` | NSS, správní soudy, ÚS a SDEU |
| `COMMENT` | systematický výklad ZZVZ |
| `LT` | odborné rozbory praxe a rozhodovací činnosti |

## Klíčové předpisy

| Předpis | CODEXIS base | Použití |
|---|---|---|
| zákon č. 134/2016 Sb., ZZVZ | `cz_law/134/2016` | celý zadávací proces |
| zákon č. 500/2004 Sb. | `cz_law/500/2004` | řízení před ÚOHS |
| zákon č. 250/2016 Sb. | `cz_law/250/2016` | odpovědnost za přestupky |
| zákon č. 99/1963 Sb. | `cz_law/99/1963` | civilní spory ze smluv |
| směrnice 2014/24/EU | `EU` | klasické veřejné zakázky |
| směrnice 2014/25/EU | `EU` | sektorové zakázky |
| směrnice 2014/23/EU | `EU` | koncese |

## Hard Rules

- Hodnotový režim, volba druhu řízení a pravidla lhůt jsou tři oddělené otázky.
- Kvalifikaci, obsah nabídky a hodnoticí kritérium neposuzuj stejným testem.
- Neoznač doporučenou úpravu zadávací dokumentace jako povinnou bez zákonné nebo vnitřně dokumentové opory.
- U prohlídky místa plnění vždy otevři § 97 ZZVZ.
- U doplnění kvalifikace čti § 46 a § 79 společně s přesnou zadávací podmínkou a judikaturou.
- U změny smlouvy nestačí procentní limit; prověř povahu změny, kumulaci a dopad na soutěž.
- Při kontrole dokumentace čti všechny přílohy jako jeden celek.

## Workflow specialisty na veřejné zakázky

1. **Scope.** Urči veřejného, sektorového či dotovaného zadavatele a zda jde o zadávání nebo plnění smlouvy.
2. **Hodnota.** Prověř funkční celek, pravidelná plnění, části a zákaz účelového dělení.
3. **Režim a postup.** Odděl zákonný režim od zadavatelem zvoleného řízení.
4. **Fáze.** Příprava, lhůta, podání, objasnění, hodnocení, výběr, námitky, ÚOHS nebo změna smlouvy.
5. **Speciální pravidlo.** Vyhledej ustanovení přesně pro danou fázi; obecné zásady jsou až kontrolní vrstva.
6. **Dokumentace.** Porovnej formuláře, krycí list, návrh smlouvy, přílohy a uveřejněné údaje.
7. **Remedy.** Uveď dokumentaci rozhodnutí, námitky, přezkum a konkrétní procesní riziko.

## Feedback-derived deep dive

Pro kvalifikaci, lhůty, prohlídku místa a review dokumentace načti `references/kvalifikace-lhuty-a-kontrola-dokumentace.md`.

## Změny smlouvy

- Identifikuj původní závazek, vyhrazenou změnu a všechny dřívější změny.
- Testuj obecné pravidlo podstatné změny a každou výjimku samostatně.
- Odděl předem vyhrazenou indexaci od pozdější nevyhrazené změny.
- Uveď výpočet kumulace, kvalitativní test a povinnost uveřejnění.

## Časté pasti

- Přehlédnutí § 97 při prohlídce místa plnění.
- Automatický zákaz nebo automatické připuštění nové reference po lhůtě.
- Směšování uveřejnění opravného formuláře s následnou změnou údaje na profilu.
- Požadavek na duplicitní přílohu, přestože údaje už dokumentace propojuje.
- Označení defensivní drafting preference jako porušení zákona.
- Generalizace judikátu z jiné fáze nebo jiného režimu.

## Struktura odpovědi

1. Přímý závěr s označením povinné, možné nebo doporučené.
2. Zadavatel, režim, druh řízení a fáze.
3. Rozhodné znění a úplný zákonný test.
4. Aplikace na fakta a přesný výpočet lhůty nebo hodnoty.
5. Vhodnost judikatury a případné odlišení.
6. Dokumentace, uveřejnění, námitky a riziko.

## Related references

- Kvalifikace, lhůty a kontrola dokumentace: `references/kvalifikace-lhuty-a-kontrola-dokumentace.md`.
