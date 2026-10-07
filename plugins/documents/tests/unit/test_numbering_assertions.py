import re
from html import escape
from pathlib import Path

import pytest
import yaml

from plugins.documents.tests.promptfoo.numbering_assertions import assert_numbering

CASES = yaml.safe_load(
    (Path(__file__).parents[1] / "promptfoo" / "numbering.config.yaml").read_text()
)["tests"]


def input_xml(case: dict) -> str:
    return "<w:numbering" + case["vars"]["prompt"].split("<w:numbering", 1)[1]


@pytest.mark.parametrize("case", CASES, ids=lambda case: case["description"])
def test_accepts_intended_restart_semantics(case):
    xml = input_xml(case)
    if case is CASES[0]:
        xml = re.sub(r"<w:lvlRestart[^>]*/>", "", xml)
    assert assert_numbering(xml, case)
    assert assert_numbering(f"```xml\n{xml}\n```\n<<CDX_EVAL_TOOL_CALLS>>ignored", case)


def test_rejects_original_redundant_restarts():
    assert not assert_numbering(input_xml(CASES[0]), CASES[0])


@pytest.mark.parametrize(
    "wrapper", ["<p>{}</p>", '<pre><code class="language-xml">{}</code></pre>']
)
def test_reads_xml_from_graphql_html_content(wrapper):
    xml = input_xml(CASES[1])
    assert assert_numbering(wrapper.format(escape(xml)), CASES[1])
    assert not assert_numbering(wrapper.format(escape(input_xml(CASES[0]))), CASES[0])


def test_rejects_replacing_default_restarts_with_never_restart():
    xml = re.sub(r'(<w:lvlRestart w:val=")[12]("/>)', r"\g<1>0\2", input_xml(CASES[0]))
    assert not assert_numbering(xml, CASES[0])


@pytest.mark.parametrize("case", CASES[1:], ids=lambda case: case["description"])
def test_rejects_removing_intentional_restarts(case):
    xml = re.sub(r"<w:lvlRestart[^>]*/>", "", input_xml(case))
    assert not assert_numbering(xml, case)


@pytest.mark.parametrize(
    ("before", "after"),
    [
        ("</w:numbering>", ""),
        ("w:numbering", "w:document"),
        ('w:numId="1"', 'w:numId="0"'),
        ('w:numId="1"', ""),
        ('<w:abstractNumId w:val="1"/>', '<w:abstractNumId w:val="99"/>'),
        ('<w:abstractNumId w:val="1"/>', ""),
        ("</w:num>", '<w:lvlOverride w:ilvl="1"/></w:num>'),
        ('w:ilvl="2"', 'w:ilvl="1"'),
        ('<w:numFmt w:val="decimal"/>', '<w:numFmt w:val="none"/>'),
        ('<w:start w:val="1"/>', '<w:start w:val="5"/>'),
        ('<w:start w:val="1"/>', '<w:start w:val="1"/><w:start w:val="1"/>'),
        ("</w:abstractNum>", '<w:lvl w:ilvl="3"/></w:abstractNum>'),
        ("</w:numbering>", '<w:num w:numId="2"/></w:numbering>'),
        ("</w:numbering>", '<w:abstractNum w:abstractNumId="2"/></w:numbering>'),
    ],
)
def test_rejects_broken_numbering_definitions(before, after):
    xml = re.sub(r"<w:lvlRestart[^>]*/>", "", input_xml(CASES[0]))
    assert not assert_numbering(xml.replace(before, after), CASES[0])


def test_rejects_xml_present_only_in_tool_metadata():
    xml = input_xml(CASES[1])
    assert not assert_numbering(f"Done.<<CDX_EVAL_TOOL_CALLS>>{xml}", CASES[1])


def test_rejects_missing_numbering_identity_and_reference():
    xml = (
        input_xml(CASES[1])
        .replace('w:abstractNumId="2"', "")
        .replace('<w:abstractNumId w:val="2"/>', "<w:abstractNumId/>")
    )
    assert not assert_numbering(xml, CASES[1])
