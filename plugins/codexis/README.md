# CODEXIS Plugin

**Právní databáze na dosah ruky — přímo v prostředí, kde pracujete.**

## Co to je?

CODEXIS plugin přináší přístup k nejrozsáhlejší české právní databázi. Místo přepínání mezi aplikacemi a ručního hledání v právních předpisech se jednoduše zeptáte a dostanete přesnou odpověď s odkazem na zdroj.

## Co v něm najdete?

### Česká a slovenská legislativa
Zákony, vyhlášky, nařízení — vždy v aktuálním platném znění. Stačí zadat například „občanský zákoník" nebo „zákoník práce § 52" a plugin vyhledá příslušné ustanovení včetně jeho přesného textu.

### Judikatura českých soudů
Rozhodnutí Nejvyššího soudu, Ústavního soudu a dalších — s možností filtrovat podle soudu, období nebo právní oblasti.

### Legislativa a judikatura EU
Nařízení, směrnice a rozhodnutí Soudního dvora EU. Užitečné při práci s přeshraničními tématy nebo implementací evropského práva.

### Právní literatura a komentáře
Odborné publikace a komentáře LIBERIS k jednotlivým zákonům — pro hlubší pochopení výkladu práva.

### Vzory smluv
Šablony a vzory smluv pro běžné právní situace.

## K čemu se to hodí v praxi?

- **Právní rešerše** — rychle ověříte, co říká zákon, jak ho vykládají soudy a co k tomu píší odborníci.
- **Příprava dokumentů** — při tvorbě smluv nebo právních podání máte okamžitý přístup k aktuálnímu znění předpisů.
- **Kontrola souladu** — ověříte, zda váš postup nebo dokument odpovídá platné legislativě.
- **Sledování změn** — komponenta „Sledované dokumenty" vám umožní sledovat změny u vybraných právních předpisů a souvisejících dokumentů.

## Součásti pluginu

### Vyhledávání v právní databázi
Hlavní funkce — fulltextové vyhledávání napříč všemi zdroji CODEXIS. Výsledky obsahují přímé odkazy do databáze.

### Sledované dokumenty
Interaktivní komponenta, která vám umožní sledovat vybrané právní předpisy a být informováni o jejich změnách.

### CLI nástroje
Plugin při instalaci přidává `cdx-cli` jako hlavní CLI a instaluje také
`cdxctl` pro správu platformy a `cdx-sledovane-dokumenty` pro správu
sledovaných dokumentů. Součástí instalace je také `cdx-link-rewriter`,
který backend používá jako PATH-resolved render hook pro přepis `cdx://`
odkazů na absolutní URL.

### Správa platformy (cdxctl)
Nástroj pro pokročilé uživatele — správa automatizací (pravidelné úlohy), marketplace pluginů a extrakce dat z dokumentů do strukturované tabulky.

## Kdo za tím stojí?

Plugin vyvíjí **ATLAS GROUP** — tvůrce systému CODEXIS, který je standardem v oblasti přístupu k českému právu.

Kontakt: klientske.centrum@atlasgroup.cz
