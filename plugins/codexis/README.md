## Co to je?

CODEXIS plugin přináší přístup k nejrozsáhlejší české právní databázi. Místo přepínání mezi aplikacemi a ručního hledání v právních předpisech se jednoduše zeptáte a dostanete přesnou odpověď s odkazem na zdroj.

## Co v něm najdete?

### Česká a slovenská legislativa
Zákony, vyhlášky, nařízení - vždy v aktuálním platném znění. Stačí zadat například „občanský zákoník" nebo „zákoník práce § 52" a plugin vyhledá příslušné ustanovení včetně jeho přesného textu.

### Judikatura českých soudů
Rozhodnutí Nejvyššího soudu, Ústavního soudu a dalších - s možností filtrovat podle soudu, období nebo právní oblasti.

### Legislativa a judikatura EU
Nařízení, směrnice a rozhodnutí Soudního dvora EU. Užitečné při práci s přeshraničními tématy nebo implementací evropského práva.

### Právní literatura a komentáře
Odborné publikace a komentáře LIBERIS k jednotlivým zákonům - pro hlubší pochopení výkladu práva.

### Vzory smluv
Šablony a vzory smluv pro běžné právní situace.

## K čemu se to hodí v praxi?

- **Právní rešerše** - rychle ověříte, co říká zákon, jak ho vykládají soudy a co k tomu píší odborníci.
- **Daně a účetnictví** - volitelné monitorové pluginy pro daňové a účetní dotazy kombinují předpisy, judikaturu a výkladovou praxi.
- **Příprava dokumentů** - při tvorbě smluv nebo právních podání máte okamžitý přístup k aktuálnímu znění předpisů.
- **Kontrola souladu** - ověříte, zda váš postup nebo dokument odpovídá platné legislativě.
- **Sledování změn legislativy** - komponenta „Sledované dokumenty" vám umožní sledovat změny u vybraných právních předpisů, nebo si nechat hlídat složku se smlouvami ve svém projektu a legislativu, na kterou odkazují.
- **Sledování judikatury** - komponenta „Sledovaná judikatura" hlídá nová rozhodnutí k tématu, které si zvolíte, a posílá pravidelný report o trendech.

## Součásti pluginu

### Vyhledávání v právní databázi
Hlavní funkce - fulltextové vyhledávání napříč všemi zdroji CODEXIS. Výsledky obsahují přímé odkazy do databáze.

### Sledované dokumenty
Interaktivní komponenta se dvěma režimy:

- **Sledování předpisů** - vyberete konkrétní právní předpisy a dozvíte se o jejich změnách.
- **Sledování složky v projektu** - vyberete některý ze svých projektů a v něm složku s dokumenty (nebo celý projekt). Smlouvy ve formátu `.docx`, `.pdf` či `.txt` plugin projde, vytěží z nich, na kterou legislativu odkazují, a dál hlídá změny právě těch předpisů. Nemusíte tedy vědět dopředu, co všechno hlídat, a nový dokument ve složce se přidá sám.

Upozornění chodí do aplikace. E-mail je ve výchozím stavu vypnutý, zapnout ho
a přidat další adresáty můžete v nastavení doplňku.

### Sledovaná judikatura
Interaktivní komponenta pro dlouhodobé sledování judikatury k tématu. Popíšete, co vás zajímá („nová judikatura k okamžitému zrušení pracovního poměru"), plugin téma rozdělí na podoblasti, postaví výchozí analýzu současného přístupu soudů a pak v pravidelných intervalech reportuje, co se změnilo. Report se přizpůsobí vaší profesi a preferovanému formátu.

### CLI nástroje
Plugin při instalaci přidává `cdx-cli` jako hlavní CLI a instaluje také
`cdxctl` pro správu platformy, `cdx-sledovane-dokumenty` pro správu
sledovaných dokumentů a složek projektů a `cdx-sledovana-judikatura` pro správu
sledovaných témat judikatury.

### Správa platformy (cdxctl)
Nástroj pro pokročilé uživatele - správa automatizací (pravidelné úlohy), marketplace pluginů a extrakce dat z dokumentů do strukturované tabulky.

### Dokumentace CODEXIS AI
Dovednost pro dotazy k funkcím, používání, podmínkám, studentské nabídce a podpoře čte aktuální veřejnou dokumentaci na docs.codexis.ai. Odpověď odkazuje na konkrétní stránku.

### Balíčky ve virtuálním počítači

Agent může pomocí `cdxctl package` zjistit stav, nainstalovat nebo odebrat balíčky Ubuntu
a aktualizovat jejich seznam. Změny platí pro celý virtuální počítač uživatele; přežijí restart,
ale ne výměnu jeho systémového obrazu.

### Klíče a nastavení pluginů

Přes `cdxctl appenv` může asistent uložit nebo smazat hodnotu, kterou si plugin čte ze svého
souboru `.env` (typicky API klíč k datovému zdroji). Plugin do souboru sám nezapisuje, o zápis
se stará platforma.

## Kdo za tím stojí?

Plugin vyvíjí **ATLAS GROUP** - tvůrce systému CODEXIS, který je standardem v oblasti přístupu k českému právu.

Kontakt: klientske.centrum@atlasgroup.cz
