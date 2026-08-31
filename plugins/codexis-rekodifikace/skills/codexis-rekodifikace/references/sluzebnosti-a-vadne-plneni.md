# Služebnosti a vadné plnění v občanském zákoníku

## Scope

Použij pro vlastnické spory kolem služebnosti užívání a pro smlouvy o dílo nebo servis, kde se náklady na odstranění vady třetí osobou uplatňují jako náhrada škody.

## Výzkumný podklad

Reference vychází ze tří pozitivně hodnocených odpovědí monitoru Rekodifikace z období 1. 1.–31. 8. 2026. Jeden chat řešil přístup vlastníka a klíče při služebnosti užívání. Dva navazující chaty řešily nekvalitní pozáruční servis, opožděné vytknutí vady, opravu třetí osobou a § 1925 občanského zákoníku.

## Proč nejde jen o přechodná ustanovení

Monitor Rekodifikace v reálných chatech pokrývá také aplikaci institutů občanského zákoníku na současné soukromoprávní vztahy. Nevnucuj historický střet starého a nového práva, pokud všechna rozhodná fakta nastala za účinnosti zákona č. 89/2012 Sb.

## Služebnost užívání

1. Načti nabývací titul, přesný popis zatížené věci, rozsah užívání, oprávněnou osobu a zápis v katastru.
2. Podle § 1257 urči, co vlastník musí trpět nebo čeho se musí zdržet.
3. Není-li rozsah určen, pracuj s výkladovým pravidlem § 1258 a skutkovou historií výkonu.
4. U užívacího práva otevři § 1283 a § 1284 a odděl potřeby oprávněného od zbývajících oprávnění a povinností vlastníka.
5. Přístup vlastníka ani vydání klíčů neposuzuj abstraktně. Testuj účel přístupu, četnost, soukromí oprávněného, údržbu, havárii a možné rušení služebnosti.
6. Navrhni praktický režim oznámení vstupu, havarijního přístupu a předání klíčů jen v rozsahu přiměřeném titulu.

## Vadný servis a oprava třetí osobou

1. Kvalifikuj smlouvu a přesně popiš, co mělo být výsledkem servisu.
2. Odděl původní vadu stroje od vady servisního plnění a od nové následné škody.
3. Podle § 2615 určuj, zda dílo odpovídá smlouvě a jak se použijí práva z vadného plnění.
4. Sestav časovou osu zjištění vady, oznámení, volby nároku, dodatečné lhůty, odmítnutí nápravy a zásahu třetí osoby.
5. Podle § 1925 odděl újmu dosažitelnou právem z vadného plnění od samostatné následné škody.
6. Opožděné vytknutí vady neobcházej pouhým přejmenováním nákladů na opravu téže vady na škodu.
7. Ani zjevná neschopnost původního servisu sama neřeší celý nárok. Ověř smluvní self-help klauzuli, marnou dodatečnou lhůtu, volbu nároku, příčinnou souvislost a judikaturu ke konkrétnímu typu nákladu.

## Rozlišení nákladů

| Náklad | Otázka |
|---|---|
| odstranění téže vady | šlo jej dosáhnout právem z vadného plnění? |
| poškození jiné části věci | jde o samostatnou následnou škodu? |
| odstávka a ušlý zisk | jsou prokázány příčina, rozsah a předvídatelnost? |
| diagnostika a nouzové opatření | bylo účelné a vyvolané porušením? |
| práce třetího servisu | umožňovala ji smlouva nebo vznikla po marné nápravě? |

## cdx-cli postup

```bash
cdx-cli get cdx://cz_law/89/2012/versions
cdx-cli get 'cdx://cz_law/89/2012/text?part=paragraf1257'
cdx-cli get 'cdx://cz_law/89/2012/text?part=paragraf1258'
cdx-cli get 'cdx://cz_law/89/2012/text?part=paragraf1283'
cdx-cli get 'cdx://cz_law/89/2012/text?part=paragraf1284'
cdx-cli get 'cdx://cz_law/89/2012/text?part=paragraf1925'
cdx-cli get 'cdx://cz_law/89/2012/text?part=paragraf2615'
cdx-cli search JD --query '§ 1925 náklady odstranění vady třetí osobou servis' --limit 10
```

## Pasti

- Automatické právo vlastníka vstupovat bez omezení do prostoru zatíženého užívacím právem.
- Automatický zákaz jakéhokoli přístupu vlastníka.
- Určení rozsahu služebnosti bez nabývacího titulu a historie výkonu.
- Smíchání původní vady věci, vady servisu a následné škody.
- Náhrada škody použitá jako náhradní cesta za neuplatněné právo z vady.
- Kategorický závěr bez kontroly smluvního práva opravit vadu třetí osobou.
