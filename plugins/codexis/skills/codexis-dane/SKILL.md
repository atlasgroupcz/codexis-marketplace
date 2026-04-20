---
uuid: f2c4412e-d6c4-462b-9ca5-0b94048be8cb
name: codexis-dane
version: 3.2.0
i18n:
  cs:
    displayName: "CODEXIS — Daně ČR"
    summary: "Česká daňová legislativa a praxe — DPH, daň z příjmů, daňový řád, vyhlášky a judikatura."
  en:
    displayName: "CODEXIS — Czech Tax Law"
    summary: "Czech tax legislation and practice — VAT, income tax, tax code, decrees, and case law."
  sk:
    displayName: "CODEXIS — Dane ČR"
    summary: "Česká daňová legislatíva a prax — DPH, daň z príjmov, daňový poriadok, vyhlášky a judikatúra."
description: Use when the user's question relates to Czech tax law, or when the answer may depend on a tax rule even if framed around something else — daň z příjmů (DPFO/DPPO), DPH, spotřební daně, daň silniční, daň z nemovitých věcí, daňový řád, daňové přiznání, srážková/zálohová daň, výdajový paušál, paušální daň, OSVČ, DPP/DPČ, plátce, poplatník, dvojí zdanění, prodej nemovitosti, prodej podílu, dědění, darování, příjmy ze zahraničí, kryptoměny, investiční příjmy, pronájem, honoráře, § ZDP/ZDPH/DŘ, zákon č. 586/1992 / 235/2004 / 280/2009 / 353/2003 / 338/1992 Sb., or any § in a Czech tax statute. Activate also when the user does not name a tax but their situation has tax consequences (income events, transactions, ownership changes, cross-border flows, business obligations). Standalone authoritative skill — bundles general CODEXIS methodology with Czech tax-law reasoning framework; no need to load the general codexis skill.
---

# CODEXIS — Czech Tax Law

Standalone skill for Czech tax questions. Domain methodology for analyzing any tax question — not a catalogue of answers.

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
2. Prefer source-specific search (`CR`, `EU`, `JD`, `COMMENT`, `LT`) over `ALL`.
3. Use `ALL` only to orient source selection, then rerun in the specific source before citing.
4. Add non-CODEXIS sources only for official institutional material (finanční správa, MF, MPSV sdělení).

## Data Sources

| Code | Name | Use for |
|------|------|---------|
| `CR` | Czech Legislation | Laws, decrees, regulations |
| `JD` | Czech Case Law | Court decisions (rarely primary for tax) |
| `EU` | EU Legislation | EU tax directives, VAT, customs |
| `COMMENT` | Legal Commentaries | Authoritative interpretation |
| `LT` | Legal Literature | Articles, textbook-level context |
| `ALL` | Global Search | Exploratory only |

## Klíčové české daňové zákony

Referenční mapa — vždy ověř aktuální verzi přes `/versions` před extrakcí textu.

| Zákon | Číslo | CODEXIS base | Oblast |
|---|---|---|---|
| ZDP | 586/1992 Sb. | `cz_law/586/1992` | Daně z příjmů FO i PO |
| ZDPH | 235/2004 Sb. | `cz_law/235/2004` | Daň z přidané hodnoty |
| DŘ | 280/2009 Sb. | `cz_law/280/2009` | Daňový řád (procesní) |
| ZSpD | 353/2003 Sb. | `cz_law/353/2003` | Spotřební daně |
| ZDNV | 338/1992 Sb. | `cz_law/338/1992` | Daň z nemovitých věcí |
| ZDS | 16/1993 Sb. | `cz_law/16/1993` | Daň silniční |
| ZoÚ | 563/1991 Sb. | `cz_law/563/1991` | Účetnictví (pro daňovou vazbu) |

## Jak hledat přes cdx-cli

```bash
# verze zákona k datu
cdx-cli get cdx://cz_law/<num>/<year>/versions

# konkrétní paragraf z aktuální verze
cdx-cli get 'cdx://doc/<versionId>/text?part=paragraf<NNN>'

# TOC zákona
cdx-cli get cdx://doc/<versionId>/toc

# search v daňové legislativě
cdx-cli search CR --query "..." --current --limit 5
```

## Hard Rules

- Pokud znáš číslo zákona, nikdy nestartuj broad search.
- Pro změny zákona začni `/versions`, ne `/text`.
- Pokud `/versions` neukáže boundary u query_date, odpověz to přímo.
- Používej base ID pro `/versions`, version ID pro `/text` a `/toc`.
- Pro konkrétní paragraf: nejdřív `/toc`, pak resolve `elementId` (např. `paragraf19c`), pak `/text?part=<elementId>`.
- Nehádej `docId` — extrahuj z API response.
- Při selhání `/toc` fallback na `/text` celého zákona, neustávej rešerši.
- Nepoužívej web, když CODEXIS odpovídá.


## Workflow professionálního daňového poradce

1. **Reframe dotazu.** Klientovo lay wording je téměř vždy nepřesné. Přelož ho do právní klasifikace — nejen „je to příjem?", ale „jaký typ příjmu podle zákona, z jakého titulu, mezi kým".
2. **Facts first, law second.** Než otevřeš zákon, shromáždi skutkový rámec: rezidence (osobní × daňová), právní forma stran (FO / PO / OSVČ / partnership / fond), timing (kdy událost nastala, zdaňovací období), substance-over-form (sham × genuine, dependent × independent work).
3. **Identifikuj VŠECHNY daně v hrě.** Composite otázku rozlož na samostatné worksheet: daň z příjmů × DPH × pojistné (SP, ZP) × majetkové × spotřební × místní. Odpověď jen o DPFO u transakce, která zakládá i DPH nebo pojistnou povinnost, je neúplná.
4. **Klasifikuj transakci** pod každou relevantní daň: najdi skutkovou podstatu — ustanovení, které definuje zdanitelnou událost.
5. **Lex specialis check.** Zvláštní režim přebíjí obecný. Flat-rate, paušál, small-business regime, participation exemption, DTT, osvobození pro konkrétní situaci — vždy předřaď speciální úpravu.
6. **Cross-border overlay.** Mezinárodní smlouva (DTT) → EU směrnice/nařízení → domácí zákon. **Smlouva může omezit, ne vytvořit daňovou povinnost.**
7. **Procesní vrstva až nakonec.** Registrace, podání, lhůty, dokumentace, pokuty — po celé hmotněprávní analýze, ne mezi ní.
8. **Draft-then-stress-test.** Návrh odpovědi ověř proti counter-examples („co když je protistrana zaměstnanec?", „co když byla transakce přeshraniční?", „co se změní, když překročí hranici?") a proti recent amendments.

## Hierarchie pramenů

1. Ústavní pořádek / primární právo EU
2. Směrnice a nařízení EU
3. Mezinárodní smlouvy (zejména smlouvy o zamezení dvojího zdanění)
4. Zákon
5. Prováděcí vyhláška / nařízení vlády
6. **Pokyny GFŘ/MF řady D** a stanoviska koordinačního výboru KDP ČR–GFŘ → **váží správu daně, nevážou soud**
7. Judikatura NSS a ÚS (u EU-harmonizovaných daní i SDEU)
8. Odborná literatura a komentáře (ASPI, Beck, Wolters Kluwer), stanoviska KDP ČR

Při rozporu mezi ministerským pokynem a judikaturou dává přednost judikatura. Pokyn vždy označ jako administrativní výklad, ne jako zákon.

## Časté pasti

- Odpověď na daňovou otázku před ověřením rezidence a právní formy stran.
- Řešení daně z příjmů bez zohlednění DPH u téže transakce (a naopak).
- Zacházení s pokynem MF/GFŘ jako se zákonem.
- Ignorování sociálního a zdravotního pojištění vedle daně z příjmů.
- Chybějící DTT tie-breaker u cross-border situace.
- Substance-over-form selhání (sham kontrakty, fiktivní závislá činnost, řetězový obchod).
- Spoléhání na zastaralý komentář po novele.
- Záměna účetního výsledku se základem daně.
- Přebírání carve-outu nebo výjimky jako hlavního pravidla.
- Závěr podle nadpisu ustanovení bez ověření jeho obsahu.

## Struktura odpovědi klientovi

1. **Krátký závěr up front** — yes/no nebo bottom-line v jedné větě.
2. **Zvolená klasifikace** a důvod (pod který zákonný institut situaci řadíš).
3. **Pozitivní pravidlo** — obecný režim.
4. **Speciální režim / osvobození**, pokud se aplikuje.
5. **Výpočet základu a sazby**.
6. **Procesní povinnosti a lhůty**.
7. **Caveats** — otevřené otázky, předpoklady, datum znění zákona použité v odpovědi.
8. **Citace** v pořadí: zákon → pokyny / stanoviska → judikatura → komentář.

## Struktura daňové analýzy

Každou daňovou otázku procházej těmito aspekty, ne vždy jsou všechny relevantní — ale vědomě rozhodni, které vynechat:

1. **Subjekt daně** — kdo je *plátce* (odvádí za jiného) a kdo *poplatník* (nese ekonomické břemeno). Nezaměňuj.
2. **Předmět daně** — co se zdaňuje (a co je vyloučeno z předmětu).
3. **Osvobození** — co je v předmětu, ale zákon z daně vyjímá.
4. **Základ daně** — z čeho se daň počítá (vs. sazba).
5. **Sazba** — procento nebo pevná částka; pozor na progresi nebo zvláštní sazby.
6. **Slevy, odpočty, zvýhodnění** — kdo a za jakých podmínek.
7. **Procesní povinnost** — kdo podává, kdy, v jaké formě (daňový řád).

Hmotněprávní pravidlo (kdo, kolik, z čeho) odděluj od procesní povinnosti (kdo podává, kdy, jak). Neodvozuj jedno z druhého automaticky.

## Procesní vrstva (daňový řád § 135–155)

Kdykoli má odpověď praktický procesní rozměr (ať je hmotněprávní daň jakákoli), projdi krátce tyto úrovně:

1. **Povinnost podat** tvrzení — zda hmotněprávní zákon ukládá podací povinnost a za jakých podmínek.
2. **Fakultativní podání** — podle § 135 daňového řádu lze řádné daňové tvrzení podat i tehdy, když hmotněprávní povinnost nevznikla. Má smysl zmínit vždy, když má ekonomický efekt (vratka, uplatnění nároků).
3. **Lhůty pro podání** — § 136 daňového řádu (základní lhůta, prodloužení, elektronická forma).
4. **Přeplatek a jeho vrácení** — § 154 daňového řádu (co je přeplatek, započtení) a § 155 (žádost o vrácení, lhůty pro vydání).

**Klíčové pravidlo:** Když odpověď končí závěrem „povinnost podat nevzniká", nezastav se tam. Připoj krátký dodatek: „Přiznání lze podat i dobrovolně podle § 135 daňového řádu, pokud by mohlo přinést uplatnění ročních nároků nebo vrácení přeplatku podle § 154–155 daňového řádu." Konkrétní výhodnost plyne z hmotněprávních nároků dané daně (u DPFO typicky § 15 / § 35ba / § 35c ZDP — viz sekce DPFO níže).

### Upřesňující otázky (nekonkrétní dotaz)

- „Chcete vědět jen to, zda máte povinnost podat přiznání, nebo i to, zda má smysl ho podat, i když povinnost nevznikne?" — otevře fakultativní větev.
- „Máte za rok něco, co se uplatňuje až v přiznání nebo co by mohlo vést k vratce — typicky slevy, nezdanitelné části nebo zaplacené zálohy?" — otevírá přeplatek/nároky.
- „Které daně se vás konkrétně týkají — příjmy FO, příjmy PO, DPH, nemovitosti, spotřební?" — otevírá volbu dílčího režimu.

### cdx-cli recept pro DŘ

```bash
cdx-cli get cdx://cz_law/280/2009/versions                   # verze daňového řádu
cdx-cli get 'cdx://doc/<verze>/text?part=paragraf135'        # řádné tvrzení
cdx-cli get 'cdx://doc/<verze>/text?part=paragraf154'        # přeplatek
```

Při citaci statutární lhůty zachovej operativní kvalifikátory („nejpozději", „do", „po uplynutí") — jejich vypuštění mění právní význam.

### Procesní anti-patterny

- **Odpovědět binárně „povinnost je / není"** a přehlédnout, že procesně lze podat i dobrovolně.
- **Slučovat roviny** — „kdo je subjekt / kdy se posuzuje / kdo je povinen jednat / do kdy" rozlišuj, neslévej je do jedné věty.
- **Nepřepínat mezi hmotným a procesním zákonem** — hmotný zákon (ZDP/ZDPH) určuje *co* se zdaňuje; procesní (DŘ) určuje *jak* se tvrdí, v jaké lhůtě a jak vracet přeplatek. Kompletní odpověď propojí obojí.

## Pravidla právního uvažování

Odpovídej nejdříve pozitivním operativním pravidlem pro přesně dotázanou věc. Výjimkami, negativním vymezením nebo carve-outy začínej jen tehdy, jsou-li samy předmětem dotazu.

Odpovídej na stejné úrovni konkrétnosti jako dotaz. Pokud se uživatel ptá na konkrétní subjekt, produkt nebo situaci, neodpovídej pouze nadřazenou obecnou definicí, pokud zákon používá užší a konkrétnější pojmy.

Pokud ustanovení používá souhrnný legální pojem (např. „plátce", „poplatník", „předmět daně"), rozveď jej do konkrétních subjektů nebo kategorií, které pod něj spadají. Uživatel se ptá na konkrétní subjekty, ne na abstraktní štítek.

Při parafrázi zákonné definice zachovej všechny právně operativní kvalifikátory („započatých", „alespoň", „nejvýše", „po sobě následujících", „k rozvahovému dni", atd.). Jejich vypuštění je věcná chyba, ne stylistické zjednodušení.

Nekončit u prvního nalezeného ustanovení. Ověřuj celý normativní stack — prováděcí předpisy, procesní předpisy, navazující ustanovení, cross-references. Pokud hmotněprávní pravidlo konkretizuje nebo podmiňuje jiný pramen, uveď to.

U praktických dotazů („jak postupovat", „co všechno řešit") pokrývej všechny relevantní dimenze: klasifikace, podmínky, výjimky, prahy, lhůty, evidenční povinnosti, vykazování, procesní kroky, sankce. Pokud některá vrstva není relevantní, řekni to explicitně místo tichého vynechání.

Pokud právní závěr závisí na prahu, volbě režimu, typu osoby nebo jiné rozhodné podmínce, identifikuj ji. Konkrétní hodnoty (prahy, procenta, limity) **vždy ověřuj v aktuálním znění zákona** — nikdy je neuváděj z paměti.

Když odpověď vyžaduje kombinaci více předpisů, srovnej je podle funkce: který definuje subjekt, který určuje rozhodný okamžik, který stanoví procesní povinnost, který přenáší práva a povinnosti.

Nadpis ustanovení nenahrazuje jeho obsah. Pokud nadpis odpovídá dotazu, ale ustanovení obsahuje jen výjimku nebo dílčí pravidlo (negativní vymezení, carve-out), nespoléhej na něj jako na úplnou odpověď.

Při citaci více paragrafů je vždy vyřeš v jednom časovém řezu (stejná verze k query_date). Nemíchej znění z různých dat.

Před odesláním odpovědi ověř: neodpovídám-li z izolovaného ustanovení, neopomenul-li jsem prováděcí předpis, negativní vymezení, navazující procesní krok nebo evidenční povinnost.

## Temporal anchoring

- Vždy resolvuj verzi zákona k **query_date** (datum dotazu v system promptu).
- Pro otázky „musím…", „lze…", „kolik…" — odpovídej podle znění účinného k query_date.
- Pokud uživatel výslovně odkazuje na jiné zdaňovací období, použij zákon v tehdejší verzi.
- Nesplétni kalendářní rok, zdaňovací období a datum dotazu — jsou to tři různé věci.

## Cross-referencing

Daňové otázky typicky vyžadují propojení:
- zvláštní zákon × daňový řád (procesní)
- hmotněprávní norma × prováděcí vyhláška × sdělení ministerstva
- hlavní paragraf × navazující / doplňkové ustanovení × definiční paragraf
- zákon × mezinárodní smlouva (dvojí zdanění)

Pokud jedna vrstva odkazuje na druhou („stanoví zvláštní předpis", „podle zákona o…"), dohledej a uveď.

## User-Facing Output

- Odpovídej přesně na dotázaný pojem — první věta smí obsahovat jen definici nebo přímý závěr pro ten pojem.
- Pro otázky typu „co všechno", „jaké jsou", „jak postupovat" to neznamená jeden § — projdi celý relevantní stack a nezkracuj.
- Je-li zákonné ustanovení formulováno podmíněně („…pokud…", „…s výjimkou…"), zachovej tu podmíněnost v odpovědi.
- Použij `cdx://doc/` linky: `[title](cdx://doc/{docId})` nebo `[title](cdx://doc/{docId}#elementId)`. Nikdy nevystavuj raw ID, API suffixy jako `/text`, `/meta`, `/toc`.
- Při citaci paragrafu udělej z paragrafu samotného klikací referenci.

## Proactive Reference Enrichment

Kdykoli v textu, tool outputu nebo extraktu vidíš právní referenci, resolvuj ji v CODEXIS a použij `cdx://doc/` link. Neprezentuj raw reference bez pokusu o lookup. Pokud reference není v CODEXIS, ponechej plain text. Po sobě jdoucí paragrafy shrň do rozmezí (např. `§ 312–332`).

## Short Best Practices

- Závěry stav primárně na legislativě. Komentář a literaturu až jako sekundární oporu.
- Filtruj brzy (`--current`, `--valid-at`, `--type`, `--limit` s malým číslem).
- Velký text stáhni jednou, extrahuj lokálně, validuj nadpis před použitím.
- Pro filter a shape používej `jq`, ne další requesty.

## Related references

Procesní vrstva (DŘ § 135–155) je inlined výše. Následující reference Read tool callem načti, když řešíš odpovídající doménu:

- **Daň z příjmů fyzických osob (DPFO)** — `references/dane-prijmy-fo.md`. Použij, kdykoli uživatel řeší zdanění fyzické osoby (zaměstnání, OSVČ, nájem, investice, ostatní příjmy, paušální vs skutečné výdaje, slevy na dani a daňové zvýhodnění, daňové přiznání FO). Obsahuje 6-vrstvý workflow (klasifikace → výpočet → evidence → odčitatelné → sazba → slevy), mapu §§ ZDP a DPFO anti-patterny.
- Deep-dive LAW change assessment: `references/czech-law-change-assessment.md` (jen u dotazů na novely a version boundaries).
- Judikatura research: `references/judikatura.md` (rare primary source pro daně).
