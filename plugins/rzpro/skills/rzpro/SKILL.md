---
uuid: 9a4c1f72-6b8e-4d03-a5f1-2c7b0e9d6a48
name: rzpro
icon: icon.svg
description: Registr zdravotnických prostředků (RZPRO, KSRZIS). Use for Czech medical device lookups by device name or evidence number — manufacturer, device kind, risk class, intended purpose, notified body, variants/catalogue numbers, and registered persons and their activities. Triggers on "zdravotnický prostředek", "RZPRO", "evidenční číslo", "notifikovaný prostředek", "výrobce zdravotnických prostředků", "registr ZP", "najdi prostředek", "diagnostický prostředek", "IVD".
version: 1.0.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "RZPRO — registr zdravotnických prostředků"
    summary: "Vyhledávání zdravotnických prostředků v registru RZPRO — podle názvu nebo evidenčního čísla, výrobce, varianty, osoby."
  en:
    displayName: "RZPRO — Medical Devices Registry"
    summary: "Look up medical devices in the RZPRO registry — by name or evidence number, manufacturer, variants, persons."
  sk:
    displayName: "RZPRO — register zdravotníckych pomôcok"
    summary: "Vyhľadávanie zdravotníckych pomôcok v registri RZPRO — podľa názvu alebo evidenčného čísla, výrobca, varianty, osoby."
---

# RZPRO — registr zdravotnických prostředků

A single tool — **`rzpro-cli`** — wraps the public KSRZIS RZPRO OpenData datasets
(`https://eregpublicsecure.ksrzis.cz/Registr/RZPRO/OpenData`) behind a local search
index built from the downloaded CSV files.

**IMPORTANT:** The only tool in this skill is `rzpro-cli`. Do NOT call `curl` or any
other tool. Assume `rzpro-cli` is installed and available in `PATH`.

**IMPORTANT:** When `rzpro-cli` prints `ERROR:` (e.g. `no device with Evidenční číslo …`),
stop and report it to the user. Do NOT blindly retry.

## Keys

The registry is keyed by **Evidenční číslo** — an 8-digit device id (e.g. `00000019`).
You can pass it with or without leading zeros (`19` == `00000019`).
Persons (manufacturers, distributors) are keyed by **Registrační číslo** or **IČ**.

The datasets download themselves on first use. No API key needed.

## Commands

```bash
rzpro-cli search "gliadin"        # by name / trade name / manufacturer → {count, shown, results:[{evidencniCislo, nazev, obchodniNazev, vyrobce, druh, miraRizika, statVyrobce, stav}]}
rzpro-cli search "rouška sterilní" # all words must match (order-independent, diacritics-insensitive)
rzpro-cli search --druh "in vitro" --riziko III --stat Německo   # filters; text optional
rzpro-cli search --expiruje-do 2027-01-01   # devices whose validity ends by a date
rzpro-cli search "rouška" --limit 5         # cap results (total count still reported)
rzpro-cli values riziko           # distinct values of a filter column (druh|riziko|stat) + counts
rzpro-cli detail 00000019         # full device record (druh, míra rizika, určený účel, notif. osoba, výrobce, platnost …)
rzpro-cli detail 19               # leading zeros optional
rzpro-cli varianty 00000019       # variants / catalogue numbers of the device
rzpro-cli varianta G0068486       # reverse: which device a catalogue number belongs to
rzpro-cli osoby 00000019          # device's manufacturer-of-record (vyrobce, always) + linked Czech-registered persons (often empty for foreign makers)
rzpro-cli osoba 002276            # a registered person + its activities + devices it notified (by Registrační číslo or IČ)
rzpro-cli refresh                 # download datasets if a newer publication exists (also runs daily as automation)
rzpro-cli refresh --force         # re-download unconditionally
rzpro-cli index status            # local publication date (datum) + row counts
```

Filters for `search`: `--druh` (device kind, substring e.g. "in vitro", "implantabilní"),
`--riziko` (risk class, exact e.g. `I`, `IIa`, `III`, `IVDR C`), `--stat` (manufacturer
country, substring e.g. "Německo"), `--expiruje-do` (ISO date — devices whose `Platnost do`
is on or before it). Combine freely; with no text they list all matches. `--limit` caps the
result list (default 50; the total `count` is still reported).

`--druh` has only three values: "Obecný zdravotnický prostředek", "Diagnostický zdravotnický
prostředek in vitro", "Aktivní implantabilní zdravotnický prostředek". For the exact
`--riziko` / `--stat` values run `rzpro-cli values riziko` / `values stat` — read live from
the data (risk classes are EU codes like `IIa`, `III`, `IVDR C`, not guessable).

Risk classes (`--riziko`) come in three families tied to the device kind — pick the right
family or the filter returns nothing:
- **General device** (Obecný): `I`, `Is`, `Im`, `IIa`, `IIb`, `III`
- **Active implantable** (Aktivní implantabilní): `AI` — *not* `III` (so "implantable class III"
  yields 0; active implantables are `AI`)
- **IVD** (Diagnostický): legacy `IVD`, `IVD A`, `IVD B`, `IVDst`; new `IVDR A`–`IVDR D`

Every lookup result carries a `datum` field — the publication date of the local data,
for citing "RZPRO as of <datum>".

The JSON output is a machine intermediate — process it with **`jq`** (pull only the fields
you need; for batches across many evidence numbers, loop and parse with `jq`, not by hand).

This is the whole interface. Do NOT call `curl` or guess KSRZIS endpoints.

## Workflow

- Device by name: `search "<text>"` → pick the `evidencniCislo` → `detail` / `varianty` /
  `osoby`. Words match the device name, trade name and manufacturer together.
- Device by evidence number: `detail <evid>` directly.
- Who makes / distributes a device: `osoby <evid>` — the `vyrobce` field is the
  manufacturer-of-record (always present, incl. foreign makers); the `osoby` list is
  the linked **Czech-registered** persons (`vlastnik: true` is the notifier with edit
  rights). The list is empty for foreign manufacturers not registered in CZ — use
  `vyrobce` then.
- A company's activities and devices: `osoba <regČíslo | IČ>`.

## Data & freshness

Source is the KSRZIS RZPRO OpenData portal. Data is republished **daily** (every
morning). `refresh` checks the page's "Datum aktualizace" and only downloads when it
advanced, so repeated runs are cheap no-ops. The `index status` `datum` field is the
publication date of the local copy.

v1 covers 4 datasets: devices (`RZPRO_ZP`), variants (`RZPRO_ZP_VAR`), device↔person
links (`RZPRO_ZP_OS`), and person activities (`RZPRO_OS_CIN`).
