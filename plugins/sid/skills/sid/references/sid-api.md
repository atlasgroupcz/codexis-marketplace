# SID API — tvary odpovědí a filtry

Referenční detaily pro `sid-cli`. Endpointy volá binárka sama; tahle stránka
slouží k tomu, aby šlo správně interpretovat výstup a poskládat filtry.

## Mapa příkaz → endpoint

| Příkaz | Endpoint |
|--------|----------|
| `search` | `GET /api/search` |
| `docs list` | `GET /api/v1/documents` |
| `docs get <id>` | `GET /api/v1/documents/{id}` |
| `download <fileId>` | `GET /api/files/download/{fileId}` |
| `confirmations list` | `GET /api/v1/confirmations` |
| `confirmations confirm <id>` | `PATCH /api/v1/confirmations/{id}` |
| `notifications` | `GET /api/notifications` |
| `check` | notifikace + confirmations naráz |
| `settings test` | `GET /api/v1/users/me` |

Autentizace vždy hlavičkou `x-api-key`; klíč se čte z
`~/.cdx/apps/sid/.env` (mode 0600), nikdy z argumentů.

## `search`

```
sid-cli search "<dotaz>" [--size N] [--from N]
```

Odpověď:

```json
{
  "total": 3,
  "hits": [
    {
      "documentId": "…",
      "documentFileId": "…",
      "pageNumber": 4,
      "title": "Spisový řád",
      "filename": "spisovy-rad.pdf",
      "snippet": "…zvýrazněný úryvek…",
      "score": 8.13
    }
  ],
  "source": "opensearch"
}
```

Dvě pasti:

- **`total` není celkový počet** — route ho po ACL filtru přepočítá na
  `hits.length`, takže je vždy ≤ `size`. Stránkovat podle něj nelze.
- **`source: "opensearch_unavailable"`** znamená výpadek fulltextu, ne
  „nic nenalezeno" — HTTP je v obou případech 200. `sid-cli` v tom případě
  přidá pole `warning`; ber ho vážně a zkus `docs list --query`.

## `docs list`

```
sid-cli docs list [--query T] [--folder ID] [--author ID]
                  [--sort newest|oldest|title|size] [--page N] [--page-size N]
```

`--page-size` je na serveru omezená na 100. Odpověď:

```json
{ "documents": [ … ], "total": 42, "page": 1, "pageSize": 20, "totalPages": 3 }
```

Tady `total` **je** skutečný počet — na rozdíl od `search`.

## `confirmations`

`--state pending` vrací to, co uživatel ještě nepotvrdil; `confirmed` už
vyřízené. Stavy potvrzení jsou `UNSEEN` → `SEEN` → `CONFIRMED`.

`confirmations confirm <id>` pošle `PATCH { "state": "CONFIRMED" }`. Je to
právní úkon — dělej ho jen na výslovný pokyn uživatele.

## `notifications`

```
sid-cli notifications [--all] [--limit N]
```

Bez `--all` vrací jen nepřečtené. `--limit` je na serveru omezený na 500.

## `check`

Digest pro automatizaci (`automations/check-sid.md`, denně v 7:00):

```json
{
  "unreadNotifications": 2,
  "pendingConfirmations": 1,
  "notifications": [ … ],
  "confirmations": [ … ],
  "notified": true
}
```

S `--notify` navíc pošle e-mail + in-app bell přes daemon — ale jen když je
co hlásit. `notified: false` znamená, že se neposílalo (nic nového), nebo že
daemon nebyl dostupný; nikdy to není chyba, která by shodila běh.

## Limity

- search 60 požadavků/min na IP, download 120/min na IP, upload 20/min
- 120 požadavků/min na klíč (nezávisle na IP)
- Klíč má TTL 90 dní; po vypršení vrací API 401 (`sid-cli` → exit 3)

## Co přes CLI nejde

Tyto věci nejsou v SID dostupné přes HTTP API (existují jen jako Next.js
server actions), takže je `sid-cli` neumí a ani umět nebude:

- správa oprávnění složek,
- zápisy do skupin,
- legislativa a Hypertexter,
- introspekce přístupů („co tenhle uživatel vidí").

Upload dokumentů API podporuje, ale `sid-cli` ho zatím nemá — nahrávání
řeš přes webové rozhraní SID.
