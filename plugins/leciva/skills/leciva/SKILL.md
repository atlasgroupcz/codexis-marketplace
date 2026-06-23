---
uuid: 3f8b1d6a-2c47-4e90-b15a-9d72e0c4a8f1
name: leciva
description: Registr léčiv SÚKL (DLP). Use for Czech medicinal product lookups by drug name or SÚKL code — strength, dosage form, packaging, active substances, ATC group, dispensing type. Triggers on "lék", "léčivo", "léčivý přípravek", "SÚKL", "kód SÚKL", "účinná látka", "ATC", "příbalový leták", "SPC", "registrovaný lék", "najdi lék", "složení léku".
version: 1.1.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "SÚKL — registr léčiv"
    summary: "Vyhledávání léků v databázi SÚKL — podle názvu nebo kódu SÚKL, účinné látky, ATC."
  en:
    displayName: "SÚKL — Medicines Registry"
    summary: "Look up medicines in the SÚKL database — by name or SÚKL code, active substances, ATC."
  sk:
    displayName: "SÚKL — register liekov"
    summary: "Vyhľadávanie liekov v databáze SÚKL — podľa názvu alebo kódu SÚKL, účinnej látky, ATC."
---

# SÚKL — registr léčiv

Jediný nástroj — **`leciva-cli`** — obaluje veřejné SÚKL DLP REST API
(`https://prehledy.sukl.cz/dlp/v1`) a lokální vyhledávací index.

**IMPORTANT:** Jediný nástroj je `leciva-cli`. Nevolej `curl` ani jiné nástroje.
Předpokládej, že `leciva-cli` je nainstalovaný v `PATH`.

**IMPORTANT:** Když `leciva-cli` vypíše `ERROR:` (např. `HTTP 404 …`), zastav se a
nahlas to uživateli. Nezkoušej slepě znovu.

## Klíče

SÚKL API neumí hledat podle názvu — má jen `kód → detail`. Proto:
1. název → kód: `leciva-cli search "<text>"` (lokální index)
2. kód → vše: `leciva-cli detail <kod>`, `leciva-cli slozeni <kod>`

Index se stáhne sám při prvním použití. Žádný API klíč není potřeba.

## Příkazy

```bash
leciva-cli search "paralen"        # hledání podle názvu → [{kodSukl, nazev, sila, doplnek}]
leciva-cli detail 0182362          # čitelný detail (síla, forma, výdej, ATC, účinné látky, jeDodavka) — přeložené názvy
leciva-cli detail 0182362 --all    # plný záznam (všechna pole) + dopsané názvy
leciva-cli slozeni 0182362         # účinné látky (názvy) + množství
leciva-cli latka "ibuprofen"       # léky obsahující danou účinnou látku (matchuje i anglicky, latinsky, synonyma)
leciva-cli ceny 0182362            # cena + úhrada pojišťovny + doplatek pacienta
leciva-cli dokumenty 0182362       # odkazy na příbalový leták (PIL) a SPC
leciva-cli refresh                 # přestavět index (běží i jako měsíční automatizace)
leciva-cli index status            # verze/stáří indexu
```

Toto je celé rozhraní. Nevolej `curl` ani jiné nástroje a **nehádej DLP endpointy** —
když potřebuješ víc polí, použij `detail --all`, ne ruční dotaz na API.

## Postup

- Lék podle názvu: `search "<text>"` → vyber kód → `detail`/`slozeni`.
- Kód SÚKL přímo: rovnou `detail`/`slozeni`.
- „Co obsahuje látku X / jaké léky mají Y": `latka "<látka>"`.

## Výstup

Příkazy vracejí čistý JSON s **názvy** (forma, cesta, výdej, účinné látky), ne syrové kódy.
Uživateli ho podej **čitelně** — lék, síla, forma, výdej (na předpis / volně prodejné),
účinné látky — **nedumpuj syrový JSON**. Když chceš úplně všechna pole, je tu `detail --all`.

Pole **`jeDodavka`** v detailu = lék byl dodáván do lékáren v posledních 6 měsících → prakticky
**je k dostání**; `false` = registrovaný, ale fakticky se nedodává. Podej to uživateli jako
„na trhu / k dostání: ano/ne".
