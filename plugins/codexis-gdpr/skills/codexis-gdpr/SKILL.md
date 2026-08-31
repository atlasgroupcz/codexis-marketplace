---
uuid: acec0787-0c11-43f7-b43d-95d046f73c07
name: codexis-gdpr
icon: icon.svg
version: 1.0.0
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

# Monitor - Ochrana osobních údajů

Use `cdx-cli` for Czech and EU law. Use only official EDPB, EU, ÚOOÚ and other competent-authority material as complementary interpretation.

## Mandatory analysis

1. Map the concrete processing: data, people, source, purpose, operations, recipients, systems, locations, retention and consequences.
2. Assign controller, joint-controller and processor roles from actual decisions and functions, not contract labels.
3. Test scope, principles and a legal basis for every purpose. Test special-category and criminal-offence data separately.
4. Determine whether another legal regime independently restricts the processing, including labour, communications, health or surveillance law.
5. Test transparency, rights, security, records, DPIA, processor terms, transfers and breach duties as separate layers.
6. Resolve current legislation and relevant CJEU, Czech court and authority decisions. Explain the legal weight of guidelines.
7. Conclude only for the processing actually mapped; list missing facts that could change the result.

## Employee monitoring

- Identify the technology, exact purpose, monitored persons, place, time, content captured and degree of continuous observation.
- Test the Labour Code restriction independently from the GDPR legal basis. Passing one test does not pass the other.
- Do not default to employee consent where the employment imbalance prevents a freely given choice.
- If consent is proposed exceptionally, verify a genuine choice, no adverse consequence, granularity, proof and effective withdrawal.
- For legitimate interests, document purpose, necessity and balancing. Compare less intrusive alternatives and residual safeguards.
- Distinguish GDPR notice content and timing from the separate Labour Code information duty.
- Limit access by role, keep audit logs and review permissions regularly.
- Set retention per purpose with a deletion trigger and a documented rule for records preserved as evidence.
- Apply the DPIA threshold test, required content and prior-consultation test.
- For disciplinary or performance use, distinguish the original monitoring purpose from secondary use, retest legal basis and purpose compatibility, and preserve a way for the employee to contest the evidence.
- Mere notice does not legalize monitoring that lacks necessity or a permissible purpose.

## Breaches and rights

- Distinguish a security incident from a personal-data breach and assess confidentiality, integrity and availability.
- Record the risk analysis even when notification is not required. Check authority and data-subject notification separately.
- For access, erasure, objection or portability, identify the requester, scope, exemptions, third-party rights, identity verification and response deadline.
- For profiling or automated decisions, test whether there is a solely automated decision with legal or similarly significant effect and which safeguard applies.

## International transfers

- Identify every exporter, importer, onward recipient, country and remote-access path.
- Separate disclosure, processor access and Chapter V transfer analysis.
- Verify adequacy, safeguards, transfer-impact assessment, supplementary measures, derogations and onward transfers.

## Common mistakes

- Treating consent as the universal legal basis.
- Treating a privacy notice as proof that processing is lawful.
- Selecting one legal basis for several incompatible purposes.
- Omitting the independent labour-law test for employee monitoring.
- Saying data are anonymized when re-identification remains reasonably possible.
- Treating pseudonymized data as outside GDPR.
- Quoting an authority guideline as binding legislation.

## Output

Lead with the conclusion per processing purpose. Then provide the mapped roles, data and flow; legal basis and special conditions; necessity and proportionality; transparency and rights; security, DPIA and retention; transfers; required actions and unresolved facts. Use user-accessible `https://` citations and never expose raw CODEXIS identifiers.
