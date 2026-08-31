# Kategorizace práce a pracovnělékařské služby

## Scope

Použij pro dotazy na kategorii práce, rizikové faktory, měření expozice, informování zaměstnance, vstupní a periodické prohlídky, profesní rizika a pracovníky jiného zaměstnavatele na sdíleném pracovišti.

## Výzkumný podklad

Reference vychází z pozitivně hodnocených chatů monitoru BOZP od 1. 1. 2026. Opakovala se v nich kategorizace prací, pravomoc hygienické stanice, sdělení kategorie zaměstnanci, zvláštní vyšetření, periodicita prohlídek a odpovědnost za pracovníky dodavatele. Právní pravidla byla ověřena v aktuálních zněních zákona č. 258/2000 Sb., zákoníku práce a zákona č. 373/2011 Sb.

## Rozhodovací workflow

1. Popiš skutečně vykonávanou práci, pracoviště, režim směn a dobu expozice.
2. Rozepiš každý rozhodující faktor pracovních podmínek zvlášť.
3. Ověř výsledky měření nebo odborného hodnocení v charakteristické směně.
4. Urči, kdo práci zařazuje a zda jde o rizikovou práci.
5. Z mapy faktorů odvoď obsah žádosti o pracovnělékařskou prohlídku, nikoli opačně.
6. Urči skutečného zaměstnavatele posuzované osoby a vztahy na sdíleném pracovišti.
7. Zkontroluj druh prohlídky, její lhůtu, zvláštní vyšetření a poskytovatele pracovnělékařských služeb.
8. Výstup rozděl na rozhodnutí orgánu, povinnosti zaměstnavatele, povinnosti provozovatele pracoviště a doklady.

## Kategorizace práce

- § 37 zákona č. 258/2000 Sb. váže kategorii na výskyt faktorů a jejich rizikovost, ne jen na název pozice.
- Práci do druhé kategorie zařazuje zaměstnavatel, nestanoví-li zvláštní předpis jinak.
- O zařazení do třetí nebo čtvrté kategorie rozhoduje příslušný orgán ochrany veřejného zdraví.
- Žádost pro třetí nebo čtvrtou kategorii musí pracovat s výsledky hodnocení expozice, dobou expozice, délkou směny, režimem směn a přijatými opatřeními.
- Změna technologie, látky, organizace práce, doby expozice nebo směn může vyvolat nové posouzení.
- Kategorie patří ke konkrétní práci v konkrétních podmínkách. Nelze ji mechanicky přenést z názvu pozice u jiného zaměstnavatele.

## Informování zaměstnance

- § 103 odst. 1 písm. b) zákoníku práce ukládá zaměstnavateli informovat zaměstnance, do jaké kategorie byla jeho práce zařazena.
- Ustanovení samo neurčuje zvláštní formu sdělení.
- Pro kontrolovatelnost doporuč písemný nebo jinak doložitelný záznam, ale neprezentuj doporučenou formu jako výslovnou zákonnou podmínku.
- Sdělení kategorie nenahrazuje informování o konkrétních rizicích, opatřeních, prohlídkách a poskytovateli pracovnělékařských služeb.

## Pracovníci dodavatele

- Pracovnělékařské služby zajišťuje pro zaměstnance jeho zaměstnavatel podle § 53 odst. 2 zákona č. 373/2011 Sb.
- To, že dodavatel dosud nemá vlastní pozici označenou kategorií 3, neřeší faktickou expozici jeho zaměstnanců na cizí lince.
- Provozovatel pracoviště a dodavatel si podle § 101 zákoníku práce písemně předávají rizika a přijatá opatření a spolupracují.
- Provozovatel musí podle § 103 odst. 1 písm. g) zajistit, aby pracovníci jiného zaměstnavatele před zahájením práce dostali vhodné pokyny k rizikům jeho pracoviště.
- Odpověď nesmí sloučit odpovědnost za prohlídku zaměstnance s odpovědností za předání místních rizik.

## Pracovnělékařské prohlídky

- U práce kategorie druhé, druhé rizikové, třetí nebo čtvrté se vstupní prohlídka posuzuje podle § 59 zákona č. 373/2011 Sb.
- Písemná smlouva s poskytovatelem pracovnělékařských služeb se řeší podle § 54 téhož zákona a podle skutečné kategorie a profesního rizika.
- Periodicitu neurčuj jen podle věku. Porovnej kategorii práce, profesní riziko, zvláštní předpis a přílohu vyhlášky č. 79/2013 Sb.; použije se kratší relevantní lhůta.
- Zvláštní vyšetření neodvozuj pouze ze státní příslušnosti. Ověř přesnou položku přílohy vyhlášky, pracovní expozici a případné epidemiologické kritérium.
- Žádost o prohlídku musí popsat práci, režim, rizikové faktory a míru expozice tak, aby poskytovatel neposuzoval fiktivní obecnou pozici.

## cdx-cli postup

```bash
cdx-cli get cdx://cz_law/258/2000/versions
cdx-cli get 'cdx://cz_law/258/2000/text?part=paragraf37'
cdx-cli get 'cdx://cz_law/262/2006/text?part=paragraf101'
cdx-cli get 'cdx://cz_law/262/2006/text?part=paragraf103'
cdx-cli get 'cdx://cz_law/373/2011/text?part=paragraf53'
cdx-cli get 'cdx://cz_law/373/2011/text?part=paragraf54'
cdx-cli get 'cdx://cz_law/373/2011/text?part=paragraf59'
cdx-cli get cdx://cz_law/79/2013/versions
```

## Pasti

- Kategorie určená jen názvem pracovní pozice.
- Zaměnění návrhu zaměstnavatele za rozhodnutí orgánu ochrany veřejného zdraví.
- Stejná periodicita prohlídky pro každou osobu obsluhující stejné zařízení.
- Přenesení povinnosti dodavatele zajistit prohlídku na provozovatele pracoviště.
- Přehlédnutí samostatné povinnosti provozovatele předat místní rizika.
- Vydávání doporučeného písemného potvrzení za jedinou zákonnou formu informování.
