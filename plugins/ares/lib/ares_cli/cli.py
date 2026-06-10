"""Command-line interface for Czech ARES lookups."""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Callable

from . import __version__
from .errors import AresCliError
from .service import AresService
from .sources import RAW_SOURCES


CommandFunc = Callable[[argparse.Namespace], dict]

BUSINESS_CASES = """\
Business cases:
  ares search <query>
    Supports: "Najdi firmu podle názvu."
    Endpoint: POST /ekonomicke-subjekty/vyhledat
    Call when: user gives a company name or partial name.

  ares company <ico>
    Supports: "Zobraz základní údaje o subjektu."
    Endpoint: GET /ekonomicke-subjekty/{ico}
    Call when: user already has IČO and needs name, address, legal form, status, dates.

  ares officers <ico>
    Supports: "Kdo může za firmu jednat?"
    Endpoint: GET /ekonomicke-subjekty-vr/{ico}
    Call when: user asks about statutory bodies, directors, board members, or signing authority.

  ares trades <ico>
    Supports: "Jaká má firma živnostenská oprávnění?"
    Endpoint: GET /ekonomicke-subjekty-rzp/{ico}
    Call when: user asks about business activities, trades, licences, or scope of business.

  ares owners <ico>
    Supports: "Kdo je skutečný majitel?"
    Endpoint: GET /ekonomicke-subjekty-rpsh/{ico}
    Call when: user asks about beneficial owners, AML, KYC, or ownership/compliance checks.

  ares raw <ico> --source <source>
    Supports: "Ukaž surová data ze zdroje."
    Endpoint: source-specific endpoint
    Call when: debugging, serving advanced users, or needing fields not exposed by simplified commands.

Raw sources:
  basic  /ekonomicke-subjekty/{ico}       basic identification; same source as `company`
  vr     /ekonomicke-subjekty-vr/{ico}    public register; same source as `officers`
  res    /ekonomicke-subjekty-res/{ico}   statistical register data
  rzp    /ekonomicke-subjekty-rzp/{ico}   trade licence register; same source as `trades`
  rpsh   /ekonomicke-subjekty-rpsh/{ico}  beneficial owners; same source as `owners`
"""


def _print_json(payload: dict) -> None:
    print(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=False))


def _not_implemented(command: str) -> dict:
    raise AresCliError(
        f"Command '{command}' is scaffolded but not implemented yet. "
        "Fill ares_cli.client and ares_cli.service to call ARES."
    )


def cmd_search(args: argparse.Namespace) -> dict:
    return AresService().search(args.query, limit=args.limit)


def cmd_company(args: argparse.Namespace) -> dict:
    return AresService().company(args.ico)


def cmd_officers(args: argparse.Namespace) -> dict:
    return AresService().officers(args.ico)


def cmd_trades(args: argparse.Namespace) -> dict:
    return AresService().trades(args.ico)


def cmd_owners(args: argparse.Namespace) -> dict:
    return AresService().owners(args.ico)


def cmd_raw(args: argparse.Namespace) -> dict:
    return AresService().raw(args.ico, source=args.source)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="ares",
        description=(
            "Query Czech ARES public-register data for legal and compliance "
            "checks. Outputs JSON suitable for Claude to summarize in Czech."
        ),
        epilog=BUSINESS_CASES,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--version", action="version", version=f"ares {__version__}")

    sub = parser.add_subparsers(dest="command", required=True)

    search = sub.add_parser(
        "search",
        help="Find economic entities by name.",
        description='Supports: "Najdi firmu podle názvu."',
        epilog=(
            "Endpoint: POST /ekonomicke-subjekty/vyhledat\n"
            "Call when: user gives a company name or partial name.\n"
            "Output: candidate entities to disambiguate before running detail commands."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    search.add_argument("query", help='Company name or name fragment, e.g. "ATLAS".')
    search.add_argument("--limit", type=int, default=10, help="Maximum number of candidates to return.")
    search.set_defaults(func=cmd_search)

    company = sub.add_parser(
        "company",
        help="Show basic company identification.",
        description='Supports: "Zobraz základní údaje o subjektu."',
        epilog=(
            "Endpoint: GET /ekonomicke-subjekty/{ico}\n"
            "Call when: user already has IČO and needs name, address, legal form, status, dates.\n"
            "Output: canonical identity, registered seat, legal form and lifecycle/status fields."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    company.add_argument("ico", help="Czech IČO, with or without leading zeroes.")
    company.set_defaults(func=cmd_company)

    officers = sub.add_parser(
        "officers",
        help="Show statutory representatives and signing authority.",
        description='Supports: "Kdo může za firmu jednat?"',
        epilog=(
            "Endpoint: GET /ekonomicke-subjekty-vr/{ico}\n"
            "Call when: user asks about statutory bodies, directors, board members, or signing authority.\n"
            "Output: statutory body records and způsob jednání if returned by ARES."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    officers.add_argument("ico", help="Czech IČO.")
    officers.set_defaults(func=cmd_officers)

    trades = sub.add_parser(
        "trades",
        help="Show trade licences.",
        description='Supports: "Jaká má firma živnostenská oprávnění?"',
        epilog=(
            "Endpoint: GET /ekonomicke-subjekty-rzp/{ico}\n"
            "Call when: user asks about business activities, trades, licences, or scope of business.\n"
            "Output: trade licences and related živnostenský rejstřík data."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    trades.add_argument("ico", help="Czech IČO.")
    trades.set_defaults(func=cmd_trades)

    owners = sub.add_parser(
        "owners",
        help="Show beneficial-owner and compliance-relevant data.",
        description='Supports: "Kdo je skutečný majitel?"',
        epilog=(
            "Endpoint: GET /ekonomicke-subjekty-rpsh/{ico}\n"
            "Call when: user asks about beneficial owners, AML, KYC, or ownership/compliance checks.\n"
            "Output: beneficial-owner/compliance records from the RPSH source when available."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    owners.add_argument("ico", help="Czech IČO.")
    owners.set_defaults(func=cmd_owners)

    raw = sub.add_parser(
        "raw",
        help="Print a raw ARES source response.",
        description='Supports: "Ukaž surová data ze zdroje."',
        epilog=(
            "Endpoint: source-specific endpoint selected by --source.\n"
            "Call when: debugging, serving advanced users, or needing fields not exposed by simplified commands.\n\n"
            "Sources:\n"
            "  basic  basic identification; same source as `company`\n"
            "  vr     public register; same source as `officers`\n"
            "  res    statistical register data\n"
            "  rzp    trade licence register; same source as `trades`\n"
            "  rpsh   beneficial owners; same source as `owners`"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    raw.add_argument("ico", help="Czech IČO.")
    raw.add_argument(
        "--source",
        required=True,
        choices=list(RAW_SOURCES),
        help="ARES source to fetch: basic, vr, res, rzp or rpsh.",
    )
    raw.set_defaults(func=cmd_raw)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    try:
        payload = args.func(args)
    except AresCliError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2

    _print_json(payload)
    return 0
