# SKVS Search Filter Values

Slovak general court decisions (okresne, krajske, etc.). ~4.6M decisions.

## query
Free-text search on decision text. Optional.

## court
Court code (exact match):
- `OSBA1` - Okresny sud Bratislava I
- `OSBA2` - Okresny sud Bratislava II
- `OSBA3` - Okresny sud Bratislava III
- `OSBA4` - Okresny sud Bratislava IV
- `OSBA5` - Okresny sud Bratislava V
- `KSBA` - Krajsky sud v Bratislave
- And many more (use courtName for full-text match)

## Citation court abbreviations
Court abbreviation used in the user-facing citation link text (`SÚD - SP. ZN. - DD.MM.RRRR`).
These match the source titles the API emits — when a resolved source title is shown, reuse it
verbatim. The pattern set covers the complete slov-lex court list (68 courts):

| `courtName` value | Abbrev |
|-------------------|--------|
| Okresný súd `<mesto>` | OS `<mesto>` |
| Mestský súd `<mesto>` | MS `<mesto>` |
| Krajský súd v `<mesto-lokál>` | KS `<mesto-nominatív>` |
| Správny súd v `<mesto-lokál>` | SS `<mesto-nominatív>` |
| Najvyšší súd Slovenskej republiky | NS SR |
| Najvyšší správny súd Slovenskej republiky | NSS SR |
| Špecializovaný trestný súd | ŠTS |

OS/MS keep the city verbatim as it appears in the name (`OS Košice okolie`, `MS Bratislava IV`).
KS/SS names carry the seat city in the locative ("v Bratislave") — the citation uses the
nominative (all 8 seat cities):

| Locative (in `courtName`) | Nominative (in citation) |
|---|---|
| v Banskej Bystrici | Banská Bystrica |
| v Bratislave | Bratislava |
| v Košiciach | Košice |
| v Nitre | Nitra |
| v Prešove | Prešov |
| v Trenčíne | Trenčín |
| v Trnave | Trnava |
| v Žiline | Žilina |

Unknown court → keep its official short name verbatim.

## courtName
Full court name (exact match). Examples:
- `Okresny sud Bratislava I`
- `Krajsky sud v Bratislave`

## judge
Judge name (exact match). Example: `JUDr. Novak`

## spisovaZnacka
Case file number (exact match on keyword). Example: `1C/123/2024`

## decisionForm
Decision form:
- `Rozsudok` - Judgment
- `Uznesenie` - Resolution
- `Platobny rozkaz` - Payment order
- `Zmenkovy platobny rozkaz` - Bill of exchange payment order

## decisionNature
Decision nature:
- `Prvostupnove nenapadnute opravnymi prostriedkami` - First-instance not challenged
- `Odvolacie` - Appellate
- `Dovolacie` - Cassation

## ecli
ECLI identifier (exact match). Example: `ECLI:SK:OSBA1:2024:1234567890`

## dateFrom / dateTo
Date range (YYYY-MM-DD). Filters on decision date (inclusive).

## sort (query param)
- `relevance` (default)
- `title`
- `date`

## order (query param)
- `asc`
- `desc` (default)

## offset
Integer, 0-based. Default: 0.

## limit
Integer, 1-100. Default: 20.
