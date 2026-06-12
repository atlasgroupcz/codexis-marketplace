# CLI Reference

Use only the installed `ares` command.

```bash
ares search <query>
ares company <ico>
ares officers <ico>
ares trades <ico>
ares raw <ico> --source basic|vr|res|rzp
```

## Business Cases And Output Mapping

Ověřeno proti přiloženému OpenAPI souboru `api-docs.json` pro endpointy `/ekonomicke-subjekty/vyhledat`, `/ekonomicke-subjekty/{ico}`, `/ekonomicke-subjekty-vr/{ico}` a `/ekonomicke-subjekty-rzp/{ico}`.

| Command | User question | ARES endpoint | When to call | Output mapping |
|---|---|---|---|---|
| `ares search <query>` | "Najdi firmu podle názvu." | `POST /ekonomicke-subjekty/vyhledat` | Uživatel zadá název nebo část názvu. | `kandidati`: `nazev`, `ico`, `sidlo`, `pravniForma`, `primarniZdroj`, `stavRegistraci`. |
| `ares company <ico>` | "Zobraz základní údaje o subjektu." | `GET /ekonomicke-subjekty/{ico}` | Uživatel zná IČO a potřebuje název, adresu, právní formu, stav nebo data. | `kartaSubjektu`: `nazev`, `ico`, `dic`, `sidlo`, `pravniForma`, `datumVzniku`, `datumZaniku`, `datumAktualizace`, `primarniZdroj`, `registrace`, `czNace`. |
| `ares officers <ico>` | "Kdo může za firmu jednat?" | `GET /ekonomicke-subjekty-vr/{ico}` | Dotaz na statutární orgány, jednatele, členy orgánů nebo způsob jednání. | `zaznamy`: `statutarniOrgany`, `clenove`, `funkce`, období členství/funkce, `zpusobJednani`, `spisovaZnacka`, `zakladniKapital`, `spolecnici`, `akcionari`, `exekuce`, `insolvence`, `konkursy`, pokud je VR vrátí. |
| `ares trades <ico>` | "Jaká má firma živnostenská oprávnění?" | `GET /ekonomicke-subjekty-rzp/{ico}` | Dotaz na podnikatelskou činnost, živnosti, licence nebo rozsah podnikání. | `zaznamy`: `zivnosti`, `druhZivnosti`, vznik/zánik/platnost, přerušení/pozastavení, `podminky`, `obory`, `odpovedniZastupci`, `provozovny`, `zivnostiStav`, `provozovnyStav`. |
| `ares raw <ico> --source <source>` | "Ukaž surová data ze zdroje." | Source-specific `GET` | Ladění, pokročilé použití nebo pole mimo zjednodušené příkazy. | Původní JSON bez interpretace. |

## Raw Sources

- `basic` - `/ekonomicke-subjekty/{ico}`; základní identifikace, zdroj pro `ares company`.
- `vr` - `/ekonomicke-subjekty-vr/{ico}`; veřejný rejstřík, zdroj pro `ares officers`.
- `res` - `/ekonomicke-subjekty-res/{ico}`; statistické údaje z RES.
- `rzp` - `/ekonomicke-subjekty-rzp/{ico}`; živnostenský rejstřík, zdroj pro `ares trades`.

## Output

CLI vypisuje JSON na stdout. Mapované příkazy obsahují `echo` s příkazem, endpointem a normalizovaným vstupem. `ares raw` vypisuje původní JSON bez `echo`.

Při chybě vypíše `ERROR: ...` na stderr a vrátí exit code `2`. HTTP chyby obsahují metodu, endpoint, HTTP status a chybový text ARES, pokud jej API vrátí.
