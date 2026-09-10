"""Node id templates in the promptfoo assertions resolve the way the daemon keys entities."""

from tests.promptfoo import assertions

CHAT_ID = "9a9a9a9a-0000-4000-8000-00000000000a"


def test_root_entity_template_encodes_a_single_segment():
    encoded = assertions._resolve_node_id_template("${Automation:abc}", None, None)
    assert assertions._decode_node_id(encoded) == [("Automation", "abc")]


def test_scoped_entity_template_nests_under_the_chat_of_the_run():
    chat = assertions._encode_node_id("Chat", CHAT_ID)
    encoded = assertions._resolve_node_id_template(
        "${TabularExtraction:/home/codexis/ee-tab-{{ run_id }}}", {"run_id": "r1"}, chat
    )
    assert assertions._decode_node_id(encoded) == [
        ("Chat", CHAT_ID),
        ("TabularExtraction", "/home/codexis/ee-tab-r1"),
    ]


def test_scoped_entity_without_a_chat_stays_a_root_segment():
    encoded = assertions._resolve_node_id_template(
        "${TabularExtraction:/x}", None, None
    )
    assert assertions._decode_node_id(encoded) == [("TabularExtraction", "/x")]


def test_decode_rejects_what_is_not_a_node_id():
    assert assertions._decode_node_id("") is None
    assert assertions._decode_node_id("not base64!") is None
    assert assertions._decode_node_id("D1B1YmxpY0lzb2xhdGlvbgxtYXJrZXRwbGFjZXM") == [
        ("PublicIsolation", "marketplaces")
    ]
