#!/usr/bin/env python3

"""Human-readable smoke test for search endpoints.

This script:
- uses the built ./cdx-cli binary directly
- forces CODEXIS_API_URL=http://localhost:8080
- expects auth config via environment or ~/.cdx/.env
- checks all search sources with source-specific example queries
- verifies STRING_CHOICE and BOOLEAN facets using the exact returned facet keys
- deliberately skips DATE_RANGE facets

"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


API_URL = "http://localhost:8080"
ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "cdx-cli"
ENV_FILE = Path.home() / ".cdx" / ".env"
SKIPPED_FACET_TYPES = {"DATE_RANGE"}


@dataclass(frozen=True)
class SourceCase:
    source: str
    query: str
    with_facets: bool


@dataclass
class CheckResult:
    status: str
    label: str
    detail: str


SOURCE_CASES = [
    SourceCase("JD", "náhrada škody", True),
    SourceCase("CR", "občanský zákoník", True),
    SourceCase("COMMENT", "nájem bytu", True),
    SourceCase("ES", "ochrana spotřebitele", True),
    SourceCase("EU", "GDPR", True),
    SourceCase("SK", "občiansky zákonník", True),
    SourceCase("LT", "odpovědnost za škodu", True),
    SourceCase("VS", "pracovní smlouva", True),
    SourceCase("ALL", "insolvence", False),
]


def main() -> int:
    if not BINARY.exists():
        print(f"Missing built binary: {BINARY}")
        print("Build it first with ./build.sh")
        return 1

    if not os.access(BINARY, os.X_OK):
        print(f"Built binary is not executable: {BINARY}")
        return 1

    auth_source = detect_auth_source()
    if auth_source is None:
        print("Missing auth configuration.")
        print("Expected CDX_API_JWT_AUTH in the environment or in ~/.cdx/.env")
        return 1

    print("Search endpoint smoke test")
    print(f"- binary : {BINARY}")
    print(f"- api    : {API_URL}")
    print(f"- auth   : {auth_source}")
    print("- scope  : JD, CR, COMMENT, ES, EU, SK, LT, VS, ALL")

    overall_failed = 0
    overall_passed = 0
    overall_skipped = 0

    for case in SOURCE_CASES:
        passed, skipped, failed = run_source_case(case)
        overall_passed += passed
        overall_skipped += skipped
        overall_failed += failed

    print("\nOverall summary")
    print(f"- passed  : {overall_passed}")
    print(f"- skipped : {overall_skipped}")
    print(f"- failed  : {overall_failed}")

    if overall_failed:
        print("\nSearch endpoint smoke test failed.")
        return 1

    print("\nSearch endpoint smoke test passed.")
    return 0


def run_source_case(case: SourceCase) -> tuple[int, int, int]:
    print(f"\n{case.source} search")
    print(f"- query : {case.query}")

    args = ["search", case.source]
    if case.with_facets:
        args.append("--with-full-facets")
    args.extend(["--query", case.query, "--limit", "1"])

    response = run_cli(args)
    total_results = int(response["totalResults"])
    results = response.get("results") or []
    facets = response.get("availableFilters") or []

    print(f"- total results   : {total_results}")
    print(f"- returned items  : {len(results)}")
    print(f"- returned facets : {len(facets)}")

    checks: list[CheckResult] = []

    if total_results <= 0:
        checks.append(CheckResult("FAIL", "base search", "expected totalResults > 0"))
    else:
        checks.append(CheckResult("PASS", "base search", f"totalResults={total_results}"))

    if not results:
        checks.append(CheckResult("FAIL", "results", "expected at least one result item"))
    else:
        doc_id = results[0].get("docId", "<missing docId>")
        checks.append(CheckResult("PASS", "results", f"first docId={doc_id}"))

    if case.with_facets:
        if not facets:
            checks.append(
                CheckResult("FAIL", "availableFilters", "expected availableFilters in response")
            )
        else:
            checks.append(
                CheckResult("PASS", "availableFilters", f"count={len(facets)}")
            )
            checks.extend(run_facet_checks(case, total_results, facets))
    else:
        if facets:
            checks.append(
                CheckResult("FAIL", "availableFilters", "ALL should not return facets here")
            )
        else:
            checks.append(CheckResult("PASS", "availableFilters", "none returned as expected"))

    for check in checks:
        print(f"[{check.status:<4}] {check.label:<28} {check.detail}")

    passed = sum(1 for check in checks if check.status == "PASS")
    skipped = sum(1 for check in checks if check.status == "SKIP")
    failed = sum(1 for check in checks if check.status == "FAIL")

    print(f"- source summary  : passed={passed} skipped={skipped} failed={failed}")
    return passed, skipped, failed


def run_facet_checks(
    case: SourceCase, base_total: int, facets: list[dict[str, Any]]
) -> list[CheckResult]:
    checks: list[CheckResult] = []

    for facet in facets:
        key = str(facet.get("key"))
        facet_type = str(facet.get("type"))
        label = f"facet {key}"

        if facet_type in SKIPPED_FACET_TYPES:
            checks.append(CheckResult("SKIP", label, f"{facet_type} skipped on purpose"))
            continue

        if facet_type == "STRING_CHOICE":
            checks.append(check_string_choice(case, base_total, facet))
            continue

        if facet_type == "BOOLEAN":
            checks.append(check_boolean(case, base_total, key))
            continue

        checks.append(CheckResult("FAIL", label, f"unsupported facet type: {facet_type}"))

    return checks


def check_string_choice(
    case: SourceCase, base_total: int, facet: dict[str, Any]
) -> CheckResult:
    key = str(facet["key"])
    values = facet.get("values") or []
    if not values:
        return CheckResult("FAIL", f"facet {key}", "STRING_CHOICE facet has no values")

    picked = max(values, key=lambda item: int(item.get("count") or 0))
    chosen_value = str(picked["value"])
    expected = int(picked["count"])

    response = run_cli(
        ["search", case.source, "-"],
        search_payload(case.query, {key: [chosen_value]}),
    )
    actual = int(response["totalResults"])
    status = "PASS" if actual == expected else "FAIL"
    detail = f'value="{chosen_value}" expected={expected} actual={actual} base={base_total}'
    return CheckResult(status, f"facet {key}", detail)


def check_boolean(case: SourceCase, base_total: int, key: str) -> CheckResult:
    true_total = int(
        run_cli(["search", case.source, "-"], search_payload(case.query, {key: True}))[
            "totalResults"
        ]
    )
    false_total = int(
        run_cli(["search", case.source, "-"], search_payload(case.query, {key: False}))[
            "totalResults"
        ]
    )

    changed = true_total != base_total or false_total != base_total
    split = true_total != false_total
    status = "PASS" if changed and split else "FAIL"
    detail = f"true={true_total} false={false_total} base={base_total}"
    return CheckResult(status, f"facet {key}", detail)


def search_payload(query: str, extra_fields: dict[str, Any]) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "query": query,
        "limit": 1,
        "offset": 1,
        "sort": "RELEVANCE",
        "sortOrder": "DESC",
    }
    payload.update(extra_fields)
    return payload


def detect_auth_source() -> str | None:
    if os.environ.get("CDX_API_JWT_AUTH", "").strip():
        return "environment"

    env_vars = read_env_file(ENV_FILE)
    if env_vars.get("CDX_API_JWT_AUTH", "").strip():
        return str(ENV_FILE)

    return None


def read_env_file(path: Path) -> dict[str, str]:
    if not path.exists():
        return {}

    values: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("export "):
            line = line[len("export ") :].strip()
        if "=" not in line:
            continue
        key, raw_value = line.split("=", 1)
        key = key.strip()
        raw_value = raw_value.strip()
        if len(raw_value) >= 2 and raw_value[0] == raw_value[-1] and raw_value[0] in {"'", '"'}:
            raw_value = raw_value[1:-1]
        values[key] = raw_value
    return values


def run_cli(args: list[str], payload: dict[str, Any] | None = None) -> dict[str, Any]:
    env = os.environ.copy()
    env["CODEXIS_API_URL"] = API_URL

    input_text = None
    if payload is not None:
        input_text = json.dumps(payload, ensure_ascii=False)

    completed = subprocess.run(
        [str(BINARY), *args],
        input=input_text,
        capture_output=True,
        text=True,
        env=env,
        check=False,
    )

    if completed.returncode != 0:
        print("\ncdx-cli call failed")
        print(f"- command : {' '.join([str(BINARY), *args])}")
        print(f"- exit    : {completed.returncode}")
        if completed.stdout.strip():
            print("- stdout  :")
            print(indent(completed.stdout.strip()))
        if completed.stderr.strip():
            print("- stderr  :")
            print(indent(completed.stderr.strip()))
        raise SystemExit(1)

    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        print("\ncdx-cli returned invalid JSON")
        print(f"- command : {' '.join([str(BINARY), *args])}")
        print(f"- error   : {error}")
        if completed.stdout.strip():
            print("- stdout  :")
            print(indent(completed.stdout.strip()))
        if completed.stderr.strip():
            print("- stderr  :")
            print(indent(completed.stderr.strip()))
        raise SystemExit(1)


def indent(text: str) -> str:
    return "\n".join(f"  {line}" for line in text.splitlines())


if __name__ == "__main__":
    sys.exit(main())
