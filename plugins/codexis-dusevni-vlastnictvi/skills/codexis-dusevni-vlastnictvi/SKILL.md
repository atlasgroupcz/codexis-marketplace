---
uuid: 80d7a381-c4e3-41d4-9d08-fc148762d845
name: codexis-dusevni-vlastnictvi
icon: icon.svg
version: 1.1.0
jurisdictions: [CZ, EU]
description: Use when a Czech or EU legal question concerns copyright, software, databases, patents, utility models, employee inventions, trademarks, designs, trade secrets, know-how, licences, collective management, unfair competition or enforcement of intellectual property rights. Standalone IP methodology using CODEXIS.
i18n:
  cs:
    displayName: "Monitor - Ochrana duševního vlastnictví"
    summary: "Autorské, patentové, známkové a další právo duševního vlastnictví v ČR a EU."
    examplePrompts:
      - "Rozliš zaměstnanecké dílo a zaměstnanecký vynález a popiš práva zaměstnavatele."
      - "Prověř, zda naše licence pokrývá úpravy a další distribuci softwaru."
      - "Jaké nároky lze uplatnit při neoprávněném užití ochranné známky?"
  en:
    displayName: "Monitor - Intellectual Property"
    summary: "Czech and EU copyright, patent, trademark and other intellectual property law."
    examplePrompts:
      - "Distinguish an employee work from an employee invention and explain the employer's rights."
      - "Check whether our licence covers modifying and redistributing the software."
      - "What remedies are available for unauthorized use of a trademark?"
  sk:
    displayName: "Monitor - Ochrana duševného vlastníctva"
    summary: "České a európske autorské, patentové, známkové a ďalšie právo duševného vlastníctva."
    examplePrompts:
      - "Rozlíš zamestnanecké dielo a zamestnanecký vynález a opíš práva zamestnávateľa."
      - "Prever, či naša licencia pokrýva úpravy a ďalšiu distribúciu softvéru."
      - "Aké nároky možno uplatniť pri neoprávnenom použití ochrannej známky?"
---

# CODEXIS - Ochrana duševního vlastnictví

Standalone skill pro české a evropské právo duševního vlastnictví.

## Operating Assumptions

- Pro legislativu, judikaturu, komentáře a literaturu používej `cdx-cli`.
- Registrovaná práva ověřuj v příslušném českém, unijním nebo mezinárodním registru.
- U online užití vždy ověř území, platformu, smluvní podmínky a přeshraniční prvek.
- Tool calls jsou interní práce; uživatel musí dostat dokončenou odpověď, ne serializovaný nástrojový payload.

## Research Strategy

1. Identifikuj výsledek, tvůrce, objednatele nebo zaměstnavatele, datum, území, užití a smluvní řetězec.
2. Klasifikuj samostatně autorské právo, software, databázi, patent, užitný vzor, design, ochrannou známku, obchodní tajemství a nekalou soutěž.
3. U každého režimu určuj vznik nebo registraci, nositele, výkon práv, trvání, území, omezení a nároky.
4. Ověř českou, unijní a případně mezinárodní vrstvu k rozhodnému datu.
5. U licence nebo převodu projdi rozsah, výhradnost, území, dobu, podlicenci, změny a ukončení.

## Klíčové předpisy

| Předpis | CODEXIS base | Použití |
|---|---|---|
| zákon č. 121/2000 Sb. | `cz_law/121/2000` | autorské právo, software a databáze |
| zákon č. 527/1990 Sb. | `cz_law/527/1990` | vynálezy a patenty |
| zákon č. 478/1992 Sb. | `cz_law/478/1992` | užitné vzory |
| zákon č. 207/2000 Sb. | `cz_law/207/2000` | průmyslové vzory |
| zákon č. 441/2003 Sb. | `cz_law/441/2003` | ochranné známky |
| zákon č. 452/2001 Sb. | `cz_law/452/2001` | označení původu a zeměpisná označení |
| zákon č. 89/2012 Sb. | `cz_law/89/2012` | licence, obchodní tajemství a nekalá soutěž |
| nařízení (EU) 2017/1001 | `EU` | ochranná známka EU |

## Hard Rules

- Jeden výsledek může být chráněn více režimy současně.
- Autor, nositel majetkových práv, vykonavatel práv a vlastník nosiče nejsou automaticky tatáž osoba.
- Vlastnictví kopie nedává právo dílo rozmnožovat, sdělovat nebo měnit.
- U zaměstnaneckého nebo objednaného výsledku nejdřív klasifikuj typ práva; pravidla se liší pro dílo, software a průmyslové právo.
- Výjimku nebo zákonnou licenci vykládej pro konkrétní způsob užití, ne podle uživatelova označení.
- Při požadavku na „veškerou judikaturu“ dolož vyhledávací strategii, duplicity a hranici úplnosti.

## Workflow IP specialisty

1. **Předmět.** Co přesně vzniklo a které prvky jsou chránitelné.
2. **Vznik a priorita.** Automatická ochrana, přihláška, registrace, zveřejnění a novost.
3. **Nositel.** Autor, zaměstnavatel, objednatel, nabyvatel nebo společný nositel.
4. **Užití.** Rozmnožení, rozšiřování, sdělování, zpracování, těžba databáze nebo označení v obchodním styku.
5. **Oprávnění.** Licence, výjimka, vyčerpání, souhlas nebo zákonné omezení.
6. **Zásah a nároky.** Aktivní legitimace, důkazy, zákaz, informace, odstranění následků, peněžité nároky a předběžné opatření.

## Judikatura a web scraping

Při rešerši judikatury nebo web/data scrapingu načti `references/judikatura-a-web-scraping.md`.

## Software a výsledky výzkumu

Při nákupu nebo vývoji software, licenci třetí strany, zaměstnaneckém programu či databázi, školním díle, podlicenci nebo smluvní práci s AI načti `references/software-licence-a-vysledky-vyzkumu.md`.

- Reference vychází z tematického korpusu chatů monitoru z období 1. 1.-31. 8. 2026.
- Složený systém vždy rozpadni na vlastní vývoj, předexistující komponenty, třetí licence, open source, data, dokumentaci a AI výstupy.
- U zaměstnance-studenta klasifikuj každý výstup podle okolností jeho vzniku; nepřenášej automaticky režim publikace na dizertaci nebo opačně.

## Časté pasti

- Závěr jen podle autorského zákona bez testu databázového práva, smlouvy a nekalé soutěže.
- Převod vlastnictví nosiče zaměněný za licenci.
- Ochranná známka ověřená jen podle názvu bez registru a tříd.
- Licence bez územního a časového rozsahu.
- Seznam judikatury bez odstranění duplicit a ověření, proč je každé rozhodnutí relevantní.
- Vrácení raw tool calls místo odpovědi.

## Struktura odpovědi

1. Klasifikace všech možných práv.
2. Nositel a řetězec oprávnění.
3. Chráněné jednání a aplikovaná výjimka nebo licence.
4. Doba, území a registrace.
5. Zásah, důkazy, obrany a nároky.
6. Limity rešerše a přesné citace.

## Related references

- Judikatura a web scraping: `references/judikatura-a-web-scraping.md`.
- Software, licence třetích stran a výsledky výzkumu: `references/software-licence-a-vysledky-vyzkumu.md`.
