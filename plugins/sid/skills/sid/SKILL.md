---
uuid: 62923a9a-9e6d-4803-a9a8-c005049eeb71
name: sid
icon: icon.svg
description: "This skill should be invoked whenever the user asks about internal company documents managed in SID (Systém interních dokumentů) — searching directives, guidelines or regulations ('směrnice', 'řád', 'předpis', 'interní dokument', 'vnitřní předpis'), reading or downloading them, checking what they are required to confirm having read ('co mám potvrdit', 'nepotvrzené dokumenty'), confirming read receipts, or checking SID notifications and news."
version: 1.0.0
i18n:
  cs:
    displayName: "SID — Interní dokumenty"
    summary: "Vyhledávání a čtení řízených dokumentů, potvrzování přečtení, hlídání novinek."
  en:
    displayName: "SID — Internal Documents"
    summary: "Search and read managed documents, confirm reading, watch for updates."
  sk:
    displayName: "SID — Interné dokumenty"
    summary: "Vyhľadávanie a čítanie riadených dokumentov, potvrdzovanie prečítania, stráženie noviniek."
---

# SID — Interní dokumenty

**IMPORTANT:** Jediný nástroj této skill je `sid-cli`. NIKDY nevolej `curl`,
`wget` ani jiné nástroje přímo proti SID. Předpokládej, že `sid-cli` je
v PATH — nedělej preflight (`which sid-cli`, výpis env proměnných); začni
rovnou prvním skutečným příkazem.

**API klíč NIKDY neprochází konverzací.** Nastavuje se v panelu pluginu SID
(aplikace Codexis) nebo `echo KLIC | sid-cli settings set-key`. Nikdy nežádej
uživatele, aby klíč vložil do chatu, a nikdy ho nevypisuj. Když příkaz skončí
**exit kódem 3**, řekni uživateli, ať si vygeneruje klíč v SID
(Profil → API klíče, `/profile/api-klice`) a vloží ho v nastavení pluginu.

## Příkazy

```bash
sid-cli search "smlouva o dílo" --size 5          # fulltext napříč obsahem
sid-cli docs list --query "spisový řád" --sort newest
sid-cli docs get <documentId>                     # metadata + soubory
sid-cli download <fileId> --out ./smernice.pdf
sid-cli confirmations list --state pending        # co mám potvrdit
sid-cli confirmations confirm <confirmationId>
sid-cli notifications                             # nepřečtené novinky
sid-cli check                                     # souhrn: novinky + potvrzení
```

Výstup je JSON — kombinuj s `jq`:

```bash
sid-cli search "bezpečnost práce" --size 5 | jq '.hits[] | {documentId, title, snippet}'
sid-cli confirmations list --state pending | jq 'length'
sid-cli docs list --sort newest --page-size 5 | jq '.documents[] | {id, title}'
```

## Zrádnosti

- **`search` s polem `warning`** = fulltext je dočasně nedostupný. Prázdný
  výsledek pak NEZNAMENÁ, že dokument neexistuje — použij
  `sid-cli docs list --query "..."` a uživateli to řekni.
- **`search.total` NELZE použít na stránkování** — po ACL filtru je vždy
  ≤ `size`. Na procházení výpisu použij `docs list --page`.
- **Potvrzení (`confirmations confirm`) proveď jen na výslovný pokyn
  uživatele** — je to právní úkon „četl jsem a beru na vědomí". Nikdy ho
  nedělej „pro pořádek" ani hromadně bez potvrzení od uživatele.
- Klíč má **stejná oprávnění jako uživatel** — když něco nevidí, není to
  chyba nástroje, ale chybějící přístup ke složce.

## Exit kódy

| kód | význam | co udělat |
|-----|--------|-----------|
| 0 | OK | — |
| 2 | špatné použití | oprav argumenty |
| 3 | klíč chybí / vypršel | pošli uživatele na `/profile/api-klice` (viz výše) |
| 4 | nenalezeno | ověř id |
| 5 | rate limit | zpomal, zkus za chvíli |
| 1 | jiná chyba | přečti hlášku na stderr |

Detailní tvary odpovědí, všechny filtry a limity: `references/sid-api.md`.
