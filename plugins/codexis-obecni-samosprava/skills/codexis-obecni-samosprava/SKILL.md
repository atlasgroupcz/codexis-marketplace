---
uuid: a892e30a-aa51-4e58-8b13-979d9bd897c1
name: codexis-obecni-samosprava
icon: icon.svg
version: 1.0.0
jurisdictions: [CZ]
description: Use when a Czech legal question concerns municipalities, municipal councils, boards, mayors, municipal offices, independent or delegated powers, municipal property, local ordinances, elections, access to information, contributory organizations or municipal administrative procedure. Standalone municipal-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "CODEXIS - Obecní samospráva"
    summary: "Právo obcí, jejich orgánů, majetku a výkonu samostatné i přenesené působnosti."
    examplePrompts:
      - "Který orgán obce musí schválit převod obecního pozemku a jaké kroky tomu předcházejí?"
      - "Prověř platnost usnesení rady obce a navazující smlouvy."
      - "Jaké povinnosti má obec a registrační úřad před komunálními volbami?"
  en:
    displayName: "CODEXIS - Czech Municipal Law"
    summary: "Czech municipal bodies, property and independent or delegated government powers."
    examplePrompts:
      - "Which municipal body must approve the transfer of municipal land and what steps come first?"
      - "Review the validity of a municipal board resolution and the resulting contract."
      - "What duties apply to a municipality and registration office before local elections?"
  sk:
    displayName: "CODEXIS - Obecná samospráva ČR"
    summary: "České právo obcí, ich orgánov, majetku a samostatnej aj prenesenej pôsobnosti."
    examplePrompts:
      - "Ktorý orgán obce musí schváliť prevod obecného pozemku a aké kroky tomu predchádzajú?"
      - "Prever platnosť uznesenia rady obce a nadväzujúcej zmluvy."
      - "Aké povinnosti má obec a registračný úrad pred komunálnymi voľbami?"
---

# CODEXIS - Obecní samospráva

Use `cdx-cli` for legal sources and official public-administration sources for current forms or operational instructions.

## Mandatory analysis

1. Fix the relevant date or clarify whether the request concerns one date or every version across an interval. Identify the exact municipality, body, official and role from which the user is acting.
2. Classify the matter as independent powers, delegated state administration or mixed activity before selecting the legal regime.
3. Build a competence chain: reserved body, possible delegation, preparatory body, signatory and reviewing authority.
4. Resolve the statute version valid at the requested date through `/versions`; never answer a historical question from the current document version.
5. Read the operative provision with definitions, cross-references, publication duties, quorum and voting rules, procedural rules and transitional provisions.
6. Check consequences separately for the public-law act, internal municipal decision and any resulting private-law transaction.
7. Search case law when invalidity, competence, transparency, information access, elections or review is disputed.

## Core distinctions

| Question | Verify |
|---|---|
| Competence | Zastupitelstvo, rada, starosta, obecní úřad, zvláštní orgán or delegated official |
| Capacity | Municipality as legal person versus its body or office |
| Function | Decision, preparation, execution, signature, publication and supervision |
| Regime | Independent powers versus delegated state administration |
| Time | Version effective for the event, not the date of the answer |

## Municipal property

- Identify the asset, legal act, value only when legally relevant and whether a reserved competence applies.
- Verify prior publication of intent, required duration, exact subject identification and whether an exception applies.
- Verify the resolution's body, quorum, wording, price or other essential terms and relationship to the signed contract.
- Verify the authority and manner of signature separately from approval.
- Verify the required land-register filing and any pending registry proceeding separately from the municipality's approval.
- Explain separately the consequence of a defective intent, defective resolution, unauthorized signature and conflict with due-care duties.
- Never invent a monetary threshold below which the board or mayor may dispose of real property.

## Elections and public procedure

- Identify the exact actor named by the statute. Do not transfer a municipality's publication duty to a registration office or the reverse, even when both roles are performed within the same municipal office.
- Distinguish deadline, method of publication, service, registration and legal effect.
- For forms and election administration, match official implementation guidance to the same historic or current period as the legal rule before providing a template.
- When a term is ambiguous, such as an appeal versus removal from office, resolve the intended act before researching.

## Common mistakes

- Treating the municipality, council, board, mayor and municipal office as interchangeable.
- Using current law for an expressly historical date.
- Finding the approval rule but omitting publication, voting, signature or review.
- Assuming an internal resolution alone transfers property or creates a contract.
- Giving advice to the applicant when the user is acting as the authority.
- Treating an optional governance recommendation as a statutory duty.

## Output

Lead with the competent body and short conclusion. Then show the relevant date and version, competence chain, mandatory steps, document requirements, consequences of defects and practical next action for the user's actual role. Use resolved `https://` links; never expose raw CODEXIS identifiers.
