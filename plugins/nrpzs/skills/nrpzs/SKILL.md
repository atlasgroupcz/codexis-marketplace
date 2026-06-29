---
uuid: 7c1e2a90-5b34-4d21-9f08-3a6e2d4c8b15
name: nrpzs
icon: icon.svg
jurisdictions: [CZ]
description: Národní registr poskytovatelů zdravotních služeb (NRPZS, ÚZIS). Use for Czech healthcare-provider lookups by IČO, name, or obor péče — provider profile, places of care (adresa, kontakt, GPS, datová schránka), departments and office hours. Triggers on "NRPZS", "poskytovatel zdravotních služeb", "zdravotnické zařízení", "ordinace", "najdi lékaře/nemocnici", "obor péče", "ÚZIS", and IČO + zdravotnictví.
version: 1.0.0
i18n:
  cs:
    displayName: "NRPZS — poskytovatelé zdravotních služeb"
    summary: "Vyhledávání poskytovatelů zdravotní péče (NRPZS/ÚZIS) podle IČO, názvu nebo oboru péče."
  en:
    displayName: "NRPZS — Healthcare Providers"
    summary: "Look up Czech healthcare providers (NRPZS/ÚZIS) by ICO, name or specialty."
  sk:
    displayName: "NRPZS — poskytovatelia zdravotných služieb"
    summary: "Vyhľadávanie poskytovateľov zdravotnej starostlivosti (NRPZS/ÚZIS) podľa IČO, názvu alebo odboru."
---

# NRPZS — registr poskytovatelů zdravotních služeb

A single tool — **`nrpzs-cli`** — wraps the public NRPZS REST API
(`https://nrpzs-api.uzis.cz/api/v1`). No API key. Read-only.

**IMPORTANT:** The only tool in this skill is `nrpzs-cli`. Do NOT call `curl` or any
other tool directly. Assume `nrpzs-cli` is installed and available in `PATH`.

**IMPORTANT:** If `nrpzs-cli` prints an `ERROR:` line, stop and report it to the
user. Do NOT retry blindly.

## Output

`nrpzs-cli` prints **JSON to stdout** (verbatim API responses; `profil` and
region-filtered `search` are shaped objects/arrays). Parse with `jq`, never `sed`/`grep`.
Code lists return names inline (e.g. `OborPece: "urologie"`) — you usually don't need
to translate codes. `PravniFormaKod` is returned as a code (e.g. 112 = s.r.o.).

## Commands

```bash
nrpzs-cli search --ico 27085031              # all places of a provider
nrpzs-cli search --nazev "Nemocnice Příbram" # by name (min 3 chars)
nrpzs-cli search --obor urologie             # by obor péče (name or code, e.g. L42)
nrpzs-cli search --obor urologie --kraj "Středočeský"   # narrow by kraj (local filter)
nrpzs-cli search --nazev "Nemocnice" --level zarizeni   # facility level (ico/nazev only)
nrpzs-cli detail <MistoPoskytovaniId>        # full record of a place of care
nrpzs-cli oddeleni <ZdravotnickeZarizeniId>  # departments
nrpzs-cli ordinacni-doba <ZdravotnickeZarizeniId>
nrpzs-cli profil 27085031                    # aggregated provider profile
nrpzs-cli ciselnik obory-pece                # code lists (obory/formy/druhy péče, druhy zařízení, …)
nrpzs-cli api get "/version"                 # raw GET escape hatch
```

## Notes

- `--kraj`/`--okres` filter **locally** (the API has no region parameter), so they
  work only together with `--obor` or `--nazev`, and only with `--level mista` (default).
- IČO accepts 1–8 digits (zero-padded automatically).
- For the full IDs to pass to `detail`/`oddeleni`, read `MistoPoskytovaniId` /
  `ZdravotnickeZarizeniId` from `search` results.
