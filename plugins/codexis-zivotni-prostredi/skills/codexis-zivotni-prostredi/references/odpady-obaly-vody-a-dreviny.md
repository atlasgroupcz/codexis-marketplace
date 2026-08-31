# Odpady, obaly, odpadní vody a dřeviny

## Scope

Použij pro praktické dotazy obcí a podniků na hranici výrobku a odpadu, obalové povinnosti, zařazování odpadu, bezodtokové jímky a výjimky z povolení ke kácení dřevin.

## Výzkumný podklad

Reference vychází z pozitivně hodnocených chatů monitoru Životní prostředí z období 1. 1.–31. 8. 2026. Opakovala se prodejnost použitelných a poškozených palet, obalová výjimka pro malé množství, katalogové zařazení obuvi, kontrola odvozu odpadních vod z jímek, kompetence městských částí a kácení v ochranném pásmu energetického zařízení.

## Výrobek, obal nebo odpad

1. Identifikuj věc, její stav, původní účel, držitele a zamýšlený další krok.
2. Podle § 4 zákona č. 541/2020 Sb. testuj, zda se jí osoba zbavuje, má úmysl nebo povinnost se jí zbavit.
3. Ověř, zda je věc reálně použitelná k původnímu účelu bez předstíraného využití.
4. U obalu odděl opakované použití obalu, uvedení obalu na trh nebo do oběhu a vznik odpadu z obalu.
5. Teprve po klasifikaci řeš prodej, předání, zařízení, evidenci a katalogové číslo.

Použitelná paleta se nestává odpadem jen proto, že ji původní podnik prodá. Poškozená paleta určená pouze k odstranění nebo rozebrání může být odpadem. Rozhodují skutečné vlastnosti a úmysl držitele, ne název účetní operace.

## Malé množství obalů

- § 15a zákona č. 477/2001 Sb. vyžaduje současné splnění dvou podmínek: nejvýše 300 kg obalů za kalendářní rok a roční obrat nejvýše 25 000 000 Kč.
- Výjimka se týká povinností podle § 10 až 15, nikoli automaticky všech povinností zákona o obalech.
- Podmínky musí osoba na požádání prokázat.
- Jakmile je zřejmé, že některá podmínka v daném roce splněna nebude, povinnosti podle § 10 až 15 se začnou plnit neprodleně.
- U vývozu nebo přeshraničního dodání odděl české uvedení na trh od povinností v cílovém státě.

## Katalogové zařazení odpadu

- Kód neurčuj jen podle obchodního názvu výrobku.
- Pracuj s původem odpadu, materiálem, činností, při níž vznikl, a strukturou Katalogu odpadů.
- Pokud katalog nemá výslovnou položku pro smíšený výrobek, nevydávej analogickou položku za bezvýhradně jistou.
- U obuvi rozliš sběr textilu, materiálové složení a provozní řešení příjemce. Zachovej nejistotu, pokud se právní a provozní klasifikace rozcházejí.

## Bezodtokové jímky

- § 38 odst. 7 vodního zákona ukládá tomu, kdo akumuluje odpadní vody v bezodtokové jímce, zajistit odvoz na čistírnu odpadních vod.
- Doklady za poslední dva kalendářní roky se předkládají na výzvu vodoprávního úřadu nebo České inspekce životního prostředí.
- Před závěrem, že kontrolu může provádět obec, ověř, zda konkrétní obecní úřad vykonává pro danou věc působnost vodoprávního úřadu.
- Odděl obec jako vlastnickou nebo samosprávnou entitu od obecního úřadu vykonávajícího přenesenou působnost.
- Z dokladu ověř osobu, lokalizaci jímky, množství, datum odvozu, odvozce a čistírnu.

## Kácení dřevin

- Výchozím pravidlem § 8 odst. 1 zákona č. 114/1992 Sb. je povolení orgánu ochrany přírody.
- U § 8 odst. 2 ověř přesný zákonný důvod, osobu provádějící zásah a souvislost s provozováním vyjmenovaného zařízení.
- Odstranění dřevin v ochranném pásmu zařízení elektrizační nebo plynárenské soustavy při provozování soustavy nepodléhá povolení, ale písemnému oznámení nejméně 15 dnů předem.
- Usnesení obce nebo souhlas vlastníka nevytváří zákonnou výjimku a nenahrazuje oznámení.
- Havarijní kácení podle § 8 odst. 4 má jiný skutkový test a následné oznámení; nemíchej je s předchozím oznámením podle odstavce 2.

## cdx-cli postup

```bash
cdx-cli get cdx://cz_law/541/2020/versions
cdx-cli get 'cdx://cz_law/541/2020/text?part=paragraf4'
cdx-cli get 'cdx://cz_law/477/2001/text?part=paragraf15a'
cdx-cli get cdx://cz_law/8/2021/versions
cdx-cli get 'cdx://cz_law/254/2001/text?part=paragraf38'
cdx-cli get 'cdx://cz_law/114/1992/text?part=paragraf8'
```

## Pasti

- Označení věci za odpad jen proto, že ji podnik nechce dále používat sám.
- Obalová výjimka odvozena pouze z hmotnosti bez kontroly obratu.
- Výjimka podle § 15a vydávaná za osvobození od celého zákona o obalech.
- Katalogové číslo určené pouze podle názvu výrobku.
- Obec zaměněná za věcně příslušný vodoprávní úřad.
- Souhlas rady obce vydávaný za právní titul ke kácení.
