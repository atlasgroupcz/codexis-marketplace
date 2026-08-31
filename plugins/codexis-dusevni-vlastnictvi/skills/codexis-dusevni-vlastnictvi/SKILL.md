---
uuid: 80d7a381-c4e3-41d4-9d08-fc148762d845
name: codexis-dusevni-vlastnictvi
icon: icon.svg
version: 1.0.0
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

# Monitor - Ochrana duševního vlastnictví

Use `cdx-cli` for legal sources and official Czech, EU and international IP registers when the existence or status of a registered right matters.

## Mandatory analysis

1. Identify the concrete result, its creators, commissioning or employment context, creation date, territory, use and contractual chain.
2. Classify each possible protection separately: copyright, software, database right, invention, utility model, design, trademark, trade secret or unfair competition.
3. For each applicable regime, determine creation or registration, ownership, exercise of rights, duration, territory, limitations and enforcement.
4. Resolve the effective Czech and EU rules at the relevant date. Check international instruments only where territory or priority makes them relevant.
5. Verify registry status, priority, classes, proprietor and procedural state for registered rights.
6. Read every licence or assignment against mandatory rules, scope, exclusivity, territory, duration, sublicensing, modifications and termination.
7. Analyze overlaps without assuming that one form of protection excludes another.

For copyright, test originality and authorship. For inventions, test novelty, inventive step, industrial applicability and exclusions. Where several people or employers are involved, map co-authorship, co-inventorship and every employment relationship separately.

## Employee-created results

| Result | Starting regime | Required checks |
|---|---|---|
| Copyright work or software | Copyright Act | Work duties, author, employer's exercise of economic rights, personal rights and special software rules |
| Invention | Inventions and Rationalisation Proposals Act | Technical solution, work task, notification, employer's exercise, statutory period and remuneration |
| Design | Design legislation | Creation in employment, entitlement to registration and contractual allocation |
| Trade secret or know-how | Civil Code, labour duties and contract | Secrecy, value, protective measures, access and permitted disclosure |

Never merge an employee work with an employee invention. Determine whether the creator is an employee, contractor, officer, student or another person. For inventions, verify the form, required content, delivery and proof of notification; the employer's timely exercise; consequences of inaction; and the basis, maturity, limitation period and contractual limits of fair or additional remuneration. For software, test the required technical effect and audit applicable open-source licences alongside copyright, patent, database and trade-secret layers.

For know-how, distinguish information that meets every trade-secret condition from knowledge protected only by contract or labour duties. Check permitted whistleblowing, employee mobility and post-employment use. For cross-border creation, separate governing employment law, entitlement to file, each national or regional application and the territorial acquisition of rights.

## Infringement and remedies

- Identify the protected subject matter and compare legally relevant features, not just overall similarity.
- Check consent, licence scope, exhaustion, statutory exceptions, prior rights and territorial reach.
- Separate injunction, removal, information, damages, unjust enrichment, publication and interim relief.
- Preserve evidence and identify the party entitled to sue before quantifying a claim.
- For online uses, distinguish the direct user, intermediary and platform and assess each legal basis separately.

## Common mistakes

- Calling every creative or technical output a copyright work.
- Treating registration as necessary for copyright or irrelevant for patents and trademarks.
- Assuming the employer owns every employee-created result under one uniform rule.
- Omitting notification periods, priority, renewal, additional remuneration or territorial scope.
- Treating a contractual permission as broader than its proven licence terms.
- Ignoring cumulative protection, open-source conditions or trade-secret loss through disclosure.

## Output

Lead with the classification and rights holder for each regime. Then show creation or registration, chain of title, permitted use, limitations, deadlines, evidence and remedies. Link CODEXIS sources and official registry records with user-accessible `https://` URLs; never expose raw internal identifiers.
