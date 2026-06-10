# CLI Reference

Use only the installed `ares` command.

```bash
ares search <query>
ares company <ico>
ares officers <ico>
ares trades <ico>
ares owners <ico>
ares raw <ico> --source basic|vr|res|rzp|rpsh
```

## Business Cases

| Command | User question | ARES endpoint | When to call |
|---|---|---|---|
| `ares search <query>` | "Najdi firmu podle názvu." | `POST /ekonomicke-subjekty/vyhledat` | Uživatel zadá název nebo část názvu. |
| `ares company <ico>` | "Zobraz základní údaje o subjektu." | `GET /ekonomicke-subjekty/{ico}` | Uživatel zná IČO a potřebuje název, adresu, právní formu, stav nebo data. |
| `ares officers <ico>` | "Kdo může za firmu jednat?" | `GET /ekonomicke-subjekty-vr/{ico}` | Dotaz na statutární orgány, jednatele, členy orgánů nebo způsob jednání. |
| `ares trades <ico>` | "Jaká má firma živnostenská oprávnění?" | `GET /ekonomicke-subjekty-rzp/{ico}` | Dotaz na podnikatelskou činnost, živnosti, licence nebo rozsah podnikání. |
| `ares owners <ico>` | "Kdo je skutečný majitel?" | `GET /ekonomicke-subjekty-rpsh/{ico}` | Dotaz na skutečné majitele, AML, KYC nebo vlastnickou/compliance kontrolu. |
| `ares raw <ico> --source <source>` | "Ukaž surová data ze zdroje." | Source-specific endpoint | Ladění, pokročilé použití nebo pole mimo zjednodušené příkazy. |

## Raw Sources

- `basic` - `/ekonomicke-subjekty/{ico}`; základní identifikace, zdroj pro `ares company`.
- `vr` - `/ekonomicke-subjekty-vr/{ico}`; veřejný rejstřík, zdroj pro `ares officers`.
- `res` - `/ekonomicke-subjekty-res/{ico}`; statistické údaje z RES.
- `rzp` - `/ekonomicke-subjekty-rzp/{ico}`; živnostenský rejstřík, zdroj pro `ares trades`.
- `rpsh` - `/ekonomicke-subjekty-rpsh/{ico}`; skuteční majitelé, zdroj pro `ares owners`.

## Output

CLI vypisuje JSON na stdout. Při chybě vypíše `ERROR: ...` na stderr a vrátí nenulový exit code.
