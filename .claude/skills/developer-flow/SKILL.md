---
name: developer-flow
description: Use when implementing a feature or bugfix in codexis-marketplace (a plugin, skill, CLI, component, automation or the test harness) that should end in a GitLab merge request, including trying the change on the dev daemon and attaching screenshots or video to the MR. Not for read-only investigation or advice.
---

# Developer Flow - codexis-marketplace

Take a change from a clean branch to a merge request: use cases first, a test per use case, a
run on the dev daemon, and an MR whose evidence lives in GitLab, not in the repository.

All paths below are fixed. Do not search for them. `$SKILL_DIR` is the directory this SKILL.md
was loaded from; run the helpers from there, because a worktree cut from `devel` may not have
them yet.

| What | Where |
| --- | --- |
| Main checkout | `~/IdeaProjects/codexis-marketplace` (often on someone else's branch - never edit it) |
| Source repo (private) | `origin` = `ssh://git@gitlab.agrp.dev:2022/profidata/codexis-marketplace.git`, glab repo `profidata/codexis-marketplace` |
| Same repo over HTTPS, as the daemon clones it | `https://gitlab.agrp.dev/profidata/codexis-marketplace.git` |
| Public dist (never push here) | `gh` = `github.com/atlasgroupcz/codexis-marketplace` |
| Dev daemon | `https://vm.dev.internal.codexis.ai`, test user `umtest@atlasgroup.cz` / `test12` |
| Dev daemon helper | `node $SKILL_DIR/scripts/devel-vm.cjs` |
| Evidence upload | `bash $SKILL_DIR/scripts/upload-evidence.sh` |
| Evidence and MR text | `$CDX_EVIDENCE_DIR` = `${TMPDIR:-/tmp}/cdx-evidence/<slug>`, outside every worktree |
| Dev daemon logs | Grafana Loki `P8E80F9AEF21F6940`, `{app="cdx-daemon", cluster="legal-machine-dev"}` |

## 1. Pick the target and isolate the work

Choose the MR target first, because the branch is cut from it:

- **`devel`** when the change needs an end-to-end check on the daemon: anything a user or the
  model reaches through a plugin (skill text, CLI, component, automation, manifest, install hooks).
- **`main`** when it does not: repo tooling, CI, the test harness, docs, this skill.

```bash
git -C ~/IdeaProjects/codexis-marketplace fetch origin
git -C ~/IdeaProjects/codexis-marketplace worktree add \
  ../codexis-marketplace-<slug> -b <feat|fix|chore>/<slug> origin/<devel|main>
cd ~/IdeaProjects/codexis-marketplace-<slug>
export CDX_EVIDENCE_DIR="${TMPDIR:-/tmp}/cdx-evidence/<slug>"
```

## 2. Write the use cases

Before editing, write a numbered list of use cases. Each one names an input and the observable
result, for example *"Jan Novák, born 1980-05-01, no proceeding -> `{"rizeni": []}` and the
answer says plainly there is none"*. Include the use case that fails today. The same list drives
the tests (step 3), the dev daemon run (step 4) and the MR (step 6).

When the plugin wraps an external API, call that API directly for each use case (curl the real
request) before relying on its documentation.

## 3. Implement and test every use case

- Follow the repo `CLAUDE.md` (no em-dashes, test tiers, README and example prompt rules). When
  you rename a flag, command or field, `git grep` the old name across the plugin (skill,
  references, README, CLI help, tests) and change every hit.
- One test per use case, named after the behavior:
  - Python: `plugins/<name>/tests/unit/test_*.py`. Mock only the transport to the external
    service (for example `monkeypatch.setattr(api_client, "_post", ...)`), never the code under test.
  - Rust: inline `#[cfg(test)]` or `plugins/<name>/src/<crate>/tests/*.rs`.
  - What the model says or which command it runs: a promptfoo scenario in
    `plugins/<name>/tests/promptfoo/*.config.yaml`, with `input_regex` for the command and
    `not-regex` for the bug's wording. The eval tier needs a daemon token and is opt-in, so check
    each regex locally in Python against one good and one bad answer; the step 4 chats are the
    live run of the same scenarios.
- Prove the code tests catch the bug: run them against the old code (`git stash` the fix, or
  `git show origin/<target>:<file> > <file>`), see them fail, restore the fix.
- Coverage of changed executable code >= 90%:
  `uv run --frozen --with pytest-cov pytest plugins/<name>/tests/unit --cov=plugins/<name>/lib --cov-report=term-missing`.
  A change with no executable lines (skill text, README, manifest) has no coverage number - say so
  in the MR.
- Bump `version` in `plugins/<name>/.claude-plugin/plugin.json` when a plugin changes, so the
  daemon offers the update: patch for a fix, minor for a feature.
- Gates, from the worktree root:
  `uv run --frozen ruff format plugins/<name>`, then
  `uv run --frozen python tests/run.py check` and `uv run --frozen python tests/run.py test`
  (`--only backend` when nothing under `frontend/` changed).
- Commit: `git add <the change, its tests, the version bump>` - never `git add -A` - and
  `git commit` with a conventional message (`fix(<plugin>): ...`), body saying why, ending with
  the attribution line from the session instructions.

## 4. Try it on the dev daemon (target `devel` only)

The daemon clones the GitLab branch over HTTPS, so push to `origin` only. Never push a source
branch to `gh`: it is the public dist, and only the Jenkins publish job writes to it. The branch
must still exist when the daemon clones it, so do this before the MR merges (merging deletes it).

The test user is shared. `use-branch` changes its marketplace for everyone until `restore`, so
keep the window short and always restore.

One-time setup on a machine (Google Chrome must be installed):

```bash
npm install --prefix ~/.cache/cdx-devel-vm playwright@1.59.1
~/.cache/cdx-devel-vm/node_modules/.bin/playwright install ffmpeg
```

Then, from the worktree root:

```bash
git push -u origin <branch>
S=$SKILL_DIR/scripts/devel-vm.cjs
node $S chat <failing-use-case>-before "<prompt>"   # fix only: record the bug on the published plugin
node $S use-branch <branch>        # builtin marketplace -> GitLab <branch>
node $S status                     # the bumped version is offered
node $S install <plugin>           # installs or replaces the installed plugin with the branch version
node $S chat <use-case-slug> "<prompt in Czech, as a user would ask>"   # once per use case
node $S restore                    # ALWAYS: marketplace back to the public GitHub dist
node $S install <plugin>           # ALWAYS: plugin back to the published version
```

`chat` writes `<slug>.png`, `<slug>.webm` and `<slug>.json` (tool calls with their input and
output, plus the answer). For each use case, read the JSON: the model called the plugin's CLI and
not `curl`, the CLI output is the new one, and the answer matches the use case. Open the PNG and
look at it. Add `HEADED=1` to watch the browser. A 503 means the VM is waking up; the script retries.

If `use-branch` fails, the error printed is GraphQL JSON with `extensions.executionId`. Search
Loki over the last 15 minutes for
`{app="cdx-daemon", cluster="legal-machine-dev"} |= "<executionId>"`, then for the warning logged
just before it with the same `traceId`. `Remote branch ... not found` means the branch was not
pushed or the merge deleted it: push and retry once. Anything else: run `restore`, stop, and
report the log line.

## 5. Final check

- `git status` in the worktree is clean apart from your commits, and no screenshot, video, JSON
  or `mr.md` is tracked. Evidence is never committed, not even under `tests/fixtures/`.
- `git diff origin/<target>...` contains only the change, its tests and the version bump.

## 6. Open the merge request

1. `git push -u origin <branch>`
2. `bash $SKILL_DIR/scripts/upload-evidence.sh "$CDX_EVIDENCE_DIR"` - prints one markdown line
   per PNG and video (`![name](/uploads/...)`); GitLab plays the `.webm` lines inline.
3. Write `$CDX_EVIDENCE_DIR/mr.md` in English with these sections, pasting the upload lines:
   - `## What & why`: the problem the user saw, its cause, and the fix.
   - `## Changes`: one line per touched file.
   - `## Verification`: every command you ran and its result, and which tests failed on the old code.
   - `## Evidence`: per use case, the prompt, the command the model ran, the answer, the
     screenshot and the video; for a fix, the `-before` recording first. For a `main`-target
     change with nothing to show on the daemon, say so.
   - `## Notes`: limits and follow-ups.
   - The attribution line from the session instructions, last.
4. Create it:

```bash
glab mr create -R profidata/codexis-marketplace \
  --source-branch <branch> --target-branch <devel|main> \
  --title "<type>(<plugin>): <summary>" --remove-source-branch --yes \
  --description "$(cat "$CDX_EVIDENCE_DIR/mr.md")"
```

## 7. After the merge

`git -C ~/IdeaProjects/codexis-marketplace worktree remove ../codexis-marketplace-<slug>` and
`rm -rf "$CDX_EVIDENCE_DIR"` - the uploads stay in GitLab.
