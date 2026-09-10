"""Unit tests for the file lookup on DaemonClient.

The daemon scopes file access to one isolated environment, so a lookup that
names no isolation is refused before anything reads the filesystem. These cover
what goes on the wire and the FileEntry translation the callers read.
"""

from __future__ import annotations

import sys

import pytest

from tests.harness.daemon_client import DaemonClient

CHAT_NODE_ID = "Q2hhdDpjaGF0LTE"


def test_get_entry_names_the_isolation_it_looks_in(monkeypatch):
    client = DaemonClient("http://x", "tok")
    seen: dict = {}

    def _gql_data(query, variables=None, url=None):
        seen["variables"] = variables
        return {"getEntry": {"name": "memo.docx", "type": "FILE"}}

    monkeypatch.setattr(client, "gql_data", _gql_data)

    client.get_entry(CHAT_NODE_ID, "/home/codexis/memo.docx")

    assert seen["variables"] == {
        "isolationId": CHAT_NODE_ID,
        "path": "/home/codexis/memo.docx",
    }


@pytest.mark.parametrize(
    ("kind", "flags"),
    [
        ("FILE", {"isFile": True, "isDirectory": False, "isSymlink": False}),
        ("DIRECTORY", {"isFile": False, "isDirectory": True, "isSymlink": False}),
        ("SYMLINK", {"isFile": False, "isDirectory": False, "isSymlink": True}),
    ],
)
def test_get_entry_translates_the_entry_type(monkeypatch, kind, flags):
    client = DaemonClient("http://x", "tok")
    monkeypatch.setattr(
        client,
        "gql_data",
        lambda *a, **k: {"getEntry": {"name": "memo.docx", "type": kind}},
    )

    entry = client.get_entry(CHAT_NODE_ID, "/home/codexis/memo.docx")

    assert entry == {"name": "memo.docx", **flags}


def test_get_entry_is_none_when_nothing_is_there(monkeypatch):
    client = DaemonClient("http://x", "tok")
    monkeypatch.setattr(client, "gql_data", lambda *a, **k: {"getEntry": None})

    assert client.get_entry(CHAT_NODE_ID, "/home/codexis/gone") is None


if __name__ == "__main__":
    sys.exit(pytest.main([__file__]))
