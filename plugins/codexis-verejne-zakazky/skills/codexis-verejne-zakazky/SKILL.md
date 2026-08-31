---
uuid: 6a3301ef-581b-45ce-af55-50234bffa5f8
name: codexis-verejne-zakazky
icon: icon.svg
version: 1.0.0
jurisdictions: [CZ, EU]
description: Use when a Czech or EU legal question concerns public procurement, contracting authorities, estimated value, procurement procedures, tender conditions, qualification, bids, clarification, evaluation, deadlines, site visits, objections, review, contract award or modification under ZZVZ. Standalone procurement methodology using CODEXIS.
i18n:
  cs:
    displayName: "CODEXIS - Veřejné zakázky"
    summary: "České a evropské právo veřejných zakázek, zadávací postupy a změny smluv."
    examplePrompts:
      - "Lze po lhůtě doplnit kvalifikaci nebo novou referenční zakázku?"
      - "Prověř lhůtu pro podání nabídek a termín prohlídky místa plnění."
      - "Posuď, zda lze po zadání změnit cenu nebo rozsah smlouvy."
  en:
    displayName: "CODEXIS - Public Procurement"
    summary: "Czech and EU public procurement law, tender procedures and contract changes."
    examplePrompts:
      - "Can qualification or a new reference contract be supplemented after the bid deadline?"
      - "Review the bid deadline and scheduled site visit."
      - "Assess whether the price or scope of an awarded contract may be changed."
  sk:
    displayName: "CODEXIS - Verejné obstarávanie ČR"
    summary: "České a európske právo verejného obstarávania, postupy a zmeny zmlúv."
    examplePrompts:
      - "Možno po lehote doplniť kvalifikáciu alebo novú referenčnú zákazku?"
      - "Prever lehotu na predkladanie ponúk a termín obhliadky miesta plnenia."
      - "Posúď, či možno po zadaní zmeniť cenu alebo rozsah zmluvy."
---

# CODEXIS - Veřejné zakázky

Use `cdx-cli` for legislation, Czech and EU case law and commentary. Use official ÚOHS, TED and procurement-portal material only as complementary sources.

## Mandatory analysis

1. Identify the contracting entity, subject, estimated value, aggregation, procurement regime, procedure, current phase and decisive dates.
2. Separate a statutory duty, permitted option, reserved choice, best-practice recommendation and contractual requirement. Label each conclusion.
3. Resolve the applicable ZZVZ and EU-law version. Check transitional rules for the procedure and contract.
4. Build the complete rule stack for the issue: principles, general provision, phase-specific provision, exception, publication and remedy.
5. Read cited case law for its facts, procedural setting and holding. Record the research date, verify procedural status or finality, and check whether later authority overruled, limited or distinguished it. Do not generalize a rule from a different phase or procurement regime.
6. Check equal treatment, transparency, proportionality, non-discrimination and material comparability after the specific statutory test.
7. State what the authority must do, may do and should document separately.

## Qualification and clarification

- Distinguish qualification facts from bid content and evaluation criteria.
- Read sections 46 and 79 together with the exact tender condition and relevant case law.
- Determine whether a fact may arise after the bid deadline, whether the authority may request clarification and whether the change alters evaluated bid content.
- Do not conclude that a new reference or subcontractor is always prohibited or always allowed. Match the holding to the exact change.
- Verify whether the authority reserved a stricter lawful requirement and whether applying it remains transparent and equal.

## Deadlines and site visits

- Calculate the statutory minimum for the exact procedure and every reduction or extension condition.
- When a site visit is offered, always test the special rule in section 97, including the interval before the bid deadline and the requirement that the bid period exceed the minimum.
- For corrected notices, distinguish publication in VVZ or TED, the profile update and the legally effective new deadline.
- Verify that the effective deadline is synchronized across every applicable notice, profile field, procurement document and attachment.
- Use exact calendar dates, working days and public holidays. Do not rely on approximate weeks.

## Contract modifications

- Identify the original contract, reservation clause, nature and cumulative value of every prior and proposed change.
- Test the general material-change rule and each statutory exception separately.
- Do not say that a percentage threshold alone makes a change lawful. Check subject, cumulative calculation, overall cap, nature and whether the change alters competition.
- Separate price indexation already reserved in the tender from a later unreserved modification.
- Check publication, written reasoning and review risk after the substantive test.

## Document review

- Review internal consistency before proposing extra forms or duplicate information.
- Do not present a defensible drafting preference as a legal defect.
- Tie every required edit to a statute, tender choice or concrete inconsistency.
- Preserve the user's requested structure unless changing it is necessary and explain that necessity.

## Common mistakes

- Mixing value regime, procedure choice and deadline rules.
- Missing a phase-specific provision such as section 97.
- Treating all post-deadline qualification changes alike.
- Calling an optional improvement legally mandatory.
- Applying a percentage exception for contract changes without cumulative and qualitative tests.
- Reviewing one attachment in isolation from the rest of the tender documents.

## Output

Lead with a direct answer labeled mandatory, permitted or recommended. Then show regime and phase, effective sources, complete statutory test, facts, calculation, case-law fit, required documentation and risk. Cite resolved `https://` URLs and never raw CODEXIS identifiers.
