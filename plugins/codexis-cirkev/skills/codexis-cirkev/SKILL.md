---
uuid: 826a0cd8-d9e5-4607-9ba1-62b733df865d
name: codexis-cirkev
icon: icon.svg
version: 1.0.0
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

# Monitor - Církev

Use `cdx-cli` as the primary legal source. Add official registry data from the Ministry of Culture or another competent authority when identity, registration or representation must be verified.

## Mandatory analysis

1. Identify the exact entity, its registration category, identifier, governing church and the relevant date.
2. Separate five questions: legal personality, ownership, authority to act, power of attorney and any required internal approval.
3. Resolve the effective statute version and all transitional rules. Read the operative provision with constitutional and human-rights protections.
4. Determine whether an internal religious rule has civil effect, evidentiary relevance or only internal consequences. Never assume one from the other.
5. Verify the official register, title register or other authoritative record needed for the specific conclusion.
6. Search case law when autonomy, clergy status, employment, property, restitution, discrimination or judicial review is disputed.
7. State which conclusion follows from public law, private law and internal church law separately.

## Source stack

| Layer | Use |
|---|---|
| Constitutional order and human-rights law | Freedom of religion, autonomy, equality and limits of state interference |
| Act on churches and religious societies | Registration, recorded legal persons, special rights and public registers |
| Civil Code and registry law | Legal personality, representation, property and transactions |
| Labour and anti-discrimination law | Employment and service relationships where applicable |
| Restitution and financing legislation | Historic property claims, settlements and state funding |
| Internal church rules | Internal competence and approval, only with the civil effect proven |

## Entity and property verification

- Match the legal name, identifier, registered seat and legal form across current and historic records.
- If the public register may be stale, verify the office-holder through the underlying registry file, appointment instrument or other authoritative document.
- Do not treat entry in the church register as proof of ownership.
- Do not treat a land-register entry as proof that the signatory may act for the owner.
- Verify the acquisition title, current ownership, co-ownership, encumbrances, pending registry proceedings and any historic succession issue separately. Trace historic succession through registry files, filed instruments, archived land-register records, transformation acts and restitution decisions as relevant.
- Verify the registered manner of representation, the actual office-holder, the grantor's authority, and the form, scope, substitution, revocation and expiry of any power of attorney.
- For internal approval, verify the hierarchy and relevant version of internal rules, the approving body's competence and the form and date of its decision.
- Explain whether absence of internal approval affects external validity, internal responsibility or both; do not speculate.
- Test whether the internal restriction is incorporated by statute, publicized toward third parties, compatible with third-party protection and interpreted in relevant case law.

## Autonomy and civil review

- Identify the protected sphere of religious autonomy and the civil right or obligation asserted against it.
- Distinguish doctrinal questions from employment, property, personality and procedural rights that civil authorities can assess.
- When clergy are involved, classify the relationship from its legal features rather than its religious title.
- Check Czech constitutional case law and relevant European human-rights case law before stating the reach of judicial review.

## Common mistakes

- Conflating a church, religious society, parish, congregation, diocese and a separately recorded legal person.
- Assuming canonical or internal authority automatically equals civil authority to sign.
- Using one current register extract to resolve historic identity or succession.
- Treating religious autonomy as either absolute immunity or legally irrelevant.
- Omitting constitutional and European human-rights law.
- Giving a property conclusion without verifying both ownership and representation.

## Output

Lead with a conditional conclusion naming the entity and act assessed. Then show the identity chain, ownership evidence, representation evidence, internal approvals, governing legal layers and remaining uncertainties. Use resolved `https://` links from CODEXIS and official registers; never expose raw `cdx://` identifiers.
