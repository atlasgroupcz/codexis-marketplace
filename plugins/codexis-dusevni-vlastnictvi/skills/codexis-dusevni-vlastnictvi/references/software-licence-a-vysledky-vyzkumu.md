# Software, licence třetích stran a výsledky výzkumu

## Scope

Použij pro nákup a vývoj software, OEM a předplatné třetích stran, dlouhodobý servis, podlicence, zaměstnanecké programy a databáze, školní a zaměstnanecká díla, AI výstupy a smluvní zajištění řetězce práv.

## Výzkumný podklad

Reference vychází z tematického korpusu chatů monitoru Ochrana duševního vlastnictví od 1. 1. 2026. Opakovala se dodávka software s licencemi výrobce, dlouhodobý servis při ročním předplatném, zaměstnanecké programy a databáze, dizertační a zaměstnanecká díla, podlicence technických specifikací, AI ve smlouvách a rozsah licence na dobu trvání majetkových práv.

## Inventura předmětu plnění

Rozděl systém alespoň na:

- software vytvořený dodavatelem pro objednatele,
- předexistující komponenty dodavatele,
- proprietární software třetích stran,
- open-source komponenty,
- databáze, datové sady a dokumentaci,
- grafiku, texty, školení a jiné autorské výstupy,
- konfigurace, integrace a provozní know-how,
- výstupy vytvořené s pomocí AI.

Pro každou položku určuj autora nebo výrobce, nositele a vykonavatele práv, licenční zdroj, oprávněného uživatele, rozsah užití, území, dobu, počet zařízení nebo uživatelů, převoditelnost, podlicenci, změny, audit a ukončení.

## Licence třetích stran

- Označení „OEM“, „perpetuální“, „proprietární“ a „subscription“ popisují různé vlastnosti. Jedno slovo nenahrazuje celý licenční rozsah.
- Dodavatel nemůže objednateli poskytnout více práv, než získal od výrobce.
- U ročního předplatného a dlouhodobého servisu určuj, kdo licenci obnovuje, kdo nese růst ceny, co se stane při ukončení produktu a jak funguje migrace.
- Vyžaduj seznam komponent, licenční metriky, verze podmínek, doklad oprávnění a povinnost oznamovat změny.
- U kritické komponenty řeš kontinuitu: escrow, export dat, náhradní řešení, přechodnou licenci nebo součinnost při migraci.
- Formulace „licence je součástí ceny“ nestačí, pokud není jasné období, metrika a závislost na předplatném třetí strany.

## Zaměstnanecké a školní výstupy

- U každého výstupu určuj, zda vznikl ke splnění pracovních povinností, studijních povinností, obojího, nebo mimo ně.
- Vědecká publikace vytvořená při zaměstnání a dizertační práce vytvořená jako studijní výstup se neposuzují automaticky stejně.
- Pro zaměstnanecké dílo otevři § 58 autorského zákona; pro školní dílo § 60; pro počítačové programy a databáze také zvláštní pravidla autorského zákona.
- Osobnostní autorská práva, majetková práva a výkon majetkových práv jsou rozdílné kategorie.
- Trojstranná dohoda musí popsat konkrétní budoucí užití, publikaci, licenci, odměnu, embargo, otevřený přístup, podkladová data a vztah k povinnostem školy.
- Neuzavírej závěr o převoditelnosti nebo kogentnosti jen podle názoru smluvních stran; ověř přesné zákonné pravidlo pro daný typ výstupu.

## AI ve smlouvě

- Neuváděj kategoricky, že každý AI výstup je bez autorskoprávní ochrany. Zkoumej lidský tvůrčí vklad a konkrétní výsledek.
- Smlouva má oddělit vstupy do AI, oprávnění k jejich použití, osobní údaje a důvěrné informace, trénování poskytovatelem, evidenci použitého nástroje, kontrolu výstupu a odpovědnost za zásah do práv třetích osob.
- Vyžaduj informaci o použití AI tam, kde ovlivňuje původ, licenční jistotu nebo možnost výhradního užití výstupu.
- Záruka originality bez pravidel pro třetí komponenty, modelové podmínky a lidskou kontrolu je nedostatečná.

## cdx-cli postup

```bash
cdx-cli get cdx://cz_law/121/2000/versions
cdx-cli get 'cdx://cz_law/121/2000/text?part=paragraf58'
cdx-cli get 'cdx://cz_law/121/2000/text?part=paragraf60'
cdx-cli get 'cdx://cz_law/121/2000/text?part=paragraf66'
cdx-cli get 'cdx://cz_law/89/2012/text?part=paragraf2358'
cdx-cli search JD --query 'zaměstnanecké dílo počítačový program databáze výkon majetkových práv' --limit 10
cdx-cli search JD --query 'školní dílo dizertační práce licence' --limit 10
```

## Pasti

- Jedna licence pro celý technicky složený systém.
- Zaměnění vlastnictví hardware za licenci k software.
- „Na dobu neurčitou“ zaměněné za dobu trvání majetkových práv.
- Přenesení rizika obnovy předplatného na objednatele bez cenového a exit mechanismu.
- Automatické přiznání zaměstnavatelského režimu každému výstupu zaměstnance-studenta.
- Absolutní tvrzení o nechráněnosti AI výstupu bez analýzy lidské tvorby.
