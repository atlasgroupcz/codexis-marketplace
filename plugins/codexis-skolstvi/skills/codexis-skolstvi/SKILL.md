---
uuid: b2400c7f-edab-4722-8222-dbf6722850b0
name: codexis-skolstvi
icon: icon.svg
version: 1.0.0
jurisdictions: [CZ, EU]
description: Use when a Czech legal question concerns schools, education, admissions, pupils, students, assessment, school directors, founders, school boards, special educational needs, educational records, administrative decisions or employment links in education. Standalone education-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "CODEXIS - Školství"
    summary: "České školské právo, rozhodování škol, práva žáků a role zřizovatelů."
    examplePrompts:
      - "Musí přijetí žáka do ZUŠ proběhnout ve správním řízení?"
      - "Jaký proces platí při odvolání ředitele školy z funkce?"
      - "Prověř pravidla hodnocení, přezkoumání výsledku a opravné zkoušky."
  en:
    displayName: "CODEXIS - Czech Education Law"
    summary: "Czech education law, school decisions, student rights and founders' roles."
    examplePrompts:
      - "Must admission to a Czech art school follow administrative procedure?"
      - "What process applies when removing a school director from office?"
      - "Review the rules for assessment, review of results and resit examinations."
  sk:
    displayName: "CODEXIS - Školstvo ČR"
    summary: "České školské právo, rozhodovanie škôl, práva žiakov a úlohy zriaďovateľov."
    examplePrompts:
      - "Musí prijatie žiaka do českej ZUŠ prebehnúť v správnom konaní?"
      - "Aký proces platí pri odvolaní riaditeľa školy z funkcie?"
      - "Prever pravidlá hodnotenia, preskúmania výsledku a opravnej skúšky."
---

# CODEXIS - Školství

Use `cdx-cli` for legal sources. Use official MŠMT or inspection material only after verifying its date and legal basis.

## Mandatory analysis

1. Identify the school type, legal form, founder, education level, person affected, user's role and decisive dates.
2. Resolve ambiguous terminology before research, especially appeal versus removal from office and school decision versus employment act.
3. Build the source stack: Education Act, school-specific decree, Administrative Procedure Code, founder rules, employment law and special regulation.
4. Resolve the effective version and transitional provisions of every layer.
5. Determine the actor, public or private function, form, required findings, notice or publication, appeal and judicial protection.
6. Read connected provisions together. Never infer the absence of administrative procedure solely because one item is not explicit in one list.
7. Separate the education-law effect from enrolment records, funding and employment consequences.

## Admission and school decisions

- Identify the specific admission rule for the school type, selection or talent-test conditions and the director's decision-making authority.
- For ZUŠ, read Decree No. 71/2005 Coll. together with sections 165 and 183 of the Education Act and the Administrative Procedure Code.
- Determine whether a formal administrative decision is issued, how a favourable and unfavourable result is announced, when it is served and which appeal is available.
- Distinguish publication under registration numbers from an informal publication of results.
- Preserve the application, criteria, commission record, factual findings, file record and proof of notice.

## School directors

- Distinguish appointment, removal from office, expiry of term, waiver, employment change and termination of employment.
- Start the director-removal analysis with section 166 of the Education Act and verify its effective version.
- Identify school type, founder, competent body, statutory ground, required supporting facts, consultation or proposal and relevant period.
- Do not equate removal from office with automatic end of employment.
- Determine the required form, reasoning, delivery, effective date and judicial remedy.
- Verify the legal role of inspection findings, school-board acts and founder documents instead of treating each as automatically decisive.
- Do not assume the founder and the appellate administrative authority are the same actor; establish each competence independently.

## Assessment and support

- Link grading, non-assessment, review, resit and remedial measures to the exact school and study type.
- Distinguish statutory duties, decree requirements, school rules and recommended pedagogy.
- For special educational needs, identify the support level, recommendation, consent, provider and funding track.
- Verify deadlines and competence for review or examination from current sources.

## Common mistakes

- Applying rules for one school type to another.
- Reading section 165 in isolation and missing a special admission or notification rule.
- Confusing removal of a director with an administrative appeal or employment termination.
- Treating ministry methodology as law or using a current method for a historic case.
- Omitting service, publication, appeal and file documentation.
- Presenting optional good practice as a mandatory school duty.

## Output

Lead with the school type, competent actor and procedural classification. Then show the effective source stack, conditions, evidence, form, notice, deadlines, remedies, employment effects and open facts. Use resolved `https://` citations and never raw CODEXIS identifiers.
