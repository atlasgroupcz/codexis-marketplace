#!/usr/bin/env bash
# Checks that quoted passages really appear in the CODEXIS documents they are attributed to.
#
# Reads a JSON array on stdin:
#   [{"id": "c1", "docId": "JD402788", "quote": "…", "part": "paragraf19c"}, …]
# `id` and `part` are optional. Writes a JSON array on stdout, one entry per input:
#   [{"id": "c1", "docId": "JD402788", "status": "verbatim|not_found|unreachable", "detail": "…"}]
#
# Exit code: 0 when every quote was found verbatim, 1 when any was not, 2 on a usage error.
# A quote that cannot be checked is never reported as found.

set -uo pipefail

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
    exit 0
fi

command -v cdx-cli >/dev/null 2>&1 || { echo "citation_checker: cdx-cli not found" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "citation_checker: python3 not found" >&2; exit 2; }

# The program is read into a variable rather than piped in, so the citations on stdin stay
# available to python itself.
CHECKER=$(cat <<'PYTHON'
import json
import re
import subprocess
import sys
import unicodedata

FETCH_TIMEOUT_SECONDS = 60
MIN_QUOTE_CHARS = 12

# Typography differs between what a model echoes back and what the database stores, so compare on a
# normalised form: same case, one kind of space, one kind of quote and dash. Diacritics are kept —
# in Czech legal text they carry meaning.
DASHES = dict.fromkeys(map(ord, "‐‑‒–—―−"), "-")
QUOTES = dict.fromkeys(map(ord, "‘’‚‛′"), "'")
QUOTES.update(dict.fromkeys(map(ord, "“”„‟″"), '"'))


def normalise(text):
    folded = unicodedata.normalize("NFC", text).translate(DASHES).translate(QUOTES)
    return re.sub(r"\s+", " ", folded).strip().lower()


def fetch(doc_id, part):
    address = "cdx://doc/{}/text".format(doc_id)
    if part:
        address += "?part={}".format(part)
    try:
        done = subprocess.run(["cdx-cli", "get", address], capture_output=True, text=True,
                              timeout=FETCH_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        return None, "cdx-cli timed out after {}s".format(FETCH_TIMEOUT_SECONDS)
    except OSError as error:
        return None, "cdx-cli could not be run: {}".format(error)
    if done.returncode != 0:
        return None, "cdx-cli exited {}: {}".format(done.returncode, done.stderr.strip()[:200])
    if not done.stdout.strip():
        return None, "document is empty or was not returned"
    return done.stdout, None


def check(entry, cache):
    doc_id = str(entry.get("docId") or "").strip()
    quote = str(entry.get("quote") or "").strip()
    part = str(entry.get("part") or "").strip()
    result = {"id": entry.get("id"), "docId": doc_id}
    if not doc_id:
        return {**result, "status": "unreachable", "detail": "no docId given, nothing to check against"}
    if len(quote) < MIN_QUOTE_CHARS:
        return {**result, "status": "unreachable",
                "detail": "quote is shorter than {} characters, too weak to verify".format(MIN_QUOTE_CHARS)}

    key = (doc_id, part)
    if key not in cache:
        cache[key] = fetch(doc_id, part)
    text, error = cache[key]
    if text is None:
        return {**result, "status": "unreachable", "detail": error}
    if normalise(quote) in normalise(text):
        return {**result, "status": "verbatim", "detail": "found in the document"}
    return {**result, "status": "not_found",
            "detail": "the document does not contain this passage"}


def main():
    try:
        entries = json.load(sys.stdin)
    except json.JSONDecodeError as error:
        print("citation_checker: stdin is not valid JSON: {}".format(error), file=sys.stderr)
        return 2
    if not isinstance(entries, list):
        print("citation_checker: expected a JSON array of citations", file=sys.stderr)
        return 2

    cache = {}
    results = [check(entry, cache) if isinstance(entry, dict)
               else {"id": None, "docId": "", "status": "unreachable", "detail": "entry is not an object"}
               for entry in entries]
    json.dump(results, sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")
    return 0 if all(r["status"] == "verbatim" for r in results) else 1


sys.exit(main())
PYTHON
)

python3 -c "$CHECKER"
