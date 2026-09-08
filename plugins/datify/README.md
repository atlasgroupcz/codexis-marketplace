## Co to je?

Doplněk pro práci s Datify - datovou platformou, kde jsou údaje uložené
v kolekcích, datasetech a záznamech. Agent v nich umí číst, filtrovat je,
zakládat a upravovat záznamy a nahrávat přílohy.

## Co v něm najdete?

### Kolekce a datasety
Přehled toho, co v Datify máte, a jaké má dataset vlastnosti - jejich názvy,
typy a nastavení.

### Záznamy
Procházení po stránkách i cílené filtrování podle hodnot - text, čísla, data,
výběry, vazby, uživatelé, zaškrtávátka.

### Zakládání a úpravy
Nový záznam, hromadně až deset naráz, úprava jednotlivých vlastností i přepis
celého záznamu.

### Přílohy
Nahrání souboru přímo na záznam, do zásoby datasetu, nebo nepřipojeně.
Volitelně nad ním Datify spustí extrakci.

## Než začnete

Doplněk potřebuje API token. Vygenerujete ho v Datify a uložíte v nastavení
aplikace - token se před uložením ověří, takže překlep poznáte hned. Token se
ukládá přes platformu (`cdxctl`), doplněk sám do souboru s tokenem nezapisuje.

## Na co si dát pozor

**Mazání je nevratné.** Datify nemá koš ani historii, takže smazaný záznam je
pryč. Agent má pokyn si mazání nechat potvrdit a nejdřív ukázat, čeho se týká -
ale u hromadného mazání si výpis raději zkontrolujte.

**Hodnoty se v Datify neadresují názvem, ale identifikátorem vlastnosti.** Agent
si proto nejdřív načte dataset, aby věděl, co je co. Když se ptáte na konkrétní
sloupec, stačí ho pojmenovat běžně - překlad si obstará sám.
