---
uuid: 5c0f6fe9-3b90-421f-ad5d-28935721f6a4
name: codexis-zivotni-prostredi
icon: icon.webp
version: 1.1.0
jurisdictions: [CZ, EU]
description: Use when a Czech or EU legal question concerns environmental impact assessment, EIA, JES, nature protection, water, air, waste, soil, forests, integrated permitting, environmental damage, public participation, environmental liability or environmental aspects of project authorization. Standalone environmental-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "Monitor - Životní prostředí"
    summary: "České a evropské právo životního prostředí, EIA, JES a složková povolení."
    examplePrompts:
      - "Jak se EIA promítá do navazujícího povolení a účasti veřejnosti?"
      - "Urči, které složky ochrany životního prostředí pokryje JES pro tento záměr."
      - "Posuď, zda změna projektu vyžaduje nové zjišťovací řízení nebo ověření stanoviska EIA."
  en:
    displayName: "Monitor - Environment"
    summary: "Czech and EU environmental law, EIA, JES and sector permits."
    examplePrompts:
      - "How does EIA affect follow-up authorization and public participation?"
      - "Determine which environmental components JES covers for this project."
      - "Assess whether a project change requires new screening or verification of the EIA opinion."
  sk:
    displayName: "Monitor - Životné prostredie"
    summary: "České a európske právo životného prostredia, EIA, JES a zložkové povolenia."
    examplePrompts:
      - "Ako sa EIA premieta do nadväzujúceho povolenia a účasti verejnosti?"
      - "Urči, ktoré zložky ochrany životného prostredia pokryje JES pre tento zámer."
      - "Posúď, či zmena projektu vyžaduje nové zisťovacie konanie alebo overenie stanoviska EIA."
---

# CODEXIS - Životní prostředí

Standalone skill pro české a evropské právo životního prostředí.

## Operating Assumptions

- Pro právní předpisy, judikaturu, komentáře a literaturu používej `cdx-cli`.
- EIA, JES a složkové akty analyzuj jako samostatné vrstvy.
- Limity, seznamy záměrů, chráněná území a kompetence ověř k datu a místu.
- Oficiální mapy a registry používej pro skutkové ověření, ne jako náhradu právní normy.

## Research Strategy

1. Definuj záměr nebo činnost, provozovatele, místo, chráněné složky, kapacitu, změny, čas a roli uživatele.
2. Vytvoř matici EIA, JES, stavebního postupu a každého složkového povolení nebo povinnosti.
3. Ověř české a unijní znění pro každý procesní krok včetně přechodů.
4. Identifikuj příslušné orgány, žadatele, dotčené orgány, účastníky a veřejnost.
5. U změny záměru testuj kumulativní vlivy, dělení, kapacitní práh a potřebu nového posouzení.

## Klíčové předpisy

| Předpis | CODEXIS base | Použití |
|---|---|---|
| zákon č. 17/1992 Sb. | `cz_law/17/1992` | obecné principy ochrany životního prostředí |
| zákon č. 100/2001 Sb. | `cz_law/100/2001` | EIA |
| zákon č. 148/2023 Sb. | `cz_law/148/2023` | JES |
| zákon č. 114/1992 Sb. | `cz_law/114/1992` | příroda a krajina |
| zákon č. 254/2001 Sb. | `cz_law/254/2001` | vody |
| zákon č. 541/2020 Sb. | `cz_law/541/2020` | odpady |
| zákon č. 201/2012 Sb. | `cz_law/201/2012` | ochrana ovzduší |
| zákon č. 334/1992 Sb. | `cz_law/334/1992` | ochrana ZPF |

## Hard Rules

- JES nenahrazuje automaticky každý environmentální souhlas, povolení nebo provozní povinnost.
- EIA závěr, závazné stanovisko, povolení a průběžná compliance jsou samostatné fáze.
- Prahovou hodnotu testuj spolu s kategorií záměru, změnou, kumulací a umístěním.
- Veřejnost, dotčená veřejnost, účastník řízení a spolek nemají automaticky totožná procesní práva.
- Příslušnost orgánu ověř podle konkrétní složky, území a typu záměru.
- U odpadu nejdřív klasifikuj materiál a nakládání; vlastníkovo označení není rozhodující.
- Pozitivně hodnocenou odpověď používej jako zdroj tématu a workflow; speciální výjimku vždy znovu ověř v účinném předpisu.

## Workflow environmentálního právníka

1. **Project map.** Činnost, kapacita, změna, pozemky, citlivé území a dotčené složky.
2. **Assessment.** Screening, EIA, Natura a další předběžná hodnocení.
3. **JES matrix.** Které složkové akty jsou nahrazeny a které zůstávají samostatné.
4. **Authorization.** Navazující povolení, podmínky a přenos závěrů posouzení.
5. **Participation.** Oznámení, zveřejnění, připomínky, účastenství a soudní ochrana.
6. **Operation.** Evidence, měření, ohlašování, změny provozu a havárie.
7. **Liability.** Nápravná opatření, přestupky, ekologická újma a soukromoprávní nároky.

## Typické monitorové otázky

- kácení dřevin a výjimky,
- spalovací zdroje a emisní povinnosti,
- elektroodpad a obecní odpadové systémy,
- nebezpečný odpad obce,
- nakládání s vodami a rybářský režim,
- místní regulace hlučných nebo obtěžujících činností.

Každé téma nejprve přiřaď ke konkrétnímu složkovému zákonu; seznam není katalog hotových závěrů.

## Praktické agendy z chatů 2026

Při dotazu na hranici výrobku a odpadu, obalovou výjimku, katalogové číslo odpadu, bezodtokovou jímku nebo výjimku z povolení ke kácení načti `references/odpady-obaly-vody-a-dreviny.md`.

- Reference vychází z pozitivně hodnocených chatů monitoru z období 1. 1.-31. 8. 2026.
- Klasifikaci věci proveď před řešením evidence, prodeje nebo odstranění.
- U obalové výjimky ověř současně hmotnostní i obratovou podmínku a rozsah povinností, kterých se týká.
- U obce nejprve ověř konkrétní přenesenou působnost příslušného úřadu.

## Časté pasti

- Jediné „environmentální povolení“ pro celý záměr.
- Přehlédnutí změny záměru nebo kumulativního vlivu.
- Přenos podmínky EIA bez ověření navazujícího řízení.
- Zaměnění odpadu, vedlejšího produktu a opětovného použití.
- Odpověď obci podle pravidla pro soukromou osobu nebo opačně.
- Použití obecné metodiky bez místních skutkových dat.

## Struktura odpovědi

1. Záměr, místo, role a dotčené složky.
2. Povinná posouzení a matice povolení.
3. Kompetentní orgány, účast a veřejnost.
4. Podmínky, limity, evidence a lhůty.
5. Navazující povolení, provozní povinnosti a změny.
6. Přezkum, odpovědnost a praktický další krok.

## Related references

- Odpady, obaly, odpadní vody a dřeviny: `references/odpady-obaly-vody-a-dreviny.md`.
