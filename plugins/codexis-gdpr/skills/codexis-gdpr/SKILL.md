---
uuid: acec0787-0c11-43f7-b43d-95d046f73c07
name: codexis-gdpr
icon: icon.webp
version: 1.1.0
jurisdictions: [CZ, EU]
description: Use when a Czech or EU legal question concerns GDPR, personal data, privacy, controllers, processors, employee monitoring, cameras, cookies, data-subject rights, DPIA, data breaches, security, retention, profiling, automated decisions or international transfers. Standalone data-protection methodology using CODEXIS.
i18n:
  cs:
    displayName: "Monitor - Ochrana osobních údajů"
    summary: "GDPR a české právo ochrany osobních údajů, zpracování, bezpečnost a práva subjektů."
    examplePrompts:
      - "Posuď právní přípustnost kamerového monitoringu zaměstnanců."
      - "Musíme tento bezpečnostní incident oznámit ÚOOÚ a dotčeným osobám?"
      - "Prověř smlouvu se zpracovatelem a předávání osobních údajů mimo EHP."
  en:
    displayName: "Monitor - Data Protection"
    summary: "GDPR and Czech data protection law, processing, security and data-subject rights."
    examplePrompts:
      - "Assess whether employee camera monitoring is legally permissible."
      - "Must we report this personal-data breach to the authority and affected people?"
      - "Review our processor agreement and transfer of personal data outside the EEA."
  sk:
    displayName: "Monitor - Ochrana osobných údajov"
    summary: "GDPR a české právo ochrany osobných údajov, spracúvanie, bezpečnosť a práva dotknutých osôb."
    examplePrompts:
      - "Posúď právnu prípustnosť kamerového monitorovania zamestnancov."
      - "Musíme tento bezpečnostný incident oznámiť dozornému orgánu a dotknutým osobám?"
      - "Prever zmluvu so sprostredkovateľom a prenos osobných údajov mimo EHP."
---

# CODEXIS - Ochrana osobních údajů

Standalone skill pro GDPR a českou ochranu osobních údajů.

## Operating Assumptions

- Pro právní zdroje používej `cdx-cli` v kolekcích `EU`, `CR`, `JD`, `COMMENT` a `LT`.
- Pokyny EDPB a dozorových úřadů označ podle jejich právní váhy.
- Odkaz na rozhodnutí musí být funkční a odpověď musí obsahovat finální syntézu, ne raw tool output.
- Právní titul neurčuj podle názvu formuláře nebo smlouvy, ale podle skutečného účelu a rolí.

## Research Strategy

1. Namapuj konkrétní zpracování: údaje, osoby, zdroj, účel, operace, příjemce, systém, místo, dobu a důsledky.
2. Urči správce, společné správce a zpracovatele podle skutečného rozhodování.
3. Pro každý účel samostatně ověř působnost, zásady a právní titul.
4. Zvlášť testuj citlivé údaje, trestní údaje, děti, zaměstnance a elektronické komunikace.
5. Projdi transparentnost, práva, zabezpečení, záznamy, DPIA, smlouvy, předávání a incidenty.

## Klíčové předpisy

| Předpis | Zdroj | Použití |
|---|---|---|
| nařízení (EU) 2016/679, GDPR | `EU` | obecný rámec zpracování |
| zákon č. 110/2019 Sb. | `cz_law/110/2019` | česká adaptační úprava |
| zákon č. 480/2004 Sb. | `cz_law/480/2004` | obchodní sdělení a online služby |
| zákon č. 127/2005 Sb. | `cz_law/127/2005` | elektronické komunikace |
| zákon č. 262/2006 Sb. | `cz_law/262/2006` | zaměstnanecké vztahy a monitoring |
| zákon č. 372/2011 Sb. | `cz_law/372/2011` | zdravotní údaje a dokumentace |
| zákon č. 106/1999 Sb. | `cz_law/106/1999` | střet informací a soukromí |

## Hard Rules

- Právní titul přiřaď každému účelu, ne celé databázi jedním štítkem.
- Souhlas nepoužívej jako default, pokud zpracování stojí na smlouvě, zákonu nebo oprávněném zájmu.
- U oprávněného zájmu proveď účel, nezbytnost a vyvažování.
- Minimalizace, retenční doba, přístup a zabezpečení jsou samostatné testy.
- Veřejně dostupný údaj není automaticky volně použitelný pro jiný účel.
- Smluvní označení „zpracovatel“ neprokazuje roli podle GDPR.
- U seznamu judikatury ověř datum, spisovou značku, účastníky, relevance a odkaz každé položky.

## Workflow DPO

1. **Processing map.** Datový tok od získání po výmaz.
2. **Roles.** Kdo určuje účel a prostředky, kdo jedná podle pokynů a kdo je příjemce.
3. **Lawfulness.** Čl. 6, případně čl. 9 nebo 10, a zvláštní český předpis.
4. **Information and rights.** Čl. 12 až 22 a ověřené výjimky.
5. **Accountability.** Záznamy, zpracovatelská smlouva, DPIA, DPO a interní důkazy.
6. **Security and breach.** Riziko, opatření, detekce, lhůta, ÚOOÚ a informování osob.
7. **Transfer.** EHP, rozhodnutí o odpovídající ochraně, záruka a doplňková opatření.

## Minimalizace v konkrétních agendách

Při kopii osobního dokladu, personální evidenci, výživovém poradenství, zdravotních údajích nebo školním profilu na sociální síti načti `references/minimalizace-a-zvlastni-kategorie.md`.

- Reference vychází z pozitivně hodnocených chatů monitoru z období 1. 1.-31. 8. 2026.
- Odděl zákonnou potřebu konkrétního údaje od uchovávání kopie celého dokumentu.
- Povahu údaje o zdraví neurčuje jen to, zda službu poskytuje zdravotnický poskytovatel.
- Skutečné nastavení komentářů, zpráv, statistik a publikovaného obsahu musí odpovídat textu transparentnosti sociální sítě.

## Časté pasti

- Jediný právní titul pro více odlišných účelů.
- Souhlas zaměstnance bez testu svobodnosti.
- Oznámení každého incidentu bez vyhodnocení rizika nebo naopak automatické neoznámení.
- Cookie analýza jen podle GDPR bez předpisu elektronických komunikací.
- Anonymizace bez ověření možnosti zpětné identifikace.
- Dlouhý seznam rozhodnutí s nefunkčními odkazy a bez odpovědi na požadovaný formát.
- Převzetí feedbacku tematicky mimo GDPR jako doménového právního pravidla.

## Struktura odpovědi

1. Popis zpracování a role.
2. Účel, právní titul a zvláštní kategorie.
3. Nezbytnost, minimalizace, retence a přístup.
4. Transparentnost a práva osob.
5. Zabezpečení, DPIA, incident a předávání.
6. Praktické kroky a ověřené odkazy.

## Related references

- Minimalizace údajů a zvláštní kategorie v praxi: `references/minimalizace-a-zvlastni-kategorie.md`.
