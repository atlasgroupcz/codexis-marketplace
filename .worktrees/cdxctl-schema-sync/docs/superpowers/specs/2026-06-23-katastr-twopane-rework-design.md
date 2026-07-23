# Katastr — dvoupanel + odhlučnění (rework LM-121)

**Datum:** 2026-06-23
**Branch:** `lm-121-katastr-redesign`
**Navazuje na:** `2026-06-23-katastr-redesign-design.md` (původní redesign). Toto je iterace po zhlédnutí naživo.

## Proč

Po nasazení do CODEXIS démona vyšlo najevo:
- Appka běží uvnitř démonova obsahového panelu (démon má vlastní levou navigaci). Plnostránkový layout s obsahem vycentrovaným na `max-w-3xl` → obsah „pluje" uprostřed, kolem mrtvý prostor, působí rozházeně.
- Seznam i detail jsou přehlcené; nepotvrzené změny vizuálně rozházené, není jasné, kam koukat.
- Timeline se líbí — zůstává.

Uživatel chce **mailový klient**: vlevo seznam řízení, vpravo detail vybraného.

## Co se mění (jen frontend, backend/schémata beze změny)

### 1. Layout → dvoupanel master–detail

- **Shell** (hlavička s ikonou, serif titul, akce vpravo: ↻ Aktualizovat, ⚙ Nastavení) je nad oběma panely a vyplní celou šířku panelu démona — žádné centrované plovoucí `max-w`. (Pozn.: „+ Přidat" už v hlavičce není — viz níže.)
- **Desktop (md+):** dva sloupce `grid-cols-[300px_1fr]`.
  - Levý = nahoře **trvale viditelné pole pro přidání** (mono input „V-1234/2026-701" + tlačítko Přidat), pod ním seznam řízení (vždy viditelný), pevných ~300 px, vlastní scroll. Žádný toggle/rozbalování — pole je vidět pořád, je to primární akce. `data-testid` zachovat (`add-proceeding-input`, `add-proceeding-button`).
  - Pravý = detail vybraného řízení; když není nic vybráno, prázdný stav („Vyberte řízení vlevo"). Pravý panel vyplní zbytek šířky, ale **text uvnitř** (status, historie změn) má čtecí strop ~`max-w-2xl` (≈640–720 px) zarovnaný vlevo; timeline smí využít víc.
- **Mobil (< md):** jeden panel. Default seznam; výběr řízení → detail přes celou šířku + „← Zpět". Řízeno přes stávající nuqs `view`/`uuid`.

### 2. Levý seznam = mailové řádky (odhlučnění)

Řádek = kompaktní položka schránky:
- malá **tečka** vlevo = je nepotvrzená změna (jako nepřečtený mail); jinak prázdné místo,
- číslo řízení (mono, 600),
- označení (label) — tlumené, jednořádkové s ellipsis,
- malý stav badge.

**Z řádku zmizí** úhrada, datum přijetí, poslední milník, počty — to vše je v detailu. Vybraná položka: světlé pozadí `secondary` + levý zelený proužek (`inset 3px brand`).

### 3. Pravý detail — odhlučnění

- **Status** zůstává kompaktní jednořádkový (stav · úhrada · přijato · poslední kontrola).
- **Timeline** vizuálně beze změny (signature).
- **Historie změn** přepracovat z velkých orámovaných karet na klidné řádky:
  - hlavička „Historie změn · N nepotvrzené" + vpravo „Označit vše přečteno",
  - každá změna = řádek; **nepotvrzená** má jeden zelený proužek vlevo (`inset 3px brand`) + „✓ Potvrdit" vpravo (žádné rozházené oranžové/modré bloky),
  - tělo změny kompaktní: datum (tlumené) + `starý stav → nový stav`; přidané operace/úhrada jako drobné řádky pod tím, ne velký blok.
- Detail header bez plnostránkového rámce: „← Zpět" jen na mobilu; číslo + editace označení + Odebrat.

### 4. Bezpečné chování timeline (oprava)

Šedé „budoucí" kroky se smí dokreslovat jen když řízení skutečně tou cestou půjde:

- **Vklad na normální trati** (rozpoznaný průběžný stav): kanonická osa 5 fází + šedé budoucí kroky — jako dosud. Posloupnost je daná zákonem, nic jiného nenastane.
- **Ne-vklad** (Z-… apod.): osa jen z reálných operací, žádné budoucí kroky — jako dosud.
- **NOVÉ — koncový/negativní stav nebo neznámý stav:** když je stav vkladu negativně koncový (zamítnuto, zastaveno, zpětvzetí, odmítnuto) **nebo** ho neumíme zařadit a žádná operace se nenamapovala → **nedokresluj budoucí kroky**. Osa se v tomto režimu skládá **jen z reálných operací** (`provedene_operace`) jako hotové uzly, plus **koncový uzel = aktuální stav**. Žádný předpřipravený scénář se nekreslí, takže nikdy neukážeme šedé kroky, které už nenastanou. Cokoli dalšího ČÚZK reálně zaznamená jako operaci se objeví samo jako další uzel (osa je tu řízena daty, ne scénářem).

**Zobrazení zamítnutí (a jiných negativních konců):** poslední uzel není „teď" se zeleným prstencem (to by sugerovalo pokračování), ale **koncový negativní uzel** — tlumená/červená (plomba-like neutrální) tečka bez prstence, aby bylo jasné „řízení tu skončilo, dál nepokračuje". Pro tohle dostane `TimelineNode` příznak `isTerminal: boolean` (negativní konec). Komponenta `timeline.tsx` přidá jen tento jeden vizuální stav (tlumený koncový uzel); jinak beze změny.

Logika žije v `deriveTimeline` (`lib/timeline.ts`), pokrytá testy (zamítnutý vklad → poslední uzel `isTerminal`, žádné `future` uzly).

## Komponenty (as-is → to-be)

| soubor | změna |
|--------|-------|
| `app.tsx` | stane se shellem: drží `useOverview`, výběr (nuqs `uuid`/`view`), stav check/settings (žádný add-toggle); vykreslí hlavičku + dvoupanel + responsivní přepínání |
| `proceedings-list.tsx` | levý sloupec: nahoře trvalé pole pro přidání (`AddProceedingForm`), pod ním seznam (řádky + stavy loading/error/empty/pre-config), bez hlavičky a shellu; dostává proceedings + selectedUuid + onSelect |
| `add-proceeding-form.tsx` | beze změny chování (input + Přidat), jen se umístí natrvalo nad seznam (žádný toggle) |
| `proceeding-row.tsx` | mailový řádek (tečka, číslo, label, stav badge); odstranit úhradu/milník/datum; selected styl |
| `proceeding-detail.tsx` | pravý panel bez `h-screen`/`app-root` obalu; čtecí šířka obsahu; mobilní „Zpět" |
| `detail/detail-header.tsx` | „← Zpět" jen na mobilu; jinak beze změny |
| `detail/change-history.tsx` | přestylovat na klidné řádky (nepotvrzené = levý zelený proužek + Potvrdit) |
| `detail/status-grid.tsx` | beze změny (už je kompaktní řádek) |
| `lib/timeline.ts` | přidat bezpečné chování (koncový/negativní/neznámý stav → bez budoucích kroků), nový příznak `isTerminal` + testy |
| `detail/timeline.tsx` (komponenta) | přidat jeden vizuální stav: koncový negativní uzel (tlumená/červená tečka, bez prstence) |

`ListHeader` zůstává module-scope komponenta; přesune se do `app.tsx` (nebo vlastní soubor) jako součást shellu.

## Stavy a okraje

- Prázdný výběr na desktopu: pravý panel ukáže „Vyberte řízení ze seznamu vlevo."
- Pre-config (bez API klíče): banner v pravém panelu / přes obsah, hlavička bez akcí.
- Loading/error/empty seznamu: v levém panelu.
- Negativně koncové i neznámé stavy: viz §4.

## Mimo rozsah

Backend, API, `schemas.ts`, řazení/filtrování seznamu, nové sledovací funkce. Žádné nové i18n klíče nejsou nutné kromě prázdného stavu detailu (`proceedings.selectHint`) a případně koncových stavů (ty jsou jen vizuální, text bere ze `stav`).

## Testy

- `deriveTimeline`: nové případy — zamítnutý vklad (žádné budoucí kroky), neznámý stav bez operací (jeden uzel), normální vklad beze změny (regrese). 
- Stávající chování (add, check-all, edit label, remove, confirm) nesmí regredovat; `data-testid` zachovat. Dvoupanel: list i detail mají `data-testid` kotvy.
- Vizuál přes build + companion; appku nelze spustit s daty bez tokenu.
