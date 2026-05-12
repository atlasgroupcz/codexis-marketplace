---
uuid: e5c6de23-bcb5-48b7-a845-8d8c472ff220
name: codexis-ucetnictvi
version: 2.1.0
i18n:
  cs:
    displayName: "CODEXIS — Účetnictví ČR"
    summary: "České účetní předpisy a praxe — závěrky, rozvaha, výsledovka, oceňování, odpisy, ČÚS i IFRS."
  en:
    displayName: "CODEXIS — Czech Accounting"
    summary: "Czech accounting regulations and practice — financial statements, balance sheet, income statement, valuation, ČÚS and IFRS."
  sk:
    displayName: "CODEXIS — Účtovníctvo ČR"
    summary: "České účtovné predpisy a prax — závierky, súvaha, výsledovka, oceňovanie, odpisy, ČÚS i IFRS."
description: Use when the user's question relates to Czech accounting, or when the answer may depend on an accounting rule even if framed around something else — účetnictví, účetní jednotka, závěrka, rozvaha, výsledovka, příloha závěrky, vybraná účetní jednotka, účetní období, kategorie účetních jednotek (mikro/malá/střední/velká), oceňování, odpisy, rezervy, opravné položky, inventarizace, daňová evidence, archivace, účetní doklad, kniha, zákon č. 563/1991 Sb., vyhláška 500/2002 / 501/2002 / 502/2002 / 504/2002 / 410/2009 Sb., ČÚS, IFRS, or any Czech accounting standard. Activate also when the user does not name účetnictví explicitly but their situation requires accounting treatment (reporting, valuation, period-end, capitalization vs. expense, recognition, record-keeping). Standalone authoritative skill — bundles general CODEXIS methodology with Czech accounting reasoning; no need to load general codexis.
---

# CODEXIS — Czech Accounting

Standalone skill for Czech accounting questions. Domain methodology for analyzing any accounting question — not a catalogue of answers.

## Operating Assumptions

- Use `cdx-cli` for all CODEXIS requests. Assume it is installed, authenticated, and operational.
- Canonical forms:
  - search: `cdx-cli search CR --query "..." --current --limit 5`
  - fetch: `cdx-cli get cdx://doc/<docId>/text`
  - schema: `cdx-cli schema versions` or `cdx-cli schema meta CR`
- Do not run `which cdx-cli`, inspect env vars, or call bare `cdx-cli` as a preflight step.
- Prefer flag-based `search` commands. Raw JSON payload is a fallback.
- Use only the documented `search`, `get`, and `schema` subcommands.

## Research Strategy

1. Start in CODEXIS.
2. Prefer source-specific search (`CR`, `EU`, `COMMENT`, `LT`) over `ALL`.
3. Use `ALL` only to orient source selection, then rerun in the specific source before citing.
4. For IFRS-related questions consult EU law (Regulation 1606/2002 framework) and local implementation.

## Data Sources

| Code | Name | Use for |
|------|------|---------|
| `CR` | Czech Legislation | Laws, decrees, standards published in legal codex |
| `EU` | EU Legislation | IFRS endorsement, directives on accounting / audit |
| `COMMENT` | Legal Commentaries | Authoritative interpretation of accounting provisions |
| `LT` | Legal Literature | Textbook-level context, methodology |
| `JD` | Czech Case Law | Rare primary source for accounting; mostly tax-accounting interplay |
| `ALL` | Global Search | Exploratory only |

## Klíčové české účetní předpisy

Referenční mapa — vždy ověř aktuální verzi přes `/versions` před extrakcí textu.

| Předpis | Číslo | CODEXIS base | Oblast |
|---|---|---|---|
| Zákon o účetnictví (ZoÚ) | 563/1991 Sb. | `cz_law/563/1991` | obecná úprava |
| Vyhláška pro podnikatele | 500/2002 Sb. | `cz_law/500/2002` | účtová osnova + výkazy pro PO podnikatele |
| Vyhláška pro banky | 501/2002 Sb. | `cz_law/501/2002` | banky a finanční instituce |
| Vyhláška pro pojišťovny | 502/2002 Sb. | `cz_law/502/2002` | pojišťovny |
| Vyhláška pro zdravotní pojišťovny | 503/2002 Sb. | `cz_law/503/2002` | zdravotní pojišťovny |
| Vyhláška pro NO a další | 504/2002 Sb. | `cz_law/504/2002` | NO subjekty (jednoduché účetnictví) |
| Vyhláška pro VÚJ | 410/2009 Sb. | `cz_law/410/2009` | vybrané účetní jednotky (veřejný sektor) |
| ZDP (tax-accounting vazba) | 586/1992 Sb. | `cz_law/586/1992` | daňová uznatelnost, daňové odpisy |

České účetní standardy (ČÚS): vydává MF, číslované podle vyhlášky — ČÚS 001–024 pro podnikatele, samostatné řady pro banky, pojišťovny, VÚJ.

## Jak hledat přes cdx-cli

```bash
# verze zákona k datu
cdx-cli get cdx://cz_law/<num>/<year>/versions

# konkrétní paragraf z aktuální verze
cdx-cli get 'cdx://doc/<versionId>/text?part=paragraf<NNN>'

# TOC zákona
cdx-cli get cdx://doc/<versionId>/toc

# search v účetní legislativě
cdx-cli search CR --query "účetní jednotka" --current --limit 5
```

## Hard Rules

- Pokud znáš číslo zákona nebo vyhlášky, nikdy nestartuj broad search.
- Pro změny začni `/versions`, ne `/text`.
- Pokud `/versions` neukáže boundary u query_date, odpověz to přímo.
- Používej base ID pro `/versions`, version ID pro `/text` a `/toc`.
- Pro konkrétní paragraf: nejdřív `/toc`, pak resolve `elementId`, pak `/text?part=<elementId>`.
- Nehádej `docId` — extrahuj z API response.
- Při selhání `/toc` fallback na `/text` celého předpisu.
- Neříkej, že něco „není v zákoně", bez fetch celé relevantní části.

## Workflow profesionálního účetního / auditora

1. **Reframe dotazu.** Klientovo lay wording je téměř vždy nepřesné. „Mám to dát do nákladů?" přelož do právní klasifikace: co je zdanitelné plnění, co je účetní náklad, co je výdaj, co je daňově uznatelný náklad.
2. **Urči účetní režim stran.** Podnikatel v plném / zjednodušeném rozsahu, vybraná účetní jednotka, nonprofit, OSVČ na daňové evidenci, subjekt pod IFRS. Neřeš otázku, dokud neznáš režim — metody a výkazy se liší.
3. **Určení kategorie účetní jednotky** (mikro / malá / střední / velká) — z ní plyne rozsah závěrky, audit, přílohy, konsolidace.
4. **Identifikuj VŠECHNY relevantní vrstvy úpravy.** Zákon o účetnictví × prováděcí vyhláška × ČÚS × vnitřní směrnice účetní jednotky. Složitější otázky vyžadují všechny čtyři.
5. **Substance over form.** Nájem × leasing (finanční × operativní) × kupní smlouva s odkladem; sham × genuine; dependent × independent. Účetní zachycení sleduje ekonomickou podstatu, ne právní formu, pokud zákon nestanoví jinak.
6. **Tax-accounting coupling.** Každou účetní otázku ověř z druhé strany: jaký dopad na daň z příjmů (daňová uznatelnost, odpisy, opravné položky), DPH, pojistné.
7. **Časové rozlišení.** Rozlišuj moment uznání (recognition) od momentu zdanění a od momentu cash flow. Přechodové rozdíly, akruální vs. kasová báze, období vs. okamžik.
8. **Procesní vrstva až nakonec.** Ocenění, zachycení, vykázání, uložení v závěrce, zveřejnění, archivace, audit.
9. **Draft-then-stress-test.** Návrh ověř proti counter-examples („co když není plátce DPH", „co když je mikro ÚJ", „co když nabyla majetek bezúplatně").

## Hierarchie pramenů

1. Ústavní pořádek / primární právo EU
2. Nařízení EU (zejména Nařízení 1606/2002 o IFRS) a směrnice (Účetní směrnice 2013/34/EU)
3. Zákon o účetnictví
4. Prováděcí vyhláška pro příslušný typ účetní jednotky
5. České účetní standardy (ČÚS)
6. Metodické pokyny a stanoviska MF (informace, sdělení, dopisy) → nevazné, ale typicky následované
7. Interpretace Národní účetní rady (NÚR) → odborná autorita, ne závazná
8. Judikatura NSS k účetním sporům (převážně kolem daňových důsledků)
9. Komentáře a odborná literatura (ASPI, Beck, Wolters Kluwer), stanoviska KA ČR, SÚ

Při rozporu mezi metodickým pokynem a judikaturou má přednost judikatura. Interpretace NÚR označ jako odbornou autoritu, ne závaznou normu.

## Časové působnost

Identifikuj *účetní období*, ve kterém transakce nastala, a aplikuj předpis v tehdejším znění — včetně přechodných ustanovení. Pro pokračující povinnosti (inventarizace, závěrka, audit) použij verzi platnou k rozvahovému dni, resp. ke dni podání. Zkontroluj, zda dostupný komentář postdatuje novelu (jinak může být obsoletní).

## Před odpovědí: retrieval checklist

Než odpovíš, projdi tento checklist. Pokud některý bod nesplňuješ, **neposílej odpověď** a dohledej chybějící předpis.

- **Procesní / organizační dotaz** („kdo zřizuje / jmenuje / stanoví / podává / rozhoduje / projednává"): neodpovídej z metodického pokynu, interní směrnice ani obecné úvahy, dokud jsi neověřil konkrétní paragraf závazného předpisu (zákon nebo prováděcí vyhláška). Metodika a vzor bez ověřeného § = drop.
- **Vybrané účetní jednotky (VÚJ — stát, obce, kraje, příspěvkové organizace, státní fondy)**: před odpovědí vždy ověř, zda existuje zvláštní veřejnosektorová úprava (zejména vyhláška 410/2009 Sb., vyhláška 270/2010 Sb. pro inventarizaci, České účetní standardy řady 700+). Neodpovídej z obecné úpravy pro podnikatele (vyhláška 500/2002 Sb.), pokud dotaz směřuje na VÚJ.
- **Obsah výkazové položky** (rozvaha, výsledovka, příloha): otevři příslušný paragraf prováděcí vyhlášky (500/2002 pro podnikatele, 410/2009 pro VÚJ, 501/2002 banky, 502/2002 pojišťovny) a odpovídej z **obsahového vymezení** položky v této vyhlášce. Název položky v běžném jazyce je zavádějící — obsah je definován legislativou.
- **Definiční pojem se zákonnou definicí** („účetní jednotka", „účetní období", „roční úhrn čistého obratu", „kategorie účetní jednotky"): otevři přesný § ZoÚ nebo prováděcí vyhlášky a **reprodukuj přesnou definici včetně všech kvalifikátorů** (obsahuje-li vzorec/výpočet, uveď ho úplně).
- **Dotaz na konkrétní účetní knihu / výkaz / metodu**: ověř v ČÚS pro daný typ účetní jednotky (ČÚS 001–024 pro podnikatele, 700+ pro VÚJ). Pokud ČÚS mlčí, ověř příslušný § vyhlášky a teprve potom ZoÚ.

## Časté pasti

- Odpověď na účetní otázku před určením režimu a kategorie účetní jednotky.
- Záměna účetního výsledku se základem daně z příjmů.
- Řešení jen podle zákona o účetnictví bez ověření prováděcí vyhlášky a ČÚS.
- Ignorování tax-accounting vazby (daňová uznatelnost, odpisy, opravné položky).
- Odvozování z běžného jazykového významu názvu výkazové položky místo obsahového vymezení ve vyhlášce.
- Pomíjení přílohy v účetní závěrce (klíčový výkaz s doplňkovými informacemi).
- Závěr podle nadpisu ustanovení bez ověření jeho obsahu.
- Spoléhání na zastaralý komentář po novele ZoÚ.
- Ignorování rozdílu mezi oceněním pořizovací cenou, reprodukční pořizovací cenou a vlastními náklady.
- Přebírání obecného pravidla tam, kde má ÚJ speciální režim (VÚJ, banky, pojišťovny, IFRS subjekty).

## Struktura daňové-účetní analýzy

Každou účetní otázku procházej těmito aspekty:

1. **Subjekt** — kdo je účetní jednotka a v jakém režimu.
2. **Předmět** — co se účtuje (aktivum, závazek, vlastní kapitál, náklad, výnos, podrozvahová položka).
3. **Ocenění** — jakou cenou; volba metody (FIFO / průměr u zásob; lineární / zrychlené odpisy; reálná hodnota; historická cena).
4. **Zachycení (recognition)** — v jaké knize, na jakém účtu, v jakém okamžiku.
5. **Časové rozlišení** — ve kterém období vykázat.
6. **Vykázání v závěrce** — v jaké položce rozvahy / výsledovky / přílohy.
7. **Daňový dopad** — uznatelnost, zdanění, opravné položky, rezervy daňově × účetně.
8. **Dokumentační a archivační povinnost** — účetní doklad, náležitosti, lhůta archivace, průkaznost.

Účetní hmotněprávní pravidlo odděluj od procesní / dokumentační povinnosti.

## Pravidla právního a účetního uvažování

Odpovídej nejdříve pozitivním operativním pravidlem pro přesně dotázanou věc. Výjimkami, negativním vymezením nebo carve-outy začínej jen tehdy, jsou-li samy předmětem dotazu.

Odpovídej na stejné úrovni konkrétnosti jako dotaz. Pokud se uživatel ptá na konkrétní výkazovou položku, účet nebo transakci, neodpovídej pouze nadřazenou obecnou definicí.

Pokud ustanovení používá souhrnný legální pojem („účetní jednotka", „roční úhrn čistého obratu", „vybraná účetní jednotka"), rozveď jej do konkrétních subjektů nebo kategorií a **zachovej přesnou zákonnou formulaci včetně kvalifikátorů** (např. „alespoň", „nejméně", „po sobě následujících", „k rozvahovému dni"). Obsahuje-li definice vzorec nebo výpočet, uveď jej úplně.

Nekončit u prvního nalezeného ustanovení. Ověřuj celý normativní stack — zákon × vyhláška × ČÚS × metodický pokyn × NÚR interpretace.

U praktických dotazů („jak účtovat", „na co myslet") pokrývej všechny relevantní dimenze: klasifikace, ocenění, okamžik uznání, zachycení, vykázání, časové rozlišení, přílohu závěrky, archivaci, kontrolní mechanismy, daňový dopad.

Pokud závěr závisí na prahu, kategorii jednotky, volbě metody nebo jiné rozhodné podmínce, identifikuj ji. Konkrétní limity (obrat, aktiva, zaměstnanci, daňové prahy) **vždy ověř v aktuálním znění**, nikdy je neuváděj z paměti.

Nadpis ustanovení nenahrazuje jeho obsah. Pokud nadpis odpovídá dotazu, ale ustanovení obsahuje jen výjimku nebo dílčí pravidlo, nespoléhej na něj jako na úplnou odpověď.

Při citaci více předpisů je vždy vyřeš v jednom časovém řezu. Nemíchej znění z různých dat.

## Cross-referencing

Účetní otázky typicky vyžadují propojení:
- zákon o účetnictví × prováděcí vyhláška × ČÚS
- účetní předpis × daňový předpis (ZDP, ZDPH)
- účetní pravidlo × vnitřní směrnice účetní jednotky
- národní standard × IFRS (pokud se aplikuje)
- obecné pravidlo × speciální režim (VÚJ, banky, pojišťovny)

Pokud jedna vrstva odkazuje na druhou, dohledej a uveď.

## Struktura odpovědi

1. **Krátký závěr up front** — co se účtuje, v jaké výši, kdy.
2. **Zvolený režim a kategorie** účetní jednotky a důvod.
3. **Pozitivní pravidlo** — obecný přístup podle zákona + vyhlášky + ČÚS.
4. **Speciální režim** / výjimka, pokud se aplikuje.
5. **Ocenění a výpočet**.
6. **Vykázání v závěrce** — konkrétní položka rozvahy / výsledovky / přílohy.
7. **Daňový dopad** a vazba na ZDP / ZDPH / pojistné.
8. **Procesní povinnosti** — doklad, kniha, archivace, inventarizace, audit.
9. **Caveats** — předpoklady, datum znění zákona použité v odpovědi, otevřené otázky.
10. **Citace** v pořadí: zákon → vyhláška → ČÚS → metodické pokyny MF → NÚR → judikatura → komentář.

## Proactive Reference Enrichment

Kdykoli v textu, tool outputu nebo extraktu vidíš právní referenci, resolvuj ji v CODEXIS a použij `cdx://doc/` link. Neprezentuj raw reference bez pokusu o lookup. Pokud reference není v CODEXIS, ponechej plain text. Po sobě jdoucí paragrafy shrň do rozmezí.

## Short Best Practices

- Závěry stav primárně na zákonu + vyhlášce + ČÚS. Komentář a literaturu až sekundárně.
- Filtruj brzy (`--current`, `--valid-at`, `--type`, `--limit`).
- Velký text stáhni jednou, extrahuj lokálně.
- Pro filter a shape používej `jq`.

## Related references

- Deep-dive LAW change assessment: `references/czech-law-change-assessment.md`.
- Judikatura: `references/judikatura.md` (u účetnictví zřídka primární).
