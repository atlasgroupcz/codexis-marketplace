"""Known ARES source families used by the CLI."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class SourceSpec:
    """Description of one raw ARES source endpoint family."""

    key: str
    label: str
    path_template: str


RAW_SOURCES: dict[str, SourceSpec] = {
    "basic": SourceSpec("basic", "Základní identifikace", "/ekonomicke-subjekty/{ico}"),
    "vr": SourceSpec("vr", "Veřejný rejstřík", "/ekonomicke-subjekty-vr/{ico}"),
    "res": SourceSpec("res", "Registr ekonomických subjektů", "/ekonomicke-subjekty-res/{ico}"),
    "rzp": SourceSpec("rzp", "Živnostenský rejstřík", "/ekonomicke-subjekty-rzp/{ico}"),
    "rpsh": SourceSpec("rpsh", "Registr skutečných majitelů", "/ekonomicke-subjekty-rpsh/{ico}"),
}
