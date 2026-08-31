---
uuid: 5c0f6fe9-3b90-421f-ad5d-28935721f6a4
name: codexis-zivotni-prostredi
icon: icon.svg
version: 1.0.0
jurisdictions: [CZ, EU]
description: Use when a Czech or EU legal question concerns environmental impact assessment, EIA, JES, nature protection, water, air, waste, soil, forests, integrated permitting, environmental damage, public participation, environmental liability or environmental aspects of project authorization. Standalone environmental-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "CODEXIS - Životní prostředí"
    summary: "České a evropské právo životního prostředí, EIA, JES a složková povolení."
    examplePrompts:
      - "Jak se EIA promítá do navazujícího povolení a účasti veřejnosti?"
      - "Urči, které složky ochrany životního prostředí pokryje JES pro tento záměr."
      - "Posuď, zda změna projektu vyžaduje nové zjišťovací řízení nebo ověření stanoviska EIA."
  en:
    displayName: "CODEXIS - Environmental Law"
    summary: "Czech and EU environmental law, EIA, JES and sector permits."
    examplePrompts:
      - "How does EIA affect follow-up authorization and public participation?"
      - "Determine which environmental components JES covers for this project."
      - "Assess whether a project change requires new screening or verification of the EIA opinion."
  sk:
    displayName: "CODEXIS - Životné prostredie ČR"
    summary: "České a európske právo životného prostredia, EIA, JES a zložkové povolenia."
    examplePrompts:
      - "Ako sa EIA premieta do nadväzujúceho povolenia a účasti verejnosti?"
      - "Urči, ktoré zložky ochrany životného prostredia pokryje JES pre tento zámer."
      - "Posúď, či zmena projektu vyžaduje nové zisťovacie konanie alebo overenie stanoviska EIA."
---

# CODEXIS - Životní prostředí

Use `cdx-cli` for Czech and EU law and case law. Add only official ministry, authority and EU material for operational or interpretive context.

## Mandatory analysis

1. Define the project, activity, operator, location, protected receptors, capacity, timeline, changes and user's role.
2. Build a permit matrix across EIA, JES and each environmental component. Do not assume one act replaces every other permit or duty.
3. Resolve Czech and EU rules effective for each procedural step, including transitional provisions.
4. Identify competent authorities, applicants, affected authorities, participants and public-participation rights separately.
5. Trace every condition from screening, EIA opinion or sector act into the follow-up permit and compliance stage.
6. Test project changes, cumulative impacts, alternatives, validity, extensions and repeated assessment before relying on an earlier conclusion.
7. Search Czech and EU case law for participation, assessment scope, binding effect, project splitting and effective judicial protection.

## EIA and follow-up authorization

- Determine whether the project is subject to mandatory assessment, screening or neither, using verified category and capacity facts.
- Identify the assessed project and compare it with the project now authorized. Record every material change.
- Verify the EIA opinion's validity, extension, verification and conditions.
- Identify each follow-up proceeding and how the EIA information, opinion and public participation enter it.
- Do not treat EIA as a permit or as a one-time step immune from later project changes.
- Check cross-border assessment, Natura 2000 and cumulative effects independently where plausible.

## JES and sector rules

- List each environmental act potentially integrated into JES and each act, exception or operational duty that remains separate.
- Match the JES regime and transitional rule to the underlying procedure and application date.
- Separate nature, water, air, waste, soil, forest and integrated-permit tests even when one document combines the result.
- For each component, identify trigger, authority, evidence, conditions, duration, reporting and review.

## Compliance and liability

- Distinguish permit breach, administrative offence, environmental damage, civil liability and criminal exposure.
- Identify monitoring, records, reporting, emergency action, remediation and authority-notification duties.
- For waste or excavated material, classify the material and intended handling from facts before applying waste, by-product or reuse rules.

## Common mistakes

- Treating EIA, JES and the final project permit as interchangeable.
- Ignoring a project change or cumulative impact after an earlier EIA.
- Missing participation rights in a follow-up proceeding.
- Applying only Czech law where an EU directive and CJEU case law shape the rule.
- Assuming JES absorbs every environmental permission or ongoing duty.
- Mixing legal conditions from different effective dates.

## Output

Lead with the required assessment and permit path. Then show the project definition, dated procedure map, EIA and JES status, sector layers, actors, public participation, conditions, compliance duties, remedies and open facts. Cite resolved user-facing `https://` sources and never raw CODEXIS identifiers.
