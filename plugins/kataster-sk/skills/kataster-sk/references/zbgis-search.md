# ZBGIS MAPKA Search API

Base: `https://zbgis.skgeodesy.sk/mapka/api/search`

Undocumented but stable API used by the official MAPKA application
(`https://zbgis.skgeodesy.sk/mapka/sk/kataster`). Anonymous, JSON, GET only.
Accessed exclusively through `kataster-sk-cli`, which sends the browser-like
`User-Agent` header the WAF requires.

## Endpoints

### Geocode — cadastral units, municipalities, places (`kataster-sk-cli ku`)

```
GET /kataster?q={text}
```

Searches across cadastral units, municipalities, districts, streets and places.
Response:

```json
{
  "metadata": { "areMeans": false, "srcId": 0 },
  "items": [
    {
      "data": {
        "category": "katastrálne územie",
        "description": "katastrálne územie, obec Bratislava-Ružinov, okres Bratislava II",
        "extent": { "type": "envelope", "coordinates": [[17.13, 48.16], [17.19, 48.09]] },
        "id": "805556",
        "municipalityCode": "529320",
        "route": "ease/basic/805556",
        "text": "Ružinov (805556)"
      }
    }
  ]
}
```

`category` values seen: `katastrálne územie`, `obec`, `okres`, `ulica`.
For cadastral units `id` is the KU code used in all other endpoints.

### Parcel search within a cadastral unit (`kataster-sk-cli parcela`)

```
GET /kataster/{kuCode}/mu/{kuCode}?q={parcelNumber}
```

Prefix-matches parcel numbers in the given KU. Response items:

```json
{
  "data": {
    "category": "parcela-c",
    "description": "k.ú. Ružinov (805556)",
    "id": "1070374091",
    "route": "kataster/parcela-c/805556/1234_9",
    "text": "1234/9"
  }
}
```

- `category`: `parcela-c` (register C) or `parcela-e` (register E)
- `id`: object id for the ArcGIS `query?objectIds=` call (see `arcgis-api.md`)
- `route`: path suffix for the MAPKA deep link —
  `https://zbgis.skgeodesy.sk/mapka/sk/{route}` (parcel number has `/` → `_`)

Empty result is `{"items": [], "total": {"count": -1}}` — the parcel number does
not exist in that KU (or the query type is not supported anonymously; owner-name
search returns empty for anonymous sessions).

## Limitations

- No pagination metadata; long result lists are truncated by the server.
- Owner-name search is not available anonymously — do not present it as an option.
- Services may be temporarily unavailable (post-2025 cyberattack recovery).
