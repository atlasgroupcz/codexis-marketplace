# Katastr plugin — redesign UI (LM-121)

**Datum:** 2026-06-23
**Branch:** `lm-121-katastr-redesign`
**Jira:** LM-121 „Katastr plugin zhezčit"

## 1. Cíl a rozsah

Předělat frontend pluginu **katastr** (sledovaná katastrální řízení) tak, aby:

1. vizuálně patřil do rodiny CODEXIS — stejný shell a design jako Jirkův redesign `sledovana-judikatura` a `sledovane-dokumenty` (zelený akcent, serif titulky, ikona v zaobleném čtverci v hlavičce),
2. používal **progressive disclosure** — seznam ukazuje jen aktuální stav a poslední milník, celá historie operací je až v detailu,
3. měl jeden výrazný doménový prvek („signature") — **vodorovnou časovou osu průběhu řízení** s plombou jako zvýrazněným uzlem.

Měníme **jen frontend** appky `frontend/apps/katastr:katastr`. Backend (Rust/Python CLI, API tvar, schémata odpovědí) zůstává beze změny. Datový model (`schemas.ts`) se nemění — osa se odvozuje z existujících polí.

**Mimo rozsah (YAGNI):** žádné nové API endpointy, žádná změna backendu, žádné nové sledovací funkce, žádný dvoupanelový master–detail layout, žádné řazení/filtrování seznamu (není v zadání).

## 2. Odsouhlasená designová rozhodnutí

Vzešla z brainstormingu nad vizuálními mockupy (companion):

- **Řádek seznamu = varianta A** — bez levého stavového pruhu. Číslo řízení (spisovka, mono) vede, označení (label) je sekundární za ním, stav badge vpravo. Druhý řádek meta: úhrada · poslední milník (název + datum) · počet nepotvrzených změn. Klidné, dvouřádkové, nejblíž judikatuře.
- **Navigace = celá stránka.** Seznam → klik na řádek → detail přes celou šířku → „← Zpět". Žádný dvoupanel. (Beze změny oproti dnešku — `view`/`uuid` v URL přes nuqs.)
- **Detail = vystředěná kolona** max ~780 px, ne celý viewport.
- **Signature = vodorovná osa „Průběh řízení"** — tenký proužek s milníky (vizuál „T2"), pod každým uzlem nejdřív název, pod ním datum (pořadí „T1"). Hotová část čáry zelená, budoucí šedá, plomba oranžová, aktuální uzel má prstenec. Tlumené barvy, žádné vybledlé zelené plochy.
- **Status = jeden kompaktní řádek** (stav · úhrada · datum přijetí · poslední kontrola), ne 4 ohraničené boxy.
- **Barva:** CODEXIS zelená `#5ea500` jako primární akcent (maximální sjednocení). Plomba = oranžová `#c2410c`.
- **Mobil:** osa se na úzké obrazovce překlopí na svislou.

## 3. Vizuální systém

### 3.1 styles.css (hotovo)

`src/styles.css` už obsahuje brand vrstvu zkopírovanou z judikatury (zelený ramp `--color-brand-50..700`, `:root` tokeny `--primary:#5ea500`, `--secondary:#f4f7f0`, `@layer base` s `.font-display` serifem). Toto je základ sjednocení.

**Doplnit:** plomba tokeny do `@theme` / `:root`, aby osa a badge nečerpaly barvu z ad-hoc tříd:

```css
--color-plomba: #c2410c;       /* oranžová – plomba */
--color-plomba-soft: #fdf0e9;  /* podklad badge plomby */
```

### 3.2 Sjednocení badge komponent

`stav-badge.tsx` a `uhrada-badge.tsx` dnes používají ad-hoc Tailwind třídy (`bg-green-100`, `bg-yellow-100`, `bg-orange-100`, `bg-red-100`, `bg-blue-100`). Sjednotit na semantickou paletu konzistentní s CODEXIS:

- **dokončeno / uhrazeno** → zelená z brand rampu (`bg-brand-50 text-brand-700`),
- **plomba** → plomba tokeny (`bg-plomba-soft text-plomba`),
- **probíhá / čeká** → jantarová (`amber`),
- **neuhrazeno** → plomba/červená,
- **ukončeno / neutrální** → `bg-muted text-muted-foreground`.

Mapování zůstává keyword-based jako dnes (`stav.toLowerCase().includes(...)`), jen barvy jdou přes tokeny.

## 4. Odvození časové osy (nejcitlivější část)

Data nedávají hotový seznam fází. Máme:

- `stav` — volný text aktuálního stavu (např. „Povolení vkladu", „Plomba vyznačena", „Ukončeno"),
- `provedene_operace: {nazev, datumProvedeni}[]` — co se reálně stalo, s daty,
- `typ_rizeni`, `cislo_rizeni` (prefix `V-` = vklad, `Z-` = záznam, …).

### 4.1 Dvojí režim osy

**Režim A — kanonická osa (jen pro vklad, `V-…`):**
Pevná posloupnost 5 fází:

1. Přijetí návrhu
2. Plomba (vyznačení plomby) — *zvýrazněný uzel*
3. Povolení vkladu
4. Provedení vkladu
5. Vyrozumění o provedeném vkladu

Každou fázi zkusíme napárovat na `provedene_operace` přes keyword match (stejný styl jako stav-badge). Napárovaná fáze = **done**, dostane datum z operace. Poslední napárovaná (nebo fáze odpovídající `stav`) = **current** (prstenec). Nenapárované fáze za ní = **future** (tečkované, bez data nebo „—"). Plomba uzel je vždy oranžový bez ohledu na done/future.

**Režim B — fallback (ostatní typy, `Z-` apod., nebo když párování selže):**
Osu složíme **čistě z `provedene_operace`** — každá operace = jeden uzel s názvem a datem, seřazené podle `datumProvedeni`. Žádné spekulativní budoucí kroky. Poslední operace = current.

Tím dodržíme dřívější dohodu: *budoucí kroky ukazujeme jen když je sled jistý.*

### 4.2 Umístění logiky

Čistá funkce `deriveTimeline(proc): TimelineNode[]` v novém `src/lib/timeline.ts`:

```ts
type TimelineNode = {
  key: string
  name: string          // název fáze / operace
  date: string | null   // ISO, nebo null pro budoucí
  state: 'done' | 'current' | 'future'
  isPlomba: boolean
}
```

Bez Reactu, bez i18n uvnitř — názvy fází bere z i18n přes klíče (viz §7), funkce vrací klíče/holá data, překlad řeší komponenta. Testovatelná izolovaně.

### 4.3 Poslední milník pro seznam

Řádek seznamu potřebuje „poslední milník". Vezmeme poslední prvek `provedene_operace` (nebo `last_op_date` + odpovídající název). Když operace nejsou, fallback na `stav`.

## 5. Architektura komponent (as-is → to-be)

Navigace v `app.tsx` zůstává (list ↔ detail přes URL). Mění se vnitřek dvou obrazovek.

### 5.1 Seznam

| dnes | nově |
|------|------|
| `proceedings-list.tsx` — `<h1>` + ghost settings, `AddProceedingForm`, HTML `<table>` s 5 sloupci vč. „Provedené operace" (dump všech operací) | shell jako judikatura: `flex h-screen flex-col`, `header h-14` s ikonou v `rounded-xl` čtverci, serif titul + muted podtitul, akce vpravo (Přidat, Aktualizovat, Nastavení). Místo tabulky **seznam řádků** (var. A). |
| `proceeding-row.tsx` — `<tr>` se 5 `<td>`, sloupec dumpuje všechny `provedene_operace` | `<button>`/`<div>` řádek: číslo (mono) + label, stav badge vpravo, meta řádek (úhrada · poslední milník · nepotvrzené). **Žádný dump operací.** |

Ikona hlavičky: vybrat lucide ikonu pro katastr (kandidáti `Landmark`, `MapPin`, `FileStack`) — rozhodne se v plánu/implementaci, default `Landmark`.

`AddProceedingForm` zůstává funkčně stejný (přidat číslo, zkontrolovat vše), jen se vizuálně zasadí do shellu. Konkrétně: v hlavičce vpravo jsou tlačítka „↻ Aktualizovat" (volá `checkAll`) a „+ Přidat". Tlačítko „+ Přidat" rozbalí **inline řádek s polem** pro číslo řízení (mono input + potvrzení) hned pod hlavičkou; po přidání se sbalí. Settings zůstává dialog (ikona ozubeného kola v hlavičce).

Stavy zachovat: pre-config (banner když není API klíč), loading skeleton, error, empty.

### 5.2 Detail

| dnes | nově |
|------|------|
| `proceeding-detail.tsx` — `DetailHeader` + `StatusGrid` + `OperationsTable` + `ChangeHistory` | stejná kostra, vystředěná kolona max-w ~780 px |
| `detail/status-grid.tsx` — 4 ohraničené boxy | **kompaktní status řádek** (jeden řádek, `flex flex-wrap`, oddělovače `·`) |
| `detail/operations-table.tsx` — prostá tabulka operací | **`detail/timeline.tsx`** — vodorovná osa (signature) z `deriveTimeline`; na mobilu svislá |
| `detail/detail-header.tsx` — zpět + číslo + edit label + odebrat | zachovat chování (inline edit labelu, dvojklik-na-potvrzení odebrání), jen sladit s brand stylem |
| `detail/change-history.tsx` — historie změn, potvrzování | zachovat, sladit barvy (nepotvrzené dnes modré → brand/plomba akcent dle tématu) |

Osa nahrazuje `OperationsTable` jako primární zobrazení průběhu. Aby se v režimu A neztratily operace, které se nenamapovaly na žádný uzel, ponecháme **sbalený disclosure „Všechny provedené operace"** pod osou — sbalený ve výchozím stavu, po rozkliknutí ukáže prostý seznam `provedene_operace` (přebírá obsah dnešní `OperationsTable`). V režimu B je osa sama tvořená všemi operacemi, takže disclosure je zbytečný a nezobrazuje se. Pořadí v detailu: status řádek → osa → (disclosure operací) → historie změn.

## 6. Stavy a okrajové případy

- **Pre-config** (chybí API klíč) — banner + hlavička, beze změny chování.
- **Loading** — skeleton (sladit s novým layoutem).
- **Error** — `ErrorMessage` + retry.
- **Empty** — žádná řízení: vzdušná empty hláška (jako dnes, jen v novém shellu).
- **Osa bez operací** — režim B s prázdným polem → zobrazit jen aktuální `stav` jako jediný uzel „current".
- **Neznámý `stav`** — badge spadne do neutrální (muted) varianty.

## 7. i18n

Klávesy `proceedings.*` v `public/locales/{cs,en,sk}/translation.json`. Přidat klíče pro názvy fází osy a popisky:

- `proceedings.timeline.title` („Průběh řízení")
- `proceedings.timeline.prijeti`, `.plomba`, `.povoleniVkladu`, `.provedeniVkladu`, `.vyrozumeni`
- `proceedings.timeline.now` („teď"), `proceedings.timeline.expected` („očekává se" / „—")
- `proceedings.lastMilestone` („Poslední milník") pro meta řádek seznamu

Technické `name`/`id` se nepřekládají. Top-level `SKILL.md` / `plugin.json` `i18n` bloky se nemění (nejde o nový plugin).

## 8. Build a sync

Beze změny pipeline. Build přes Docker: `plugins/katastr/components/build.sh` (`oven/bun:1.3.9-slim`, `bun run build` v `apps/katastr:katastr`, pak `sync_component_output` z `dist` do `plugins/katastr/components/katastr`, zachová `icon.png`/`icon.svg`). Po implementaci ověřit lokální build a sync.

## 9. Testování

- **Unit:** `deriveTimeline` — režim A (vklad, párování fází, current/future, plomba), režim B (fallback z operací), prázdné operace, neznámý stav. Toto je jediná netriviální logika, pokrýt důkladně.
- **Vizuální/manuální:** appku nelze spustit živě (chybí user token), ověření přes build + companion mockupy. Vizuální shodu se rodinou ověřit proti judikatuře/dokumentům.
- Stávající chování (přidat, zkontrolovat, edit label, odebrat, potvrdit změnu) nesmí regredovat — `data-testid` atributy zachovat.

## 10. Rozčlenění práce (pro plán)

1. Doplnit plomba tokeny do `styles.css`; sjednotit `stav-badge` / `uhrada-badge` na brand paletu.
2. `lib/timeline.ts` + testy (`deriveTimeline`).
3. Shell + hlavička + řádek seznamu (var. A), bez dumpu operací.
4. Detail: kompaktní status řádek + `detail/timeline.tsx` (vodorovná, mobil svislá); sladit header a change-history.
5. i18n klíče (cs/en/sk).
6. Build + sync, ověření.
