"""Shared test-data builders for katastr unit tests."""


def _cuzk_record(stav="V řízení", stav_uhrady="N", operations=None, record_id="api-id-xyz"):
    return {
        "id": record_id,
        "stav": stav,
        "stavUhrady": stav_uhrady,
        "datumPrijeti": "2026-04-01",
        "provedeneOperace": operations or [],
    }
