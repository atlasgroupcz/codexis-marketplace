"""Command-line interface for Czech ARES lookups."""

from __future__ import annotations

import argparse
import json
import sys

from . import __version__
from .errors import AresCliError
from .service import AresService
from .sources import RAW_SOURCES


BUSINESS_CASES = """\
Business cases:
  ares search <query>
    Supports: "Najdi firmu podle názvu."
    Endpoint: POST /ekonomicke-subjekty/vyhledat
    Call when: user gives a company name or partial name.
    Output: locally ranked candidates with name, IČO, seat, legal form, primary source and registration states.

  ares company <ico>
    Supports: "Zobraz základní údaje o subjektu."
    Endpoint: GET /ekonomicke-subjekty/{ico}
    Call when: user already has IČO and needs name, address, legal form, status, dates.
    Output: company card with name, IČO, DIČ, seat, legal form, dates, update, primary source, registrations and CZ-NACE.

  ares officers <ico>
    Supports: "Kdo může za firmu jednat?"
    Endpoint: GET /ekonomicke-subjekty-vr/{ico}
    Call when: user asks about statutory bodies, directors, board members, or signing authority.
    Output: statutory bodies, members, functions, periods, signing authority, file number, capital, shareholders/partners, execution/insolvency/bankruptcy fields if returned by VR.

  ares trades <ico>
    Supports: "Jaká má firma živnostenská oprávnění?"
    Endpoint: GET /ekonomicke-subjekty-rzp/{ico}
    Call when: user asks about business activities, trades, licences, or scope of business.
    Output: trade licences, type, dates, validity, suspension/interruption, conditions, fields, responsible persons, premises and trade/premises states.

  ares raw <ico> --source <source>
    Supports: "Ukaž surová data ze zdroje."
    Endpoint: source-specific endpoint
    Call when: debugging, serving advanced users, or needing fields not exposed by simplified commands.
    Output: original JSON from the selected ARES source, without interpretation.

Raw sources:
  basic  /ekonomicke-subjekty/{ico}       basic identification; same source as `company`
  vr     /ekonomicke-subjekty-vr/{ico}    public register; same source as `officers`
  res    /ekonomicke-subjekty-res/{ico}   statistical register data
  rzp    /ekonomicke-subjekty-rzp/{ico}   trade licence register; same source as `trades`
"""


def _print_json(payload: dict) -> None:
    print(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=False))


def cmd_search(args: argparse.Namespace) -> dict:
    return AresService().search(args.query, limit=args.limit)


def cmd_company(args: argparse.Namespace) -> dict:
    return AresService().company(args.ico)


def cmd_officers(args: argparse.Namespace) -> dict:
    return AresService().officers(args.ico)


def cmd_trades(args: argparse.Namespace) -> dict:
    return AresService().trades(args.ico)


def cmd_raw(args: argparse.Namespace) -> dict:
    return AresService().raw(args.ico, source=args.source)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="ares",
        description=(
            "Query Czech ARES public-register data for legal checks. "
            "Mapped commands output JSON with an echo block; raw outputs original ARES JSON."
        ),
        epilog=BUSINESS_CASES,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--version", action="version", version=f"ares {__version__}")

    sub = parser.add_subparsers(
        dest="command",
        required=True,
        metavar="{search,company,officers,trades,raw}",
    )

    search = sub.add_parser(
        "search",
        help="Find economic entities by name.",
        description='Supports: "Najdi firmu podle názvu."',
        epilog=(
            "Endpoint: POST /ekonomicke-subjekty/vyhledat\n"
            "Call when: user gives a company name or partial name.\n"
            "Output: echo + locally ranked kandidati[].nazev, ico, sidlo, pravniForma, primarniZdroj, stavRegistraci.\n"
            "Note: --limit controls returned candidates; the CLI may fetch a wider pool to avoid hiding exact-name matches behind subsidiaries."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    search.add_argument("query", help='Company name or name fragment, e.g. "ATLAS".')
    search.add_argument("--limit", type=int, default=10, help="Maximum number of ranked candidates to return.")
    search.set_defaults(func=cmd_search)

    company = sub.add_parser(
        "company",
        help="Show basic company identification.",
        description='Supports: "Zobraz základní údaje o subjektu."',
        epilog=(
            "Endpoint: GET /ekonomicke-subjekty/{ico}\n"
            "Call when: user already has IČO and needs name, address, legal form, status, dates.\n"
            "Output: echo + kartaSubjektu with nazev, ico, dic, sidlo, pravniForma, datumVzniku/Zaniku, datumAktualizace, primarniZdroj, registrace, czNace."
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
            "Output: echo + VR zaznamy with statutarniOrgany, clenove, funkce, obdobi, zpusobJednani, spisovaZnacka, zakladniKapital, spolecnici/akcionari, exekuce/insolvence/konkursy if returned."
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
            "Output: echo + RZP zaznamy with zivnosti, druh, dates, platnost, pozastaveni/preruseni, podminky, obory, odpovedniZastupci, provozovny, zivnostiStav/provozovnyStav."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    trades.add_argument("ico", help="Czech IČO.")
    trades.set_defaults(func=cmd_trades)

    raw = sub.add_parser(
        "raw",
        help="Print a raw ARES source response.",
        description='Supports: "Ukaž surová data ze zdroje."',
        epilog=(
            "Endpoint: source-specific endpoint selected by --source.\n"
            "Call when: debugging, serving advanced users, or needing fields not exposed by simplified commands.\n\n"
            "Output: original JSON without interpretation.\n\n"
            "Sources:\n"
            "  basic  basic identification; same source as `company`\n"
            "  vr     public register; same source as `officers`\n"
            "  res    statistical register data\n"
            "  rzp    trade licence register; same source as `trades`"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    raw.add_argument("ico", help="Czech IČO.")
    raw.add_argument(
        "--source",
        required=True,
        choices=list(RAW_SOURCES),
        help="ARES source to fetch: basic, vr, res or rzp.",
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
