---
uuid: 6caec9ec-a2ec-4e10-bb21-acf94cc7df54
name: codexis-rekodifikace
icon: icon.svg
version: 1.0.0
jurisdictions: [CZ]
description: Use when a Czech private-law question turns on recodification, transitional provisions, old versus new Civil Code, legal relationships created before a reform, continuing effects, later legal facts, limitation periods, transformed legal persons or retroactivity. Standalone temporal-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "CODEXIS - Rekodifikace"
    summary: "Přechodná ustanovení a časové účinky rekodifikace českého soukromého práva."
    examplePrompts:
      - "Které právo se použije na smlouvu uzavřenou před rekodifikací a její pozdější porušení?"
      - "Posuď promlčení nároku vzniklého před změnou občanského zákoníku."
      - "Jak se rekodifikace dotkla právní formy a jednání starší právnické osoby?"
  en:
    displayName: "CODEXIS - Czech Recodification"
    summary: "Transitional provisions and temporal effects of Czech private-law recodification."
    examplePrompts:
      - "Which law governs a contract made before recodification and a later breach?"
      - "Assess limitation of a claim created before the Civil Code changed."
      - "How did recodification affect an older legal person's form and representation?"
  sk:
    displayName: "CODEXIS - Rekodifikácia ČR"
    summary: "Prechodné ustanovenia a časové účinky rekodifikácie českého súkromného práva."
    examplePrompts:
      - "Ktoré právo sa použije na zmluvu uzavretú pred rekodifikáciou a jej neskoršie porušenie?"
      - "Posúď premlčanie nároku vzniknutého pred zmenou občianskeho zákonníka."
      - "Ako rekodifikácia ovplyvnila právnu formu a konanie staršej právnickej osoby?"
---

# CODEXIS - Rekodifikace

Use `cdx-cli` to retrieve every relevant historic version, amendment and transitional provision.

## Mandatory temporal analysis

1. Build a dated timeline of formation, acquired rights, continuing effects, breach, exercise of rights, proceedings and decision.
2. Classify each event separately. Do not assign one governing law to the whole relationship without testing later facts.
3. Identify the old rule, new rule, general transitional provision and every subject-specific transitional rule.
4. Apply the specific transitional rule before the general one. Trace every cross-reference to the act it names.
5. Separate substantive rights, legal effects, limitation, procedure, evidence and registry consequences.
6. Test vested rights, legitimate expectations, true and apparent retroactivity and constitutional limits where relevant.
7. Verify current case law interpreting the exact transitional provision and state any unresolved conflict.

## Timeline matrix

| Layer | Question |
|---|---|
| Formation | Which law governed creation and validity? |
| Continuing relationship | Does the old regime continue or does the new regime govern future effects? |
| Later legal fact | Which law governs a later breach, notice, death, transfer or modification? |
| Limitation | When did the period start, which transition applies and did a long-stop rule intervene? |
| Procedure | Which procedural law applies when proceedings begin or continue? |
| Registry | Was transformation, adaptation or registration required and by when? |

## Required checks

- For Civil Code recodification, read the relevant part of sections 3028 through 3079 and any special act instead of citing the range generally.
- For property, companies, leases, consumers, family matters or legal persons, search for a special transitional provision before using a general formula.
- Distinguish the creation of a right from its exercise and from a remedy for later breach.
- Calculate every period from verified statutory language and dated facts. Preserve qualifiers such as earliest, latest and continuously.
- Do not infer impermissible retroactivity merely because a new rule affects future consequences of an existing relationship.

## Common mistakes

- Applying only the date of contract formation to every later event.
- Quoting a general transitional provision while missing a special one.
- Mixing historical and current versions in one conclusion.
- Treating substantive and procedural temporal rules as identical.
- Labeling every retrospective effect unconstitutional without the proper test.
- Omitting limitation, registry adaptation or protection of third parties.

## Output

Lead with a one-sentence choice-of-law conclusion for each event. Then show the timeline, old and new rules, specific and general transition, constitutional check, calculations and uncertainties. Use resolved `https://` citations and never raw CODEXIS identifiers.
