# leciva - search filters (CZSUKL)

`leciva-cli search CZSUKL [OPTIONS]` - every filter is optional; combine freely.
With no filters it lists all medicines (respecting `--limit`).

| Flag | Backend field | Meaning |
|---|---|---|
| `--query, -q <text>` | `q` | Fulltext over name + registered name + active substances + ATC name |
| `--kod-sukl <code>` | `kodSukl` | Kód SÚKL, exact match |
| `--atc <code>` | `atc` | ATC code, exact match (e.g. `N02BA51`) |
| `--forma <name>` | `forma` | Pharmaceutical form - resolved Czech name; enumerate with `get leciva://doc/values/forma` |
| `--vydej <name>` | `vydej` | Dispensing category - resolved name; `…/doc/values/vydej` |
| `--stav <name>` | `stavRegistrace` | Registration status - resolved name; `…/doc/values/stav` |
| `--latka <name>` | `latka` | Active substance name (fulltext) - matches Czech/Latin/synonyms |
| `--only-dodavka` | `onlyDodavka` | Only medicines currently on the market (`jeDodavka = true`) |
| `--vcetne-zrusenych` | `vcetneZrusenych` | Include withdrawn/cancelled registrations (default: active only) |
| `--limit <n>` | `limit` | Default 20, max 100 |
| `--offset <n>` | `offset` | Default 0; `offset + limit <= 10000` |
| `--sort <field>` | query param | `relevance` (default) or `date` |
| `--order <dir>` | query param | `asc` or `desc` |

Filter values for `--forma` / `--vydej` / `--stav` are the resolved Czech names,
not raw codes. Discover valid values with `get leciva://doc/values/<field>`
(`forma|vydej|stav|atc|typ|indikace`).

A raw JSON body can be passed as the positional `JSON_PAYLOAD` arg (or `-` for
stdin); it is merged last and overrides flags.
