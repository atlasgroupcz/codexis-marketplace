# Léčiva — čitelný výstup (resolved + kurátorovaný) — návrh

- **Navazuje na:** plugin `leciva` v1.0.0 (LM-117), branch `lm-117-leciva`
- **Datum:** 2026-06-23
- **Status:** návrh ke schválení (v1.1)

## 1. Problém

`detail` a `slozeni` teď vracejí syrovou odpověď SÚKL API, kde je spousta věcí
jako **kódy** (`lekovaFormaKod: "TBL FLM"`, `zpusobVydejeKod: "F"`,
`leciveLatky: [1593]`, `drzitelKod: "OHCCZ"`) a hromada **interního šumu**
(`jeDodavka`, `povinneVzorky`, DDD, braillovo písmo, MRP…). AI to pak vypíše
ošklivě („účinná látka: kód 1593", „jeDodavka: true"). Pro běžného uživatele
nepoužitelné.

## 2. Use case

Čitelné vyhledávání registrovaných léků v ČR (SÚKL DLP) pro běžné lidi i profíky.
Reálné dotazy:
- „najdi lék X" → kód SÚKL + varianty balení (`search`)
- „co je v tom léku" → účinné látky + množství (`slozeni`)
- „detail léku" → síla, forma, balení, ATC, **výdej (Rx/OTC)**, držitel, stav registrace (`detail`)
- **„jaké léky obsahují látku Y"** → hledání podle účinné látky (`latka`, nové)

## 3. Řešení (přehled)

Vše lokálně, **bez API volání navíc** — číselníky i složení jsou ve stejném
měsíčním dumpu, co už stahujeme:

1. `refresh` navíc vytáhne z dumpu **číselníky** (kód → název) a postaví
   **reverzní index látka → léky**.
2. `detail` přeloží kódy na názvy a vrátí **kurátorovaný** objekt (jen lidská
   pole); `--raw` dá plnou syrovou odpověď.
3. `slozeni` přidá ke každé položce **název látky**.
4. nový `latka "<text>"` → léky obsahující danou účinnou látku.

## 4. Číselníky (ověřeno v dumpu)

Vše v `DLP….zip`, oddělovač `;`, kódování cp1250, join podle prvního sloupce:

| Zdroj CSV | Klíč → název | Použití |
|---|---|---|
| `dlp_latky.csv` | `KOD_LATKY` → `NAZEV` (sl. 5; fallback `NAZEV_INN`) | účinné látky (PARACETAMOL, IBUPROFEN) |
| `dlp_formy.csv` | `FORMA` → `NAZEV` | „TBL FLM" → „Potahovaná tableta" |
| `dlp_cesty.csv` | `CESTA` → `NAZEV` | „POR" → „Perorální podání" |
| `dlp_vydej.csv` | `VYDEJ` → `NAZEV` | „F" → „volně prodejné léčivé přípravky", „R" → „na lékařský předpis" |
| `dlp_stavyreg.csv` | `REG` → `NAZEV` | „R" → „registrovaný léčivý přípravek" |
| `dlp_organizace.csv` | `ZKR_ORG` → `NAZEV` | držitel: „OHCCZ" → název firmy |
| `dlp_atc.csv` | `ATC` → `NAZEV` | ATC skupina |
| `dlp_zavislost.csv` | kód → název | návykovost (jen když vyplněno) |
| `dlp_slozeni.csv` | `KOD_SUKL` → `KOD_LATKY` | reverzní index látka→léky |

## 5. Změny v `refresh`

Vedle `index.tsv` (beze změny) navíc postavit do `~/.cdx/apps/leciva/`:

```
cis/latky.tsv          # KOD_LATKY \t NAZEV
cis/formy.tsv          # FORMA \t NAZEV
cis/cesty.tsv          # CESTA \t NAZEV
cis/vydej.tsv          # VYDEJ \t NAZEV
cis/stavyreg.tsv       # REG \t NAZEV
cis/organizace.tsv     # ZKR_ORG \t NAZEV
cis/atc.tsv            # ATC \t NAZEV
cis/zavislost.tsv      # kód \t NAZEV
latka-drugs.tsv        # KOD_LATKY \t KOD_SUKL  (z dlp_slozeni.csv; po řádcích)
```

Vše překódovat cp1250→UTF-8. Atomicky jako dosud (temp → rename; manifest až
nakonec). `manifest.json` dostane navíc orientační počty (např. `cisLoaded: true`).

## 6. `detail <kod>` — kurátorovaný výstup

Default = čistý objekt (kódy přeložené, kódy zachované u ATC):

```json
{
  "kodSukl": "0254299",
  "nazev": "IBALGIN 400MG TBL FLM 24",
  "sila": "400MG",
  "forma": "Potahovaná tableta",
  "baleni": "24",
  "cestaPodani": "Perorální podání",
  "atc": { "kod": "M01AE01", "nazev": "ibuprofen" },
  "vydej": "volně prodejné léčivé přípravky",
  "stavRegistrace": "registrovaný léčivý přípravek",
  "drzitel": "Sanofi-Aventis, s.r.o.",
  "registracniCislo": "29/154/88-C",
  "ucinneLatky": ["IBUPROFEN"]
}
```

**Nechat** (10–12 lidských polí): kodSukl, nazev, sila, doplnek, baleni, forma,
cestaPodani, atc{kod,nazev}, vydej, stavRegistrace, registracniCislo, drzitel,
ucinneLatky[] (názvy). Hraniční, nechat když vyplněno: `zavislost`, `platnostRegistraceDo`.

**Vyhodit** (interní šum): jeDodavka, povinneVzorky, obalKod, zeme*, aktualniDrzitel*,
kodSUKLreg, uvadeniNaTrhDo, indikacniSkupinaKod, registracniProcedura*, souběžný
dovoz*, ddd*, dopingKod, narizeniVladyKod, braillovoPismo, expirace*, MRPcislo,
registrovanyNazevLP.

`leciva-cli detail <kod> --raw` → plná nefiltrovaná odpověď API (escape hatch;
`api get` zůstává taky). Účinné látky v detailu = jen názvy (množství dává `slozeni`).

## 7. `slozeni <kod>` — názvy látek

```json
[ { "latka": "IBUPROFEN", "mnozstvi": "400", "jednotka": "MG" } ]
```
Z živého `/slozeni/{kod}` (kódy + množství) + join na `cis/latky.tsv`. Kód látky,
co v číselníku není (čerstvější live vs měsíční dump), → `latka: null`, kód
zachovat (`kodLatky`), graceful.

## 8. `latka "<text>"` — hledání podle účinné látky (nové)

1. v `cis/latky.tsv` najdi kódy látek, jejichž název (NAZEV/INN) obsahuje dotaz (case-insensitive)
2. v `latka-drugs.tsv` najdi kódy léků pro ty látky
3. přelož kódy léků na názvy z `index.tsv`

```json
{ "query": "ibuprofen", "count": 142, "results": [ { "kodSukl": "0254299", "nazev": "IBALGIN", "sila": "400MG" } ] }
```
Limit jako u `search` (`--limit`, default 20).

## 9. SKILL.md

- doplnit příkaz `latka` + `detail --raw`
- jedna věta k prezentaci: „Výstup je čistý JSON s názvy; uživateli ho podej čitelně (lék, síla, forma, výdej, účinné látky), nedumpuj syrový JSON. Plný záznam je přes `detail --raw` / `api get`."

## 10. Edge-case

- Kód není v číselníku → přelož na `null`, syrový kód zachovej. Nikdy nespadnout.
- Číselníky chybí (starý cache bez `cis/`) → `ensure_index` / lazy refresh je dostaví; do té doby `detail` vrací kódy bez názvů (graceful), ne chybu.
- `latka` bez reverzního indexu → vrať jasný `ERROR:` ať uživatel ví, že chybí refresh.

## 11. Mimo rozsah

- `dokumenty` (SPC/PIL) a `ceny` (úhrady) — pořád v2
- překlad ATC na víceúrovňovou hierarchii — stačí název dané ATC skupiny
- lokalizace názvů látek do EN — bereme český `NAZEV`
