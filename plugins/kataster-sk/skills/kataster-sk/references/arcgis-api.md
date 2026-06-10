# ESKN ArcGIS REST — parcel layers

Base: `https://kataster.skgeodesy.sk/eskn/rest/services/VRM`

Standard ArcGIS MapServer REST (version 10.91). Anonymous, JSON (`f=json`).
Accessed exclusively through `kataster-sk-cli detail`, which sends the
browser-like `User-Agent` header the WAF requires.

## Layers

| Layer | URL | Content |
|---|---|---|
| Parcely C | `/parcels_c_view/MapServer/0` | Register C parcels (current land map) |
| Parcely E | `/parcels_e_view/MapServer/0` | Register E parcels (legal/original state) |

Layer metadata: `GET {layer}?f=json` — lists all fields with Slovak aliases.

## Query (`kataster-sk-cli detail`)

```
GET {layer}/query?objectIds={id}&outFields=*&returnGeometry=false&f=json
```

- `objectIds` — id obtained from the ZBGIS parcel search (`data.id`); the only
  reliable filter.
- `returnGeometry=true&outSR=4326` (the CLI's `--geometry` flag) — adds the
  parcel polygon in WGS84 (source SRS is S-JTSK, wkid 5514).

### Restrictions (WAF)

- `where=` clauses (even `where=1=1`) are blocked — the server returns an
  **HTML error page** instead of JSON. Detect by attempting JSON parse.
- The services catalog (`/eskn/rest/services?f=json`) returns 403.
- Max 1000 records per query (`maxRecordCount`).

## Key attributes (both layers)

| Field | Meaning |
|---|---|
| `PARCEL_NUMBER` | Parcel number, e.g. `1234/9` |
| `CADASTRAL_UNIT_ID` | Internal cadastral-unit id (not the public KU code) |
| `DESCRIPTIVE_AREA_OF_PARCEL` | Official (registered) area in m² |
| `GEODETIC_AREA_OF_PARCEL` | Area computed from geometry in m² |
| `FOLIO_ID` | Internal title-deed (LV) identifier; `null` = no LV in this view |
| `NATURE_OF_LAND_USE_ID` | Land-use code (druh pozemku) |
| `PLOT_UTILISATION_ID` | Utilisation code (spôsob využívania) |
| `OWNERSHIP_TYPE_ID` | Ownership form code |
| `HOUSE_NUMBER` | Building number when a building stands on the parcel |
| `XMIN/XMAX/YMIN/YMAX` | Bounding box (Web Mercator) |

Numeric `*_ID` fields are **internal code-list identifiers without any public
mapping**: the layers expose no ArcGIS coded-value domains, the values do not
match the official kódy druhov pozemkov (vyhláška č. 461/2009 Z. z. uses
2–14 with gaps, while e.g. a built-up Ružinov parcel carries
`NATURE_OF_LAND_USE_ID = 9`), and the translating endpoint sits behind the
recaptcha-gated ESKN portal. Report the raw identifier and direct the user to
the parcel's MAPKA page, which renders the human-readable labels.

## Map images

WMS endpoint for rendering cadastre map tiles:
`https://kataster.skgeodesy.sk/eskn/services/NR/kn_wms_norm/MapServer/WmsServer`
