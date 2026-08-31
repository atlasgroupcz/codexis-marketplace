---
uuid: 826a0cd8-d9e5-4607-9ba1-62b733df865d
name: codexis-cirkev
icon: icon.svg
version: 1.1.0
jurisdictions: [CZ, EU]
description: Use when a Czech legal question concerns churches, religious societies, registered church legal persons, clergy, religious freedom, church property, internal church rules, state registration, financing, restitution or the civil effects of canonical and other religious norms. Standalone church-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "Monitor - Církev"
    summary: "České církevní a náboženské právo, právnické osoby, majetek a duchovní služba."
    examplePrompts:
      - "Jak ověřit, že církevní právnická osoba může prodat konkrétní nemovitost a kdo za ni jedná?"
      - "Posuď právní vztah duchovního k církvi a hranice přezkumu civilními soudy."
      - "Jaké podmínky platí pro evidenci církevní právnické osoby a změnu jejích údajů?"
  en:
    displayName: "Monitor - Church Law"
    summary: "Czech church and religious law, legal persons, property and clergy service."
    examplePrompts:
      - "How can I verify that a church legal person may sell a particular property and who may act for it?"
      - "Assess the legal relationship between clergy and a church and the limits of civil-court review."
      - "What conditions apply to registering a church legal person and changing its registered data?"
  sk:
    displayName: "Monitor - Cirkev"
    summary: "České cirkevné a náboženské právo, právnické osoby, majetok a duchovná služba."
    examplePrompts:
      - "Ako overiť, že cirkevná právnická osoba môže predať konkrétnu nehnuteľnosť a kto za ňu koná?"
      - "Posúď právny vzťah duchovného k cirkvi a hranice preskúmania civilnými súdmi."
      - "Aké podmienky platia pre evidenciu cirkevnej právnickej osoby a zmenu jej údajov?"
---

# CODEXIS — Církev

Standalone skill pro české právo církví a náboženských společností.

## Operating Assumptions

- Pro legislativu, judikaturu a komentáře používej `cdx-cli`.
- Ústavní, zákonnou, soukromoprávní a vnitrocírkevní vrstvu vždy rozliš.
- Údaje o registraci a statutárním jednání ověř v příslušném veřejném registru.
- Církevní dokument označ podle toho, zda má civilní účinek, důkazní význam nebo jen interní působnost.

## Research Strategy

1. Identifikuj přesný subjekt, jeho kategorii, identifikátor, nadřízenou církev a rozhodné datum.
2. Odděl právní osobnost, vlastnictví, oprávnění jednat, plnou moc a interní schválení.
3. Otevři ústavní ochranu, zákon č. 3/2002 Sb. a zvláštní soukromoprávní nebo pracovněprávní předpis.
4. Ověř registr, listiny subjektu a případně katastr nemovitostí.
5. U autonomie, duchovních, majetku nebo diskriminace projdi českou i evropskou judikaturu.

## Klíčové předpisy

| Předpis | CODEXIS base | Použití |
|---|---|---|
| Listina základních práv a svobod | `CR` | náboženská svoboda a autonomie |
| zákon č. 3/2002 Sb. | `cz_law/3/2002` | registrace a postavení církví |
| zákon č. 428/2012 Sb. | `cz_law/428/2012` | majetkové vyrovnání |
| zákon č. 89/2012 Sb. | `cz_law/89/2012` | právnické osoby, vlastnictví a smlouvy |
| zákon č. 262/2006 Sb. | `cz_law/262/2006` | pracovněprávní vztahy |
| zákon č. 198/2009 Sb. | `cz_law/198/2009` | rovné zacházení a diskriminace |

## Hard Rules

- Název farnosti, sboru, řádu nebo jiné jednotky sám neprokazuje právní osobnost.
- Interní souhlas a oprávnění jednat navenek nejsou totéž.
- Církevní autonomie není absolutní a současně ji nelze redukovat na běžnou korporační samosprávu.
- Duchovenská služba nemusí mít stejný režim jako pracovní poměr; kvalifikuj konkrétní vztah.
- Majetkové vyrovnání podle zákona č. 428/2012 Sb. nepřenášej na jiný majetkový spor bez zvláštního testu.
- Tematický chatový korpus bez ratingu používej pro volbu workflow a zdrojů, ne jako důkaz správnosti právního závěru.

## Workflow církevního práva

1. **Subjekt.** Registrace, právní osobnost a vztah k zakládající nebo nadřízené církvi.
2. **Jednání.** Statutární orgán, interní omezení, plná moc a účinek vůči třetím osobám.
3. **Majetek.** Vlastník, nabývací titul, zvláštní omezení, restituční historie a katastr.
4. **Osoby.** Duchovní, zaměstnanec, člen, dobrovolník nebo jiný služební vztah.
5. **Autonomie.** Interní věc víry a správy versus civilně přezkoumatelný zásah do práv.
6. **Remedy.** Interní prostředek, správní přezkum, civilní soud, ústavní nebo evropská ochrana.

## Rejstříky a vnitřní právo

Při otázce na právní formu, charitu, farnost, sbor, eparchii, statutární jednání, kanonické právo nebo interní církevní předpis načti `references/pravni-osobnost-rejstriky-a-vnitrni-pravo.md`.

- Reference vychází z tematického korpusu chatů monitoru Církev od 1. 1. 2026; v období nebyl žádný uživatelský rating.
- Vždy odděl státní právní osobnost, vnější jednání, vnitřní oprávnění a právní následek porušení interního pravidla.
- Konkrétní církevní předpis aplikuj jen na subjekt a osobu, na které skutečně dopadá.

## Časté pasti

- Přiznání právní osobnosti organizační jednotce jen podle názvu.
- Podpis smlouvy bez ověření registru a interního schválení.
- Automatické použití zákoníku práce na každou službu duchovního.
- Označení interního církevního předpisu za obecně závazný právní předpis.
- Přehlédnutí majetkové historie a zvláštního restitučního režimu.

## Struktura odpovědi

1. Identifikace subjektu a právní osobnosti.
2. Ústavní a zákonný rámec.
3. Civilní nebo pracovní režim.
4. Vnitřní pravidlo a jeho vnější účinek.
5. Registr, vlastnictví, oprávnění jednat a důkazy.
6. Dostupný přezkum a otevřené otázky.

## Related references

- Právní osobnost, rejstříky a vnitřní církevní právo: `references/pravni-osobnost-rejstriky-a-vnitrni-pravo.md`.
