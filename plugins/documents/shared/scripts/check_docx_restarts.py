#!/usr/bin/env python3

import argparse
import sys
from pathlib import Path
from xml.etree import ElementTree
from zipfile import BadZipFile, ZipFile

WORD = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"
NUMBERING = "word/numbering.xml"


def structure(element: ElementTree.Element) -> tuple:
    return (
        element.tag,
        sorted(element.attrib.items()),
        (element.text or "").strip(),
        [structure(child) for child in element],
    )


def check_restart_repair(original: Path, repaired: Path) -> None:
    with ZipFile(original) as source, ZipFile(repaired) as result:
        if set(source.namelist()) != set(result.namelist()):
            raise ValueError("The repair changed the DOCX package parts")
        if NUMBERING not in source.namelist():
            raise ValueError("The document has no automatic numbering definitions")
        for name in source.namelist():
            if name != NUMBERING and source.read(name) != result.read(name):
                raise ValueError(f"The restart-only repair changed {name}")
        expected = ElementTree.fromstring(source.read(NUMBERING))
        actual = ElementTree.fromstring(result.read(NUMBERING))
    if expected.tag != f"{WORD}numbering" or actual.tag != f"{WORD}numbering":
        raise ValueError("Invalid numbering root element")
    for level in expected.iter(f"{WORD}lvl"):
        index = int(level.attrib[f"{WORD}ilvl"])
        for restart in level.findall(f"{WORD}lvlRestart"):
            if index > 0 and int(restart.attrib[f"{WORD}val"]) == index:
                level.remove(restart)
    if structure(expected) != structure(actual):
        raise ValueError(
            "The numbering differs beyond removal of redundant default restarts. "
            "Preserve the original list instances, levels, starts and intentional restarts."
        )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Verify a restart-only DOCX repair against the original file. "
        "This checks preservation, not OOXML validity or Word rendering."
    )
    parser.add_argument("original", type=Path)
    parser.add_argument("repaired", type=Path)
    args = parser.parse_args(argv)
    try:
        check_restart_repair(args.original, args.repaired)
    except (OSError, BadZipFile, ElementTree.ParseError, KeyError, ValueError) as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1
    print(
        "PASS: only redundant default restarts changed; Word rendering is unverified."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
