---
uuid: 5a416920-3f3e-4075-83cf-5db20ae55800
name: codexis-personalistika
icon: icon.svg
version: 1.0.0
jurisdictions: [CZ, EU]
description: Use when a Czech legal question concerns employment, HR, hiring, contracts, termination, redundancy, working time, leave, wages, payroll, health or social insurance, discrimination, employee representation, remote work, obstacles to work or personnel records. Standalone employment-law methodology using CODEXIS.
i18n:
  cs:
    displayName: "CODEXIS - Personalistika"
    summary: "České pracovní právo, mzdy, odvody a personální agenda."
    examplePrompts:
      - "Prověř výpověď pro nadbytečnost a rizika při výběru konkrétního zaměstnance."
      - "Musí invalidní zaměstnanec při neplaceném volnu doplatit zdravotní pojištění do minima?"
      - "Navrhni správný postup při změně pracovní doby a místa výkonu práce."
  en:
    displayName: "CODEXIS - Czech HR"
    summary: "Czech employment law, payroll, insurance and HR administration."
    examplePrompts:
      - "Review a redundancy dismissal and the risks in selecting a particular employee."
      - "Must an employee with a disability top up health insurance during unpaid leave?"
      - "Propose a compliant process for changing working time and place of work."
  sk:
    displayName: "CODEXIS - Personalistika ČR"
    summary: "České pracovné právo, mzdy, odvody a personálna agenda."
    examplePrompts:
      - "Prever výpoveď pre nadbytočnosť a riziká pri výbere konkrétneho zamestnanca."
      - "Musí invalidný zamestnanec pri neplatenom voľne doplatiť zdravotné poistenie do minima?"
      - "Navrhni správny postup pri zmene pracovného času a miesta výkonu práce."
---

# CODEXIS - Personalistika

Use `cdx-cli` for legislation, case law and commentary. Use official labour, social-security and health-insurance sources only for complementary operational information.

## Mandatory analysis

1. Identify the parties, actual work arrangement, legal relationship, relevant dates and the user's role.
2. Separate employment-law, wage, income-tax, social-insurance and health-insurance tracks. Do not infer one from another.
3. Resolve the effective version of each statute and implementing rule. Verify every monetary amount, interval and threshold from the current source.
4. Map formation, required form, substantive conditions, process, delivery, evidence, deadlines and consequences of invalidity.
5. Check collective agreements, internal rules, employee-representative duties and protected status where relevant.
6. Search case law for termination, redundancy, discrimination, working-time classification, wage claims and evidentiary disputes.
7. Provide the employer and employee actions separately, including documents and dates.

## Termination and redundancy

- Identify the exact statutory ground and factual event before drafting language.
- For redundancy, verify a real prior organizational decision, competent decision-maker, timing and causal connection to the employee's redundancy.
- Separate the employer's business choice from unlawful selection criteria. Check discrimination, retaliation, abuse of rights, protected periods, union-office protection and collective-dismissal rules.
- Define the comparable employee group, document neutral selection criteria and verify their consistent application.
- Verify that the abolished work does not continue under another title, with a replacement or divided among others in a way that contradicts the asserted organizational change.
- Verify specificity, signature, delivery, notice-period calculation, severance and the preclusive period for challenging invalid termination.
- Do not state that a court can never review employee selection; explain the actual review boundaries.

## Payroll and insurance

- Determine the person's exact insurance status for every relevant day, including disability degree, state-insured status, concurrent work and exceptions.
- Distinguish minimum wage, guaranteed pay where applicable, minimum assessment base and actual assessment base.
- For unpaid leave or low income, calculate health and social insurance separately and identify who pays any difference.
- Distinguish a full from partial calendar month and verify any proportional adjustment of the minimum base.
- Verify the document establishing state-insured status, its duration, notification to the insurer, the responsible notifier and deadline.
- Distinguish agreed unpaid leave from an obstacle attributable to the employer before allocating a top-up.
- If payroll was closed on an incorrect status, identify the corrected payroll and insurance filings required.
- Verify whether an exception applies before asserting a top-up obligation. Do not infer state-insured status from a generic disability label.
- Separate payroll reporting, tax withholding, insurance reporting and employment documentation.

## Working time and leave

- Classify shift, overtime, on-call duty, travel, rest, obstacle to work, leave or remote work from the actual facts.
- Verify scheduling, notice, records, compensation and maximum limits separately.
- Distinguish an agreed contractual change from a unilateral instruction and identify the required form.

## Common mistakes

- Treating an employment title or contract label as decisive despite different actual work.
- Mixing employment validity with tax or insurance classification.
- Giving a termination answer without delivery, protected-period or severance analysis.
- Treating invalidity of one degree as equivalent to every disability or state-insured category.
- Applying a current monetary threshold to an earlier payroll period.
- Omitting discrimination and retaliation from an otherwise valid management decision.

## Output

Lead with the employment-law conclusion. Then show decisive facts, effective version, separate wage and insurance effects, required procedure, documents, deadlines, risks and unresolved facts. Use resolved user-facing `https://` sources and never expose raw CODEXIS identifiers.
