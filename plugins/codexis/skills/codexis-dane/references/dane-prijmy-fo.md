# Daň z příjmů fyzických osob (DPFO) — metodika odpovědí

## Scope

Použij tuto metodiku, když uživatel řeší zdanění fyzických osob — kombinaci příjmů (zaměstnání, OSVČ, nájem, investice, ostatní), volbu mezi paušálními a skutečnými výdaji, podání daňového přiznání, slevy a odpočty, sazby daně FO. Nepoužívej pro DPH, daň z nemovitostí, spotřební daně ani procesní otázky nenavázané na DPFO (pro procesní vrstvu viz `dane.md`).

Výchozí hmotný předpis: **ZDP** (č. 586/1992 Sb.). Procesní nadstavbu řeší **daňový řád** (č. 280/2009 Sb.).

## Úplný workflow (kontrolní průchod)

Při odpovědi na praktický dotaz k DPFO („jak správně podat přiznání", „jak uplatnit výdaje", „co mám řešit") projdi těchto šest vrstev a žádnou tiše nevynechávej. Pokud některá vrstva není pro danou otázku relevantní, napiš to výslovně.

1. **Klasifikace příjmu do dílčího základu daně** (§ 6–10 ZDP)
2. **Výpočet dílčího základu** (výdaje paušálem / skutečné / specifické pravidlo)
3. **Procesní/evidenční vrstva u skutečných výdajů** (doložení, daňová evidence)
4. **Součet dílčích základů → odčitatelné položky** (§ 15 ZDP)
5. **Aplikace sazby na upravený základ** (§ 16 ZDP, progresivní práh)
6. **Slevy na dani a daňové zvýhodnění** (§ 35ba, § 35c ZDP)

Pokud odpověď u DPFO implikuje „jak se to zdaní", nezastav se u klasifikace příjmu (krok 1). Sama klasifikace („nájem → § 9") větu neuzavírá — odpověď musí dojít k sazbě (krok 5) a vrstvě slev (krok 6), když to má pro uživatele praktický smysl.

## Upřesňující otázky (když je dotaz nekonkrétní)

- „Máte za daný rok dary, úroky z hypotéky na vlastní bydlení nebo příspěvky na penzijní/spořicí produkty, které by mohly snížit základ daně?" — dobrý pre-check k § 15.
- „Budete v přiznání uplatňovat jen základní slevu na poplatníka, nebo i další slevy či zvýhodnění na děti/manžela/invaliditu?" — odemkne § 35ba / § 35c.
- „U příjmů z podnikání — potřebujete řešit jen způsob evidence výdajů, nebo i daňovou uznatelnost konkrétních nákladů?" — otevře § 24 / § 25.
- „Vedete daňovou evidenci nebo účetnictví, a máte k výdajům doklady?" — odkazuje na § 7b.
- „Máte orientační představu, zda celkový roční základ daně zůstane pod prahem pro nižší sazbu?" — otevře § 16 odst. 1.

Jedna konkrétní upřesňující otázka před širokou rešerší je vždy lepší než spekulativní odpověď.

## Mapa zákonů (§§)

**ZDP (zákon č. 586/1992 Sb.)** — klíčová ustanovení podle vrstvy:

- Dílčí základy: § 6 (závislá činnost), § 7 (samostatná činnost), § 7b (daňová evidence OSVČ bez účetnictví), § 8 (kapitálový majetek), § 9 (nájem), § 10 (ostatní příjmy)
- Uznatelnost výdajů: § 24 (co lze uznat — „výdaje na dosažení, zajištění a udržení zdanitelných příjmů"), § 25 (co uznat nelze)
- Odčitatelné položky (po součtu dílčích základů): § 15 (dary, úroky z hypotéčního úvěru, penzijní a životní pojištění)
- Sazba: § 16 (dvě sazby s progresivním prahem navázaným na násobek průměrné mzdy — aktuální znění ověř, nespoléhej na paměť)
- Slevy a zvýhodnění: § 35ba (slevy na dani — poplatník, manžel, invalidita, student), § 35c (daňové zvýhodnění na vyživované dítě)
- Podací povinnost FO: § 38g (kdy vzniká povinnost podat přiznání)

Při parafrázi statutární definice zachovej právně operativní kvalifikátory („alespoň", „nejvýše", „započatých", „po sobě následujících") — jejich vypuštění je věcná chyba, ne stylistické zjednodušení.

## Poznámka k procesní vrstvě

Povinnost podat přiznání je pouze jedna strana mince. I když podací povinnost z § 38g nevzniká, fakultativní podání zůstává otevřené (viz `dane.md`) a může přinést uplatnění slev/odpočtů a vratku přeplatku. Kdykoli závěr u DPFO zní „povinnost nevzniká", doplň krátkou větev o fakultativním podání, pokud má pro uživatele praktický smysl.

## cdx-cli tipy

Pro konkrétní paragraf aktuální verze ZDP:

```bash
cdx-cli get cdx://cz_law/586/1992/versions                      # seznam verzí ZDP
cdx-cli get 'cdx://doc/<verze>/text?part=paragraf24'            # plné znění § 24
cdx-cli get 'cdx://doc/<verze>/toc' | rg 'paragraf15|paragraf35ba|paragraf35c'
```

Když znáš zákon i paragraf, nepoužívej broad keyword search (`search CR --query ...`) — rovnou sáhni na `cdx://doc/<verze>/text?part=paragrafN`. Keyword search vyhraď pro situace, kdy hledáš, ve kterém zákoně pravidlo je, nebo kdy ustanovení zasahuje napříč více předpisy.

Pro sazbu a progresivní práh dle § 16 navíc ověř aktuální hodnotu násobku průměrné mzdy — hranice je časově citlivá, nepracuj s číslem z paměti:

```bash
cdx-cli get 'cdx://doc/<aktuální_verze_ZDP>/text?part=paragraf16'
```

## Pasti (anti-patterny)

- **Zastavit se u klasifikace příjmu** („je to nájem → § 9") a neodvodit sazbu, slevy ani procesní vrstvu, když se dotaz ptá na celý postup zdanění.
- **Odpovědět jen procenty paušálů** bez doplnění, že u skutečných výdajů je klíčová uznatelnost (§ 24) a evidenční povinnost (§ 7b).
- **Zapomenout odčitatelné položky a slevy** po výpočtu dílčích základů. Tahle vrstva je skoro vždy relevantní a její opomenutí je nejčastější věcná chyba.
- **Odpovědět jen jednou sazbou** bez zmínky progresivního prahu, když základ daně může hranici překročit.
- **Přeformulovat statutární definici a ztratit kvalifikátory** („alespoň šesti měsíců" → „asi šest měsíců") — to je věcná změna významu, ne stylistické čištění.
- **Říct „povinnost podat není"** a neotevřít fakultativní variantu, když má ekonomický smysl (vratka přeplatku, uplatnění odpočtů).
