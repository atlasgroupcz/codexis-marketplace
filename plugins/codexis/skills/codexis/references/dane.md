# Daně — rozcestník + procesní vrstva

## Scope

Používej, když uživatel řeší daňovou otázku a nevíš, do kterého dílčího daňového režimu patří, nebo když potřebuješ procesní vrstvu společnou pro všechny daně (podání tvrzení, lhůty, přeplatek, stížnost).

Pro konkrétní daňové dílčí režimy odkaž na sub-skill:

- **Daň z příjmů fyzických osob (DPFO)** → `dane-prijmy-fo.md` (dílčí základy, odpočty, slevy, sazba, evidence)
- Ostatní daňové sub-skilly případně doplň, až vzniknou (DPPO, DPH, daň z nemovitých věcí, spotřební, silniční).

## Procesní vrstva — obecná (daňový řád)

Kdykoli má odpověď praktický procesní rozměr (ať je hmotněprávní daň jakákoli), projdi krátce tyto úrovně:

1. **Povinnost podat** tvrzení — zda hmotněprávní zákon ukládá podací povinnost a za jakých podmínek.
2. **Fakultativní podání** — podle § 135 daňového řádu lze řádné daňové tvrzení podat i tehdy, když hmotněprávní povinnost nevznikla. Má smysl zmínit vždy, když má ekonomický efekt (vratka, uplatnění nároků).
3. **Lhůty pro podání** — § 136 daňového řádu (základní lhůta, prodloužení, elektronická forma).
4. **Přeplatek a jeho vrácení** — § 154 daňového řádu (co je přeplatek, započtení) a § 155 (žádost o vrácení, lhůty pro vydání).

Když odpověď končí závěrem „povinnost podat nevzniká", nezastav se tam. Připoj krátký dodatek: „Přiznání lze podat i dobrovolně podle § 135 daňového řádu, pokud by mohlo přinést uplatnění ročních nároků nebo vrácení přeplatku podle § 154–155 daňového řádu." Konkrétní výhodnost plyne z hmotněprávních nároků dané daně (u DPFO typicky § 15 / § 35ba / § 35c ZDP — viz `dane-prijmy-fo.md`).

## Upřesňující otázky (když je dotaz nekonkrétní)

- „Chcete vědět jen to, zda máte povinnost podat přiznání, nebo i to, zda má smysl ho podat, i když povinnost nevznikne?" — otevře fakultativní větev.
- „Máte za rok něco, co se uplatňuje až v přiznání nebo co by mohlo vést k vratce — typicky slevy, nezdanitelné části nebo zaplacené zálohy?" — opens přeplatek/nároky.
- „Které daně se vás konkrétně týkají — příjmy FO, příjmy PO, DPH, nemovitosti, spotřební?" — otevře volbu sub-skillu.

## Mapa zákonů (procesních)

- **Zákon č. 280/2009 Sb., daňový řád (DŘ)** — společný procesní rámec pro všechny daně
  - § 135 — řádné daňové tvrzení (včetně fakultativního podání)
  - § 136 — lhůty pro podání tvrzení
  - § 154 — přeplatek (vznik, započtení)
  - § 155 — žádost o vrácení přeplatku, lhůty pro vydání

Při citaci statutární lhůty zachovej operativní kvalifikátory („nejpozději", „do", „po uplynutí") — jejich vypuštění mění právní význam.

## cdx-cli tipy

```bash
cdx-cli get cdx://cz_law/280/2009/versions                    # verze daňového řádu
cdx-cli get 'cdx://doc/<verze>/text?part=paragraf135'         # řádné tvrzení
cdx-cli get 'cdx://doc/<verze>/text?part=paragraf154'         # přeplatek
```

Pro rozhodné datum vyber verzi DŘ platnou k datu dotazu. Nepoužívej broad search, když víš zákon a paragraf.

## Pasti (anti-patterny)

- **Odpovědět binárně „povinnost je / není"** a přehlédnout, že procesně lze podat i dobrovolně a to má často ekonomický smysl.
- **Slučovat roviny** — „kdo je subjekt / kdy se posuzuje / kdo je povinen jednat / do kdy" rozlišuj, neslévej je do jedné věty.
- **Nepřepínat mezi hmotným a procesním zákonem** — hmotný zákon (např. ZDP) určuje co se zdaňuje; procesní zákon (DŘ) určuje jak se tvrdí, v jaké lhůtě a jak vracet přeplatek. Kompletní odpověď propojí obojí.
