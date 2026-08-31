---
uuid: 2f9253f9-2a72-4e95-ac25-1699f5f940ed
name: codexis-bozp
icon: icon.svg
version: 1.0.0
jurisdictions: [CZ, EU]
description: Use when a Czech legal question concerns occupational health and safety, prevention of workplace risks, work accidents, remote-work injuries, safety training, protective equipment, occupational medical services, work categories, shared workplaces, inspections or employer liability. Standalone BOZP methodology using CODEXIS.
i18n:
  cs:
    displayName: "CODEXIS - BOZP"
    summary: "České právo bezpečnosti a ochrany zdraví při práci, prevence rizik a pracovní úrazy."
    examplePrompts:
      - "Posuď, zda může být úraz při práci z domova pracovním úrazem a jaké podklady jsou rozhodné."
      - "Jaké povinnosti má zaměstnavatel při souběhu více firem na jednom pracovišti?"
      - "Prověř, zda naše evidence, školení a poskytování OOPP odpovídají aktuálním předpisům."
  en:
    displayName: "CODEXIS - Czech OHS"
    summary: "Czech occupational health and safety law, risk prevention and workplace accidents."
    examplePrompts:
      - "Assess whether an injury while working from home can be a workplace accident and which facts matter."
      - "What duties apply when employees of several companies share one workplace?"
      - "Check whether our records, safety training and protective equipment comply with current Czech law."
  sk:
    displayName: "CODEXIS - BOZP ČR"
    summary: "České právo bezpečnosti a ochrany zdravia pri práci, prevencia rizík a pracovné úrazy."
    examplePrompts:
      - "Posúď, či môže byť úraz pri práci z domu pracovným úrazom a ktoré skutočnosti sú rozhodujúce."
      - "Aké povinnosti platia pri súbehu viacerých firiem na jednom pracovisku?"
      - "Over, či naša evidencia, školenia a poskytovanie OOPP zodpovedajú aktuálnym českým predpisom."
---

# CODEXIS - BOZP

Use `cdx-cli` as the primary source for Czech and EU legal research. Assume it is installed and authenticated. Use `search`, `get` and `schema`; do not run setup probes before the first real request.

## Mandatory analysis

1. Fix the relevant date, workplace, activity, employment relationship and every actor before stating a rule.
2. Classify the issue as prevention, technical operation, occupational health, accident, inspection, liability or compensation. Handle each applicable track separately.
3. Resolve the effective version of every statute with `/versions`. For a known statute, fetch it directly instead of starting with a broad search.
4. Read the operative provision together with definitions, delegated regulations, cross-references and transitional provisions.
5. Search case law when the issue turns on causal connection, the meaning of fulfilling work tasks, release from liability, evidentiary standards or compensation.
6. Stress-test the conclusion against a private activity, a break, a contractor, a shared workplace, remote work and a special-risk operation when plausible.
7. State the conclusion, decisive facts, duties, evidence, deadlines, responsible actor and unresolved assumptions separately.

## Source stack

| Layer | Typical sources | Purpose |
|---|---|---|
| Employment | Labour Code | Prevention, employee duties, workplace accidents and compensation |
| Safety organization | Act No. 309/2006 Coll. | Further safety requirements, shared workplaces and coordinators |
| Health protection | Public health legislation and implementing rules | Work categories, exposure limits and occupational health |
| Technical safety | Reserved technical equipment and sector rules | Equipment-specific duties and professional competence |
| Procedure | Inspection and administrative procedure rules | Inspection, remedial measures, offences and appeals |
| Interpretation | Czech and EU case law, official guidance, commentary | Ambiguous concepts and evidentiary questions |

Do not present an internal policy, technical standard or official guidance as a statute. Explain its legal role.

## Decision checks

### Workplace accident

- Identify the exact activity at the moment of injury and its temporal, local and substantive connection to work.
- Compare the declared remote-work arrangement with the actual work schedule and working-time records.
- Separate reporting and recording from the legal conclusion that an event is a workplace accident.
- Check breaks, commuting, private deviations, remote-work arrangements and evidence available to both sides.
- Assign the burden of allegation and proof for each disputed fact and distinguish the roles of the employer, statutory insurer and court.
- Analyze employer liability and every claimed ground for full or partial release independently.
- Cover the accident record, notifications, investigation, corrective action and heads of compensation.

### Prevention and shared workplaces

- Identify who controls the workplace and which employers, contractors and self-employed persons are present.
- Allocate information, coordination and prevention duties to named actors instead of saying only that everyone is responsible.
- Link each proposed measure to an identified risk and verify whether a stricter sector rule applies.

### Training, equipment and occupational health

- Distinguish training content, frequency, documentation and verification of knowledge.
- Distinguish collective protection from personal protective equipment and explain why the selected measure is necessary.
- Determine the work category, relevant risk factor and medical-service regime before giving an examination schedule.
- Verify every interval, limit and threshold in the effective text. Never supply one from memory.

## Common mistakes

- Treating an injury during scheduled remote work as automatically occupational.
- Citing only the Labour Code while omitting implementing or technical rules.
- Confusing evidence of an accident with the substantive test for employer liability.
- Giving a generic checklist without assigning duties to the employer, employee, contractor or coordinator.
- Mixing versions from different dates or ignoring a transitional provision.
- Relying on the title of a provision without reading its conditions and exceptions.

## Output

Lead with a short conditional conclusion. Then provide the legal classification, verified source stack, duties by actor, practical procedure, evidence and open factual questions. Link cited sources with the resolved `https://` URL returned by `cdx-cli`; never expose `cdx://` identifiers or raw document IDs to the user.
