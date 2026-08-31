# Právní osobnost, rejstříky a vnitřní církevní právo

## Scope

Použij pro otázky na právní formu církevní organizace, evidovanou právnickou osobu, statutární jednání, církevní charitu, eparchii, sbor nebo farnost, vnitřní předpisy, kanonické právo a souběh církevního a státního práva.

## Výzkumný podklad

Reference vychází z tematického korpusu chatů monitoru Církev od 1. 1. 2026. Monitor v tomto období neměl žádný uživatelský rating. Opakovala se právní forma církevní charity, rejstřík konkrétní církevní osoby, trestní odpovědnost duchovních podle kanonického práva, archivace sboru, správa církevního majetku, konkordátní judikatura a užití označení odkazujícího na církev.

## Čtyři oddělené vrstvy

1. **Státní právní osobnost.** Zda subjekt existuje jako právnická osoba a v jakém rejstříku je veden.
2. **Vnější jednání.** Kdo je statutárním orgánem, jak jedná a co je veřejně zapsáno.
3. **Vnitřní církevní oprávnění.** Kdo dal souhlas podle základního dokumentu, kanonického nebo jiného vnitřního práva.
4. **Právní následek.** Zda porušení vnitřního pravidla ovlivní civilní jednání, pracovní vztah, odpovědnost nebo jen vnitřní postavení osoby.

Nikdy nepřeskakuj přímo z porušení vnitřního předpisu k závěru o neplatnosti, trestnosti nebo jiné státní sankci.

## Identifikace subjektu

- Vyžádej přesný název, sídlo, identifikační číslo a nadřízenou církev.
- Ověř Rejstřík registrovaných církví a náboženských společností a Rejstřík evidovaných právnických osob vedené Ministerstvem kultury.
- Podle § 15a zákona č. 3/2002 Sb. se evidovaná právnická osoba stává právnickou osobou dnem evidence.
- Jiná osoba založená církví může mít právní osobnost podle jiného předpisu. Samotné slovo „církevní“, „charita“, „farnost“, „sbor“ nebo „eparchie“ právní formu neprokazuje.
- Ze základního dokumentu ověř organizační strukturu, typy evidovaných právnických osob, statutární orgány a pravidla hospodaření.

## Vnitřní právo a státní právo

- Nejprve určuj konkrétní církev nebo náboženskou společnost. Katolické kanonické právo nelze automaticky použít na pravoslavnou eparchii nebo protestantský sbor.
- U kanonického kodexu nebo interního předpisu ověř název, vydavatele, verzi, účinnost, územní a osobní působnost.
- Odděl vnitřní delikt a sankci od trestného činu podle českého trestního zákoníku.
- U duchovního odděl kanonické postavení, pracovněprávní nebo služební vztah a veřejnoprávní odpovědnost.
- U archivace sestav soupis dokumentů podle původce a účelu: účetní a daňové, pracovněprávní, matriční nebo evidenční, majetkové, interní církevní a archiválie.

## Názvy, média a označení

U názvu používajícího výraz spojený s církví proveď samostatně:

1. právo obchodní firmy nebo názvu právnické osoby,
2. ochranné známky a jiná registrovaná označení,
3. nekalou soutěž a riziko klamání,
4. osobnostní nebo reputační zásah,
5. případné vnitřní pravidlo konkrétní církve a jeho osobní působnost.

Neodpovídej, že církevní schválení je vždy nutné nebo nikdy nutné, dokud nejsou ověřeny všechny vrstvy a konkrétní způsob užití.

## Majetek a chátrající stavby

- Vlastnictví církve samo nevytváří obecnou povinnost pozemek rozvíjet nebo prodat.
- Prověř stavební, památkovou, bezpečnostní, požární a sousedskou vrstvu podle skutečného stavu stavby.
- Zjisti, zda jde o kulturní památku, závadný stav, veřejné ohrožení nebo jen ekonomicky nevyužitý majetek.
- Odděl nástroj veřejného orgánu od soukromého vyjednávání o koupi, nájmu, služebnosti nebo spolupráci.

## cdx-cli postup

```bash
cdx-cli get cdx://cz_law/3/2002/versions
cdx-cli get 'cdx://cz_law/3/2002/text?part=paragraf10'
cdx-cli get 'cdx://cz_law/3/2002/text?part=paragraf15'
cdx-cli get 'cdx://cz_law/3/2002/text?part=paragraf15a'
cdx-cli search JD --query 'církev autonomie vnitřní předpis právní účinky' --limit 10
cdx-cli search JD --query 'evidovaná právnická osoba církev statutární orgán' --limit 10
```

## Pasti

- Určení právní formy podle názvu organizace.
- Záměna registrace církve a evidence jí založené právnické osoby.
- Použití kanonického práva jedné církve na jinou denominaci.
- Automatický civilní nebo trestní následek porušení interního pravidla.
- Odpověď na konkrétní rejstříkovou otázku bez kontroly veřejného zápisu.
- Jediná archivační lhůta pro obsahově odlišné dokumenty s různými právními důvody.
