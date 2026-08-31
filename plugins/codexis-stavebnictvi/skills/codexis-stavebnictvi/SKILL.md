---
uuid: 18b20f5f-1ec5-40e2-959b-591a9a48e1ae
name: codexis-stavebnictvi
icon: icon.svg
version: 1.0.0
jurisdictions: [CZ, EU]
description: Use when a Czech legal question concerns construction, building permits, project authorization, occupancy approval, additional permitting, zoning, building authorities, affected authorities, JES, project documentation, unauthorized structures, changes before completion or construction-law transitional provisions. Standalone construction-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "CODEXIS - Stavebnictví"
    summary: "České stavební právo, povolování, kolaudace, dokumentace a dotčené orgány."
    examplePrompts:
      - "Jak se kolauduje stavba povolená podle starého stavebního zákona?"
      - "Posuď, jaké podklady a stanoviska jsou nutné pro dodatečné povolení stavby."
      - "Který orgán má v tomto řízení požádat o JES a jak se postup eviduje?"
  en:
    displayName: "CODEXIS - Czech Construction Law"
    summary: "Czech construction law, permitting, occupancy, documentation and affected authorities."
    examplePrompts:
      - "How is a building authorized under the former Building Act approved for use?"
      - "Assess the documents and opinions required for additional authorization of a structure."
      - "Which authority must request JES in this proceeding and how is the process recorded?"
  sk:
    displayName: "CODEXIS - Stavebníctvo ČR"
    summary: "České stavebné právo, povoľovanie, kolaudácia, dokumentácia a dotknuté orgány."
    examplePrompts:
      - "Ako sa kolauduje stavba povolená podľa starého stavebného zákona?"
      - "Posúď podklady a stanoviská potrebné pre dodatočné povolenie stavby."
      - "Ktorý orgán má v tomto konaní požiadať o JES a ako sa postup eviduje?"
---

# CODEXIS - Stavebnictví

Use `cdx-cli` for legislation, case law and commentary. Use official ministry methodology only after matching it to the exact legal version and procedure.

## Mandatory analysis

1. Identify the user's role, competent authority, project type, location, procedural stage and every decisive date.
2. Build separate timelines for planning, project authorization, environmental acts, construction, change, completion, occupancy and enforcement.
3. Resolve which Building Act and which transitional provision governs each step. Do not choose the regime only from the application or answer date.
4. Classify the procedure precisely: new authorization, change before completion, occupancy, additional authorization, removal, remedy or review.
5. Map every required document, owner consent, binding opinion, JES component, network statement and public-interest assessment to the actor responsible for obtaining or issuing it.
6. Separate the building authority's procedure from affected-authority and environmental procedures even when systems or documents overlap.
7. Verify official methodology against the cited provision, facts and date; quote its actual condition before relying on it.

## Transitional regime

- Record dates of the original application, commencement, permit, legal force, changes, completion and new request.
- Read the complete transitional provision and any special transition for JES or replaced environmental acts.
- Determine the regime separately for the underlying building proceeding and for each later or parallel act.
- Do not assume that a proceeding under the former Building Act automatically keeps every environmental act under the old regime.
- Do not assume the new regime applies merely because a later document was filed after the reform.

## Procedure and roles

- Confirm whether the user acts for the applicant, building authority, JES authority, another affected authority, owner or participant before drafting advice.
- Name who requests, supplements, issues, serves, records and reviews each document.
- Distinguish a new project from additional authorization of an existing unauthorized structure before allocating a request for opinions or JES.
- For electronic files, distinguish ISSŘ, the authority's records service, data-box delivery and the legally authoritative case file. Do not infer a transfer duty without a source.
- When reviewing a submission, assess its content, signatory, representation, destination, file reference and procedural effect separately.

## Materials and site issues

- Distinguish reuse in the place of excavation, temporary storage within the construction site, movement to another parcel, by-product and waste.
- Identify whether another owner's parcel is legally part of the construction site and document consent, duration, restoration and safety.
- Separate soil-protection, waste, water, nature-protection and construction requirements.

## Common mistakes

- Applying only one legal regime to every phase of a project.
- Missing a special environmental transitional rule.
- Answering from the applicant's perspective when the user is the authority.
- Treating ministry guidance as a substitute for the statute or misreading its factual condition.
- Confusing additional authorization with a new project.
- Concluding that temporary storage automatically creates waste or a by-product.
- Mixing receipt in a records service with storage in ISSŘ.

## Output

Lead with the governing regime and user's procedural role. Then show the dated timeline, procedure, competent actors, required documents and opinions, electronic-file path, deadlines, defects and next action. Cite resolved `https://` sources and never raw CODEXIS identifiers.
