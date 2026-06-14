---
uuid: 214caaa4-3728-4d21-b379-ab4b376b7615
name: ares
description: "Český ARES pro právní ověřování firem a podnikatelů: vyhledání podle názvu nebo IČO, základní identifikace, DIČ, sídlo, právní forma, stav registrací, CZ-NACE, statutární orgány, způsob jednání a živnosti."
version: 0.1.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "ARES — ověřování ekonomických subjektů"
    summary: "Vyhledávání a ověřování českých firem a podnikatelů v ARES pro právní použití."
  en:
    displayName: "ARES — Czech Entity Verification"
    summary: "Verify Czech companies and entrepreneurs using ARES."
---

# ARES — ověřování ekonomických subjektů

Použij tento skill, když uživatel chce ověřit českou firmu, podnikatele nebo jiný ekonomický subjekt podle názvu, IČO, statutárních orgánů nebo živností.

Pracuj v jazyce uživatelského vstupu a používej českou právní a rejstříkovou terminologii: IČO, obchodní firma, sídlo, právní forma, statutární orgán, způsob jednání, živnostenské oprávnění.

## Nástroj

Používej pouze CLI:

```bash
ares <command>
```

Nepoužívej přímo `curl` ani jiné HTTP nástroje. CLI vrací JSON na stdout. Mapované příkazy obsahují `echo`; `ares raw` vrací původní JSON bez interpretace. Pokud CLI vypíše `ERROR: ...`, chybu stručně sděl uživateli a nevymýšlej chybějící údaje.

## Identifikace subjektu

Před voláním ARES musí být známé IČO nebo název/část názvu subjektu. Pokud uživatel neuvede ani jedno, vyzvi ho, aby doplnil IČO nebo název subjektu, a žádný příkaz zatím nespouštěj.

Když uživatel uvede IČO, nehledej podle názvu. Použij rovnou příkaz podle původního dotazu: `ares company <ico>`, `ares officers <ico>`, `ares trades <ico>` nebo `ares raw <ico> --source <source>`.

Když uživatel neuvede IČO, ale uvede název nebo část názvu, spusť `ares search "<dotaz>"` a vyhodnoť `kandidati`:

- 0 kandidátů: sděl, že ARES nenašel žádný odpovídající subjekt; požádej uživatele, aby zkontroloval název nebo zadal IČO.
- 1 kandidát: vypiš název a IČO; toto IČO použij pro pokračování v původním business case (`company`, `officers`, `trades`, případně `raw`).
- Více kandidátů: kandidáti jsou lokálně seřazeni podle relevance, ale pořád je nepovažuj za jednoznačný výběr. Zobraz kandidáty jako tabulku se sloupci `IČO | název subjektu | právní forma | sídlo` a zeptej se, se kterým subjektem chce uživatel pokračovat. Dokud uživatel nevybere subjekt nebo nezadá IČO, nespouštěj detailní příkazy.

Když uživatel žádá širší vyhledání nebo je potřeba ukázat více než 10 kandidátů, použij `ares search "<dotaz>" --limit <počet>`.

## Rozhodovací strom

Nejdřív jednoznačně urči subjekt podle sekce `Identifikace subjektu`. Po určení IČO zvol příkaz podle obsahu dotazu:

- Název nebo část názvu bez IČO: `ares search "<dotaz>"` (zdroj: ARES agregace nad více státními registry; slouží k nalezení IČO).
- Základní karta subjektu: `ares company <ico>` (zdroj: ARES agregace nad ROS, RES, OR a dalšími zdroji).
- Identifikační údaje: `ares company <ico>` pro obchodní firmu/jméno, IČO, DIČ, sídlo, právní formu, primární zdroj, datum vzniku/zániku, datum aktualizace.
- Registrace a stav registrací: `ares company <ico>`.
- CZ-NACE/NACE, obor ekonomické činnosti nebo klasifikace činnosti: `ares company <ico>`.
- Statutární orgány, jednatelé, představenstvo, členové orgánů, funkce, způsob jednání, podpisové oprávnění: `ares officers <ico>` (zdroj: VR, veřejný rejstřík Ministerstva spravedlnosti).
- Spisová značka, základní kapitál, společníci/akcionáři, veřejnorejstříkové exekuce/insolvence/konkurs: `ares officers <ico>`.
- Živnosti, živnostenská oprávnění, předmět podnikání ze živnostenského rejstříku, obory živnosti, provozovny, odpovědní zástupci: `ares trades <ico>` (zdroj: RŽP, živnostenský rejstřík).
- Surová odpověď zdroje, ladění, konkrétní registr nebo pole mimo zjednodušený výstup: `ares raw <ico> --source <source>`; zdroje jsou `basic` (ARES agregace), `vr` (VR), `res` (RES/ČSÚ), `rzp` (RŽP). Pokud uživatel chce data přímo z RES, použij `--source res`; zjednodušený RES příkaz CLI nemá.

Pokud dotaz kombinuje více oblastí, spusť více detailních příkazů po jednom. Například dotaz na základní údaje a jednatele vyžaduje `ares company <ico>` a `ares officers <ico>`.

## Reference

`references/cli.md` se nenačítá automaticky. Použij ho, když potřebuješ přesné názvy příkazů, endpointy, raw sources, chybové chování nebo mapování výstupních polí. Při otázkách na konkrétní pole jako `czNace`, `registrace`, `zpusobJednani`, `zivnosti` nebo `spisovaZnacka` si podle něj ověř, který příkaz pole vrací.

## Jak odpovídat

V odpovědi vždy uveď obchodní firmu nebo jméno, IČO a zdroj dat, pokud je dostupný. U adresy preferuj textovou adresu vrácenou ARES. U statutárních orgánů odděl osobu/funkci od způsobu jednání.

Když je výsledek neúplný, napiš to výslovně. Nezaměňuj základní identifikaci za výpis z obchodního rejstříku a neprezentuj nepřítomnost údaje jako právní závěr.

## Doporučený postup

1. Ověř, že uživatel zadal IČO nebo název subjektu; jinak si ho vyžádej.
2. Pokud chybí IČO, vyhledej kandidáty přes `ares search` a postupuj podle počtu výsledků.
3. Pro právní ověření spusť detailní příkaz podle původní otázky až po jednoznačném určení IČO.
4. Shrň výsledek česky, věcně a s upozorněním na limity veřejného zdroje.
