---
name: katastr
description: Český katastr nemovitostí (ČÚZK REST API KN). Use for both ad-hoc lookups (parcels, buildings, units, addresses, owners via LV, plomby/řízení signals) and tracked monitoring of cadastral proceedings (vklad/záznam status with automatic change detection). Triggers on "katastr", "parcela", "stavba", "list vlastnictví", "LV", "vklad", "záznam", "plomba", "řízení katastr", "sleduj řízení", "hlídej vklad".
---

# Katastr nemovitostí ČR

A single tool — **`katastr`** — wraps the entire ČÚZK REST API KN plus a stateful tracker for cadastral proceedings.

**IMPORTANT:** The only tool in this skill is `katastr`. Do NOT call `kn`, `cdx-cli`, `cdxctl`, `curl`, or any other tool directly. Assume `katastr` is installed and available in `PATH`.

**IMPORTANT:** If `katastr` outputs an `ERROR:` line, stop immediately and report it to the user. Do not retry blindly or guess workarounds.

## Three namespaces

```bash
katastr tracking <verb> ...   # stateful tracking of proceedings
katastr api <verb> ...        # raw GET requests against ČÚZK
katastr settings <verb> ...   # API key management
```

---

## Decision tree — when to use what

**The user wants raw INFO about a parcel / building / unit / address / owner / LV / plomba?**
→ `katastr api get <PATH>` (one-shot lookup, no state)

**The user wants to START MONITORING a proceeding?** Trigger words: "sleduj", "hlídej", "monitoruj", "informuj mě o změnách", "začni hlídat", "přidej na hlídání".
→ `katastr tracking add <CISLO_RIZENI> [--label "..."]`

**The user asks for a list of THEIR tracked proceedings?**
→ `katastr tracking list`

**The user asks "co je nového u řízení V-XYZ?" or "jaký je stav řízení V-XYZ?":**

Ambiguous — check tracked state first.

1. `katastr tracking list` (or just try `katastr tracking show V-XYZ`)
2. **If tracked** → `katastr tracking show V-XYZ` (rich state with change history). Optionally also `katastr tracking check V-XYZ` to refresh from ČÚZK before showing.
3. **If NOT tracked** → `katastr api get "/api/v1/Rizeni/Vyhledani?TypRizeni=V&Cislo=...&Rok=...&KodPracoviste=..."` to get the current state from ČÚZK, present it, AND offer:
   *"Pokud chceš, můžu řízení přidat ke sledování — budu hlídat změny stavu a operací každý týden."*

**Never add a proceeding to tracking without explicit user intent.** Adding requires "sleduj/hlídej/přidej/začni hlídat" or an affirmative answer to the offer above.

**The user wants to set/check/test API key?**
→ `katastr settings set <KEY>` / `katastr settings show` / `katastr settings test`

**`katastr` reports `ERROR: API klíč ... není nastaven` or `... byl odmítnut`?**
→ Stop. Tell the user to either set the key via `katastr settings set <KEY>` (if they have one) or to obtain one from the ČÚZK customer portal and configure it in the Katastr UI (gear icon in the header).

---

## Examples

**User: "Kdo vlastní parcelu 545 v k.ú. 638790?"**
```bash
katastr api get "/api/v1/Parcely/Vyhledani?KodKatastralnihoUzemi=638790&TypParcely=PKN&DruhCislovaniParcely=2&KmenoveCisloParcely=545"
# → take data[0].id, then:
katastr api get "/api/v1/Parcely/<ID>"
# Present owner info from LV cross-reference. Don't track (parcels can't be tracked anyway).
```

**User: "Sleduj řízení V-123/2026-701 jako Novákovi"**
```bash
katastr tracking add V-123/2026-701 --label "Novákovi"
# → Confirm with stav and operace count.
```

**User: "Co je nového u V-123/2026-701?"**
```bash
katastr tracking show V-123/2026-701   # if tracked: rich state + changes
# OR
katastr api get "/api/v1/Rizeni/Vyhledani?TypRizeni=V&Cislo=123&Rok=2026&KodPracoviste=701"
# if not tracked → present + offer to add
```

**User: "Označ všechny změny u V-123/... za přečtené"**
```bash
katastr tracking confirm V-123/2026-701 --all
```

**User: "Přejmenuj V-123/2026-701 na Novákovi" / "změň označení V-123/... na ..."**
```bash
katastr tracking set-label V-123/2026-701 "Novákovi"
# To clear: katastr tracking set-label V-123/2026-701 ""
```

**User: "Zkontroluj všechna moje řízení teď"**
```bash
katastr tracking check
```

**User: "Nastav mi API klíč ABCD1234..."**
```bash
katastr settings set ABCD1234...
# Validates against ČÚZK before saving.
```

---

## `katastr tracking` — proceeding monitoring

Stateful tracker. State is stored in `~/.cdx/apps/katastr/rizeni/`. A single central cron automation (`0 8 * * 1`) runs `katastr tracking check` every Monday 8:00 to refresh all tracked proceedings.

```bash
# Start tracking — verifies the proceeding exists in ČÚZK and saves baseline
katastr tracking add V-123/2026-701
katastr tracking add V-123/2026-701 --label "Novákovi"

# List all tracked
katastr tracking list

# Show full state of a tracked proceeding (JSON)
katastr tracking show V-123/2026-701

# Check for changes (call ČÚZK and diff against stored state)
katastr tracking check                  # all tracked proceedings
katastr tracking check V-123/2026-701   # one specific

# Mark detected changes as read
katastr tracking confirm V-123/2026-701              # all unread
katastr tracking confirm V-123/2026-701 --all        # explicit
katastr tracking confirm V-123/2026-701 --change 0   # specific by index

# Set or change the user-friendly label of a tracked proceeding
katastr tracking set-label V-123/2026-701 "Novákovi"
katastr tracking set-label V-123/2026-701 ""    # clear

# Stop tracking
katastr tracking remove V-123/2026-701
```

### Proceeding number format

`TYPE-NUMBER/YEAR-WORKPLACE`, e.g.:
- `V-123/2026-701` — Vklad (deposit / right transfer)
- `Z-100/2026-701` — Záznam (record)
- Supported types: `V`, `Z`, `PGP`, `PD`, `ZPV`

### What changes are detected

- **stav** — overall status string (e.g. "Probíhá zpracování" → "Provedení vkladu")
- **provedené operace** — new operations appended to the timeline (Založení řízení, Zaplombování, Informace o vyznačení plomby, Rozhodnutí o povolení vkladu, Provedení vkladu, Vyrozumění o provedení vkladu, Ukončení řízení, ...)
- **stav úhrady** — `U` Uhrazeno, `N` Neuhrazeno, `O` Osvobozeno, `null` neuvedeno

---

## `katastr api` — raw ČÚZK REST API access

The ČÚZK REST API KN base is `https://api-kn.cuzk.gov.cz`. The API is GET-only and returns JSON. The `katastr api get` command handles auth, errors and JSON output.

```bash
katastr api get "<PATH>"
```

### Response envelope

Most endpoints return:
- `data`: payload (list or object)
- `aktualnostDatK`: data freshness timestamp
- `provedenoVolani`: counter of calls
- `zpravy`: optional messages

### Sanity checks

```bash
katastr api get "/api/v1/AplikacniSluzby/Health"          # public, doesn't validate API key
katastr api get "/api/v1/AplikacniSluzby/StavUctu"        # validates the key
katastr api get "/api/v1/AplikacniSluzby/AktualnostDat"
katastr api get "/api/v1/AplikacniSluzby/ProvozniInformace"
```

### Known enums

- `TypParcely`: `PKN` (parcel in KN), `PZE` (simplified evidence)
- `DruhCislovaniParcely`: `1` (stavebni parcela), `2` (pozemkova parcela)
- `TypStavby` for searching: `1` (cislo popisne), `2` (cislo evidencni)
- `TypRizeni`: `V`, `Z`, `PGP`, `PD`, `ZPV`

Official Swagger UI: `https://api-kn.cuzk.gov.cz/swagger/`

### Workflows

This API does not provide personal data (owners etc.) directly. For owners / full LV extracts, the official channel is DP/WSDP. The API is useful for identification, linking (address ↔ building ↔ parcels ↔ LV number), basic attributes, and basic signals (plomby/řízení lists if present).

#### "Is it clean?" — quick check for a parcel

"Čisté" usually means one of two things, so clarify with the user:
- `no_plomby`: no pending proceedings (plomby/řízení) on parcel/building/unit in this API
- `no_rights_limits`: no liens/easements/rights restrictions (NOT available via this API; needs an official LV extract)

Minimal `no_plomby` workflow:
1. Resolve parcel internal ID via `Parcely/Vyhledani` (include `PoddeleniCislaParcely` for parcel numbers like `2642/9`).
2. Fetch `Parcely/{id}` and check:
   - `rizeniPlomby` (empty list = "no plomby" signal)
   - `zpusobyOchrany` (territorial/protection limit, not a plomba)
   - `lv` (cross-check LV number)
   - `stavba.id` if present → fetch `Stavby/{id}` and check `rizeniPlomby` there too
3. Optional: `LV/{lv.id}` for `rizeniPlomby` + counts of linked parcels/buildings/units.

#### Identify a parcel by (k.u. code + parcel number)

Inputs:
- `KodKatastralnihoUzemi` (e.g. 638790)
- `KmenoveCisloParcely` (e.g. 545)
- `PoddeleniCislaParcely` (optional; for `2642/9` use `KmenoveCisloParcely=2642` + `PoddeleniCislaParcely=9`)
- `DruhCislovaniParcely` (1/2 — try both if unsure)
- `TypParcely` usually `PKN`

```bash
katastr api get "/api/v1/Parcely/Vyhledani?KodKatastralnihoUzemi=638790&TypParcely=PKN&DruhCislovaniParcely=2&KmenoveCisloParcely=545"
katastr api get "/api/v1/Parcely/<ID>"
```

What to read from parcel detail:
- `vymera`, `druhPozemku`, `zpusobVyuziti`
- `lv.cislo` (+ `lv.katastralniUzemi`)
- `stavba.id` (if a numbered building is linked)
- `definicniBod` (S-JTSK) for spatial queries
- `rizeniPlomby` (if not empty, there is pending activity)

#### LV detail (limited)

If you have `lv.id`:

```bash
katastr api get "/api/v1/LV/<LV_ID>"
```

Available: `rizeniPlomby` at LV level, lists/counts of `parcely`, `stavby`, `jednotky`.
Not available: owners and full rights/restrictions sections.

#### Neighbors & spatial queries

```bash
katastr api get "/api/v1/Parcely/SousedniParcely/<PARCEL_ID>"
```

Polygon search (EPSG:5514 / EPSG:5513, meters):
```bash
katastr api get "/api/v1/Parcely/Polygon?SeznamSouradnic=%5B%7B%22x%22%3A-494110.17%2C%22y%22%3A-1116432.13%7D%2C...%5D"
```

Trick: read `definicniBod` from `Parcely/{id}` or `Stavby/{id}` and build a square polygon around it (±25/50/100 m) to get "okolí".

#### Identify building by postal address (RUIAN)

REST API KN does not accept free-text addresses. Use the RUIAN address point code:

1. Resolve address via VDP RUIAN fulltext (separate service, NOT via `katastr` — use `curl` here as an exception):
   ```bash
   curl -fsS \
     -H "X-Requested-With: XMLHttpRequest" \
     -H "Accept: application/json" \
     --get "https://vdp.cuzk.gov.cz/vdp/ruian/adresnimista/fulltext" \
     --data-urlencode "adresa=Mala Strana 66, Hladke Zivotice"
   ```
2. Take `polozky[0].kod`, then:
   ```bash
   katastr api get "/api/v1/Stavby/AdresniMisto/<RUIAN_KOD>"
   ```

Returns building info with `parcely[]`, `lv.cislo`, `definicniBod`, `jednotky`. To fetch full building detail:
```bash
katastr api get "/api/v1/Stavby/<STAVBA_ID>"
```

#### Units (apartments / non-residential)

```bash
katastr api get "/api/v1/Jednotky/Vyhledani?KodCastiObce=<KOD>&TypStavby=1&CisloDomovni=<CP>&CisloJednotky=<CJ>"
katastr api get "/api/v1/Jednotky/<JEDNOTKA_ID>"
```

#### Proceedings (basic signal, not full legal extract)

By proceeding ID:
```bash
katastr api get "/api/v1/Rizeni/<RIZENI_ID>"
```

By official identifier (for one-shot lookup of NOT-tracked proceedings; use `katastr tracking show` for tracked ones):
```bash
katastr api get "/api/v1/Rizeni/Vyhledani?TypRizeni=V&Cislo=<CISLO>&Rok=<ROK>&KodPracoviste=<KOD>"
katastr api get "/api/v1/Rizeni/PrijateDne?TypRizeni=V&KodPracoviste=<KOD>&DatumPrijeti=2026-02-13"
```

#### Code lists (decode codes for reports)

```bash
katastr api get "/api/v1/CiselnikyUzemnichJednotek/Obce"
katastr api get "/api/v1/CiselnikyUzemnichJednotek/KatastralniUzemi"
katastr api get "/api/v1/CiselnikyUzemnichJednotek/CastiObci"
katastr api get "/api/v1/CiselnikyISKN/DruhyPozemku"
katastr api get "/api/v1/CiselnikyISKN/TypyStavby"
katastr api get "/api/v1/CiselnikyISKN/TypyJednotky"
katastr api get "/api/v1/CiselnikyISKN/ZpusobyVyuzitiStavby"
katastr api get "/api/v1/CiselnikyISKN/ZpusobyVyuzitiParcely"
katastr api get "/api/v1/CiselnikyISKN/ZpusobyVyuzitiJednotky"
katastr api get "/api/v1/CiselnikyISKN/ZpusobyOchrany"
katastr api get "/api/v1/CiselnikyISKN/Pracoviste"
```

---

## `katastr settings` — API key management

```bash
katastr settings show         # show configured key (masked) or "not configured"
katastr settings set <KEY>    # validate against ČÚZK and persist
katastr settings test         # test current key against ČÚZK
```

The key is obtained by the user from the ČÚZK customer portal (free, requires Identita občana or remote-access account). It's stored in `~/.cdx/apps/katastr/.env`. The same file is read by all `katastr` subcommands and by the UI component.

If the user reports problems with `katastr` calls returning "API klíč ... není nastaven" or "... byl odmítnut":
1. `katastr settings test` to diagnose
2. If 401/403 → ask for a new key and `katastr settings set <NEW_KEY>`
3. If network error → tell user ČÚZK API is unreachable, retry later

---

## UI component

The user-facing UI is at route `/katastr` (Doplňky → Katastr). It shows the table of tracked proceedings, allows adding/removing/refreshing them, and exposes API key management via the gear icon. When you (AI) make tracking changes through `katastr tracking add/remove/confirm`, they appear in the UI on the next refresh — no extra step needed.
