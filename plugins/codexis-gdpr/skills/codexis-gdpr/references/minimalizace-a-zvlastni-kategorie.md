# Minimalizace údajů a zvláštní kategorie v praxi

## Scope

Použij pro kopie osobních dokladů, personální evidenci, zdravotní a výživové údaje, komunikaci školy na sociálních sítích a jiné případy, kde správce zaměňuje potřebný údaj s kopií celého dokumentu nebo s celým komunikačním kanálem.

## Výzkumný podklad

Reference vychází z pozitivně hodnocených chatů monitoru Ochrana osobních údajů z období 1. 1.–31. 8. 2026 a z tematického korpusu stejného období. Hodnocené případy řešily školní profily na sociálních sítích, kopii průkazu zdravotního pojištění cizince a rozdíl mezi výživovým poradenstvím a zdravotní službou.

## Test nezbytnosti

Pro každý požadovaný údaj nebo dokument odpověz:

1. Jaký přesný účel se plní?
2. Který předpis nebo smluvní krok vyžaduje konkrétní údaj?
3. Stačí ověření pohledem, zápis dílčího údaje nebo potvrzení osoby?
4. Jaké další údaje obsahuje kopie a proč by byly potřeba?
5. Kdo bude mít přístup, jak dlouho a jak se výmaz prokáže?
6. Je údaj zvláštní kategorií podle čl. 9 GDPR?

Právní titul neřeší sám o sobě minimalizaci. I zákonné zpracování musí být omezeno na potřebné údaje.

## Personální doklady

- U průkazu zdravotního pojištění odděl povinnost zaměstnance sdělit pojišťovnu a potřebné identifikační údaje od uchovávání obrazové kopie průkazu.
- Pokud zvláštní zákon vyžaduje evidovat údaje, neodvozuj z toho automaticky právo uchovávat celý dokument.
- U zaměstnanců-cizinců rozliš doklady prokazující oprávněnost pobytu, jejichž kopie řeší zákon o zaměstnanosti, od průkazu zdravotního pojištění.
- Souhlas zaměstnance nepoužívej jako pohodlnou náhradu chybějící nezbytnosti.
- Do doporučení napiš minimální datový set, zdroj povinnosti, retenční dobu a způsob doložení kontroly.

## Zdravotní služba a výživové poradenství

- Nejprve kvalifikuj poskytovatele, povahu služby a to, zda jde o zdravotní službu podle zvláštního zákona.
- Pokud nejde o zdravotní službu, nepřenášej na poradce automaticky pravidla zdravotnické dokumentace a profesní výjimku čl. 9 odst. 2 písm. h) GDPR.
- Zpracovávané údaje mohou být údaji o zdraví i mimo zdravotní službu. Rozhoduje obsah a kontext, ne označení služby.
- Samostatně posuď pořizování zvukového nebo obrazového záznamu, soukromí, právní titul, výjimku pro zvláštní kategorii, transparentnost a retenci.
- Potřeba vést klientský záznam neznamená potřebu nahrávat celý rozhovor.

## Škola na sociální síti

- Urči účel školního profilu, obsah zveřejňovaný školou, povolené interakce, statistiky platformy a roli provozovatele platformy.
- Zakázané komentáře a nepoužívané soukromé zprávy snižují rozsah školou řízeného zpracování, ale neodstraňují zpracování návštěvnosti platformou.
- Text transparentnosti musí odpovídat skutečné konfiguraci profilu. Neuváděj, že škola nezpracovává údaje, pokud provozuje profil a využívá statistiky.
- Odděl zveřejňování fotografií žáků, běžné informační příspěvky, odkazy na školní web a komunikaci s veřejností.
- Prověř předávání mimo EHP a informace provozovatele platformy, ale nevydávej jeho zásady za vlastní právní titul školy.

## cdx-cli postup

```bash
cdx-cli get cdx://cz_law/48/1997/versions
cdx-cli get 'cdx://cz_law/48/1997/text?part=paragraf10'
cdx-cli get 'cdx://cz_law/48/1997/text?part=paragraf12'
cdx-cli get 'cdx://cz_law/435/2004/text?part=paragraf102'
cdx-cli get cdx://cz_law/372/2011/versions
cdx-cli search EU --query 'GDPR article 5 data minimisation article 9 health data' --limit 5
```

## Pasti

- Kopie celého dokladu jen proto, že jeden údaj z něj je potřebný.
- Smíchání pobytového dokladu a průkazu zdravotního pojištění.
- Automatická zdravotnická dokumentace u každého výživového poradce.
- Automatický závěr, že mimo zdravotní službu nejde o údaje o zdraví.
- Souhlas zaměstnance bez testu svobodnosti a nezbytnosti.
- Transparentnost sociální sítě neodpovídající skutečně povoleným funkcím profilu.
