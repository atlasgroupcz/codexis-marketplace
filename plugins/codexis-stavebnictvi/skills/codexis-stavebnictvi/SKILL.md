---
uuid: 18b20f5f-1ec5-40e2-959b-591a9a48e1ae
name: codexis-stavebnictvi
icon: icon.png
version: 1.0.0
jurisdictions: [CZ, EU]
description: Use when a Czech legal question concerns construction, building permits, project authorization, occupancy approval, additional permitting, zoning, building authorities, affected authorities, JES, project documentation, unauthorized structures, changes before completion or construction-law transitional provisions. Standalone construction-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "Monitor - Stavebnictví"
    summary: "České stavební právo, povolování, kolaudace, dokumentace a dotčené orgány."
    examplePrompts:
      - "Jak se kolauduje stavba povolená podle starého stavebního zákona?"
      - "Posuď, jaké podklady a stanoviska jsou nutné pro dodatečné povolení stavby."
      - "Který orgán má v tomto řízení požádat o JES a jak se postup eviduje?"
  en:
    displayName: "Monitor - Construction"
    summary: "Czech construction law, permitting, occupancy, documentation and affected authorities."
    examplePrompts:
      - "How is a building authorized under the former Building Act approved for use?"
      - "Assess the documents and opinions required for additional authorization of a structure."
      - "Which authority must request JES in this proceeding and how is the process recorded?"
  sk:
    displayName: "Monitor - Stavebníctvo"
    summary: "České stavebné právo, povoľovanie, kolaudácia, dokumentácia a dotknuté orgány."
    examplePrompts:
      - "Ako sa kolauduje stavba povolená podľa starého stavebného zákona?"
      - "Posúď podklady a stanoviská potrebné pre dodatočné povolenie stavby."
      - "Ktorý orgán má v tomto konaní požiadať o JES a ako sa postup eviduje?"
---

# CODEXIS - Stavebnictví

Standalone skill pro české stavební právo, povolování, kolaudaci, JES a související environmentální postupy.

## Operating Assumptions

- Pro právní předpisy, judikaturu a komentáře používej `cdx-cli`.
- Známý předpis otevři přes `/versions`; konkrétní část přes `/toc` a `/text?part=`.
- Metodiku ministerstva použij až po ověření jejího data, právního režimu a přesné skutkové podmínky.
- Dokumentové systémy ISSŘ, interní spisová služba a datová schránka nejsou právně zaměnitelné.

## Research Strategy

1. Identifikuj roli uživatele, orgán, typ záměru, místo, fázi a všechny rozhodné dny.
2. Sestav samostatné časové osy pro povolení, environmentální akty, výstavbu, změnu, dokončení, kolaudaci a nápravu.
3. Pro každý krok zvlášť určuj použitelný stavební zákon a přechodné ustanovení.
4. Klasifikuj postup jako nový záměr, změnu před dokončením, kolaudaci, dodatečné povolení, odstranění nebo přezkum.
5. Vytvoř matici aktérů a podkladů: kdo žádá, obstarává, vydává, doručuje, eviduje a přezkoumává.
6. Odděl stavební řízení od JES a jednotlivých složkových environmentálních postupů.

## Klíčové předpisy

| Předpis | CODEXIS base | Použití |
|---|---|---|
| zákon č. 283/2021 Sb., stavební zákon | `cz_law/283/2021` | nový stavební režim |
| zákon č. 183/2006 Sb. | `cz_law/183/2006` | dřívější stavební režim |
| zákon č. 148/2023 Sb., o JES | `cz_law/148/2023` | jednotné environmentální stanovisko |
| zákon č. 500/2004 Sb. | `cz_law/500/2004` | správní řízení a spis |
| zákon č. 334/1992 Sb. | `cz_law/334/1992` | ochrana ZPF |
| zákon č. 541/2020 Sb. | `cz_law/541/2020` | odpady a zeminy |
| zákon č. 254/2001 Sb. | `cz_law/254/2001` | vodní právo |
| zákon č. 114/1992 Sb. | `cz_law/114/1992` | ochrana přírody a krajiny |

## Hard Rules

- Jedno datum podání neurčuje automaticky právní režim všech navazujících úkonů.
- Nový záměr a dodatečné povolení existující stavby neposuzuj stejným postupem.
- Před návrhem podání nebo sdělení potvrď, zda uživatel jedná jako žadatel, stavební úřad, orgán JES nebo jiný dotčený orgán.
- U metodiky cituj přesnou podmínku a porovnej ji s časovou osou případu.
- Přijetí dokumentu ve spisové službě samo neprokazuje povinnost jeho vložení do ISSŘ ani opačně.
- Dočasná mezideponie, použití zeminy v místě stavby, vedlejší produkt a odpad jsou různé kvalifikace.
- Při práci s přílohou prohledej celý dokument; nevytvářej závěr z jedné strany.

## Workflow stavebního právníka

1. **Role a kompetence.** Urči orgán a procesní postavení uživatele.
2. **Záměr.** Popiš stavbu, pozemky, povolený a skutečný stav a požadovanou změnu.
3. **Timeline.** Podání, zahájení, vydání, právní moc, změna, dokončení a nový úkon.
4. **Přechod.** Aplikuj obecné i zvláštní přechodné pravidlo na každý proces zvlášť.
5. **Podklady.** Dokumentace, vlastnické tituly, souhlasy, JES, závazná stanoviska a vyjádření sítí.
6. **Evidence.** Autoritativní spis, doručení, ISSŘ a interní spisová služba.
7. **Výstup.** Kompetentní aktér, forma úkonu, adresát, lhůta a další krok.

## JES, role, přechody a spis

Při JES, dodatečném povolení, přechodných ustanoveních, ISSŘ nebo zemině načti `references/jes-role-prechody-a-spis.md`.

## Časté pasti

- Automatické použití jediného stavebního režimu na celý životní cyklus stavby.
- Přehlédnutí zvláštního přechodného pravidla environmentálního aktu.
- Odpověď z perspektivy žadatele, ačkoli uživatel je orgán JES.
- Směšování dodatečného povolení s povolením nového záměru.
- Nesprávné přenesení dokumentu mezi ISSŘ a interní spisovou službou.
- Označení dočasně uložené zeminy za vedlejší produkt bez testu skutečného dalšího použití.
- Neúplná anonymizace nebo review přílohy kvůli kontrole pouze poslední strany.

## Struktura odpovědi

1. Použitelný režim a role uživatele.
2. Dated timeline.
3. Druh postupu a kompetentní aktéři.
4. Podklady, stanoviska, dokumentace a evidence.
5. Lhůty, adresáti, vady a praktický další krok.
6. Předpoklady a přesné citace.

## Related references

- JES, role, přechody, spis a zemina: `references/jes-role-prechody-a-spis.md`.
