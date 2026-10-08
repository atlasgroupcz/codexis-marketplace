# cdxctl Chat Search Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Let an assistant use the existing chat search with an optional project filter through marketplace cdxctl.

**Architecture:** Add a single command which calls the existing GraphQL search. Reuse authentication, project lookup and output infrastructure. Keep matching and ranking in the search backend.

**Tech Stack:** Rust, Clap, reqwest, GraphQL, pytest, local cdx-daemon Compose stack.

## Acceptance scenario

1. Search for a term present in chats from two projects and receive the backend results.
2. Repeat with a project name, UUID and Node ID and receive results from that project.
3. Request a page using limit and offset and receive that backend page.
4. Read JSON or table output and identify each chat, project and matching excerpt.
5. Receive a nonzero exit for invalid input, an unknown or ambiguous project, or a backend error.

## Task 1: Command contract and implementation

- Add parser tests in `plugins/codexis/src/cdxctl/src/main.rs` and run `cargo test -p cdxctl chat_search` to observe the missing command.
- Add `commands/chat.rs`, register it in `commands/mod.rs` and `main.rs`, and add `queries/chat_search.graphql` through `graphql.rs`.
- Reuse project fetching and Node ID encoding from `commands/project.rs` and `commands/mod.rs`; require an unambiguous existing project.
- Add loopback HTTP tests for exact GraphQL variables, preservation of results, project selection and failures. Run focused tests, then `cargo test -p cdxctl`.

## Task 2: Agent instructions and contract validation

- Baseline the current `plugins/codexis/skills/cdxctl/SKILL.md` with a reference retrieval scenario.
- Add the command, optional filter, pagination and search trigger to that skill. Repeat the reference scenario.
- Run skill conformance and GraphQL validation against the actual daemon schema. Run scoped Rust formatting and coverage.

## Task 3: Local verification and review evidence

- Start the current cdx-daemon checkout using `./start.sh full` and use its printed front door.
- Build the Linux cdxctl binary through the repository's build helper and install it only into the local test user's environment.
- Prepare identifiable local chat fixtures in two projects, then execute the acceptance scenario through cdxctl.
- Record actual commands and output, screenshots and a short video. Leave the local application running for the user's testing.
- Obtain an independent review of the addition and fix concrete findings.

## Task 4: Delivery

- Use cdx-daemon's `.gitlab/merge_request_templates/Feature.md` because marketplace has no MR templates.
- Include the scoped command relations diagram, checks and uploaded evidence in a Draft MR against marketplace `main`.
- Push without bypassing hooks, verify remote Draft state, and give the user the MR, local URL and repeatable test commands.

## Scope boundaries

The user requested only a wrapper for existing search. Do not add chat retrieval, summary generation, date filters or change backend matching. Other application interfaces keep their existing behavior. This Rust-only change does not require Java or frontend test suites or new UI BDD scenarios.
