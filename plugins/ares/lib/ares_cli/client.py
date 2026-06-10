"""HTTP client boundary for ARES public APIs."""

from __future__ import annotations

from typing import Any

from .errors import AresCliError


BASE_URL = "https://ares.gov.cz/ekonomicke-subjekty-v-be/rest"


class AresClient:
    """Small transport wrapper.

    The implementation should keep raw HTTP details here so the CLI and skill
    can rely on stable, legal-domain commands instead of endpoint trivia.
    """

    def get_json(self, path: str) -> dict[str, Any]:
        raise AresCliError(
            f"HTTP GET is not implemented yet for {BASE_URL}{path}. "
            "Add urllib/request handling, status mapping and JSON parsing here."
        )

    def post_json(self, path: str, payload: dict[str, Any]) -> dict[str, Any]:
        raise AresCliError(
            f"HTTP POST is not implemented yet for {BASE_URL}{path}. "
            "Add urllib/request handling, status mapping and JSON parsing here."
        )
