---
template: case-law-research
description: >
  A question that has to be answered from what the courts actually decide: what the settled
  practice is, which decision carries it, whether it still holds under today's wording, and
  where the courts disagree. Not for reading the client's file and not for writing the client
  opinion.
jurisdictions: CZ SK EU
args:
  - question (required) - the legal question, in the user's own words
  - jurisdiction - CZ, SK or EU; defaults to CZ
  - asOf - the date the answer must be true for; defaults to today
i18n:
  cs:
    displayName: Rešerše judikatury
  en:
    displayName: Case law research
  sk:
    displayName: Rešerš judikatúry
---

Three phases: the question is broken into issues, each issue is searched for decisions, and every
cited decision is verified against its own reasoning before the report is written.

## Before you run it

Ask for `question` in the user's own words. Do not paraphrase it into legal language first - the
phases rewrite it themselves, and a question already narrowed to one statute hides the line of case
law that answers it.

Pass `jurisdiction` only when the user named one. Pass `asOf` only when the answer has to be true
for a date other than today, typically because the facts are older than the current wording.

## What comes back

A finished research report in markdown, plus the same text as a Word document. Print the report to
the user whole and verbatim - every heading, table, citation and quoted passage. Those quotes are
what the user asked for; a summary of them is not the deliverable, and the file is a copy, not a
replacement for the answer.

## When not to use it

Not for a single decision the user already named - open it and read it. Not for reading the
client's own documents. Not for writing the opinion itself: this gathers and verifies what the
courts held, and the legal conclusion stays with you.
