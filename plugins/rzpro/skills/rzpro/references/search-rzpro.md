# rzpro - search filters

## Devices - `rzpro-cli search CZRZP [OPTIONS]`

| Flag | Backend field | Meaning |
|---|---|---|
| `--query, -q <text>` | `q` | Fulltext over device name + trade name + manufacturer + intended purpose |
| `--evidencni-cislo <n>` | `evidencniCislo` | Evidenční číslo, exact 8-digit id |
| `--druh <kind>` | `druh` | Device kind (Druh ZP) |
| `--riziko <class>` | `miraRizika` | Risk class, exact (e.g. `I`, `IIa`, `III`, `IVDR C`) |
| `--stat <country>` | `statVyrobce` | Manufacturer country |
| `--ico <ico>` | `ico` | Manufacturer/holder IČO, exact |
| `--katalogove-cislo <n>` | `katalogoveCislo` | Catalogue number of a variant - reverse lookup to its device |
| `--expiruje-do <date>` | `expirujeDo` | Only devices with Platnost do on or before the ISO date |
| `--only-active` | `onlyActive` | Only currently valid devices (Stav = Platný) |
| `--limit / --offset` | `limit` / `offset` | Default 20 / 0; `offset + limit <= 10000` |
| `--sort / --order` | query params | `relevance`\|`date` / `asc`\|`desc` |

Risk classes (`--riziko`) come in families tied to the device kind - pick the
right family or the filter returns nothing:
- **General** (Obecný): `I`, `Is`, `Im`, `IIa`, `IIb`, `III`
- **Active implantable** (Aktivní implantabilní): `AI` - not `III`
- **IVD** (Diagnostický): `IVD`, `IVD A`, `IVD B`, `IVDst`; new `IVDR A`-`IVDR D`

`--druh` has three values: "Obecný zdravotnický prostředek", "Diagnostický
zdravotnický prostředek in vitro", "Aktivní implantabilní zdravotnický
prostředek". Enumerate `--riziko` / `--stat` live via `get rzpro://doc/values/riziko`
/ `…/stat`.

## Persons/companies - `rzpro-cli search CZRZP-OS [OPTIONS]`

| Flag | Backend field | Meaning |
|---|---|---|
| `--query, -q <text>` | `nazev` | Fulltext over company/person name |
| `--ico <ico>` | `ico` | IČO, exact |
| `--limit / --offset` | `limit` / `offset` | Default 20 / 0 |

For a single person by key, prefer `get rzpro://osoba/<IČO or Registrační číslo>`.
