import subprocess
from pathlib import Path
from zipfile import ZipFile

import pytest
import yaml

from plugins.documents.shared.scripts.check_docx_restarts import (
    check_restart_repair,
    main,
)

CASES = yaml.safe_load(
    (Path(__file__).parents[1] / "promptfoo" / "numbering.config.yaml").read_text()
)["tests"]


def input_xml(case):
    return "<w:numbering" + case["vars"]["prompt"].split("<w:numbering", 1)[1]


def document(path, numbering, body=b"original document", extra=True):
    with ZipFile(path, "w") as archive:
        archive.writestr("word/numbering.xml", numbering)
        archive.writestr("word/document.xml", body)
        if extra:
            archive.writestr("word/styles.xml", b"original styles")
    return path


def corrected_numbering():
    return (
        input_xml(CASES[0])
        .replace('<w:lvlRestart w:val="1"/>', "")
        .replace('<w:lvlRestart w:val="2"/>', "")
    )


@pytest.mark.parametrize("case", CASES, ids=lambda case: case["description"])
def test_accepts_only_default_restart_removal_and_preserves_intentional_restarts(
    tmp_path, case
):
    original = document(tmp_path / "original.docx", input_xml(case))
    repaired = document(
        tmp_path / "repaired.docx",
        corrected_numbering() if case is CASES[0] else input_xml(case),
    )

    check_restart_repair(original, repaired)


def test_rejects_redundant_restarts_left_in_result(tmp_path):
    original = document(tmp_path / "original.docx", input_xml(CASES[0]))

    with pytest.raises(ValueError, match="numbering"):
        check_restart_repair(original, original)


@pytest.mark.parametrize("case", CASES[1:], ids=lambda case: case["description"])
def test_rejects_removing_intentional_restarts(tmp_path, case):
    original = document(tmp_path / "original.docx", input_xml(case))
    restart = case["vars"]["numbering_levels"][-1]["lvlRestart"]
    repaired = document(
        tmp_path / "repaired.docx",
        input_xml(case).replace(f'<w:lvlRestart w:val="{restart}"/>', ""),
    )

    with pytest.raises(ValueError, match="numbering"):
        check_restart_repair(original, repaired)


def test_rejects_fixed_continuation_added_after_a_successful_check(tmp_path):
    numbering = input_xml(CASES[3])
    original = document(tmp_path / "original.docx", numbering)
    repaired = document(tmp_path / "repaired.docx", numbering)
    check_restart_repair(original, repaired)
    document(
        repaired,
        numbering.replace(
            "</w:numbering>",
            '<w:num w:numId="3"><w:abstractNumId w:val="2"/>'
            '<w:lvlOverride w:ilvl="1"><w:startOverride w:val="2"/>'
            "</w:lvlOverride></w:num></w:numbering>",
        ),
    )

    with pytest.raises(ValueError, match="numbering"):
        check_restart_repair(original, repaired)


def test_rejects_changed_paragraph_references(tmp_path):
    original = document(tmp_path / "original.docx", input_xml(CASES[0]))
    repaired = document(
        tmp_path / "repaired.docx", corrected_numbering(), b"new references"
    )

    with pytest.raises(ValueError, match=r"word/document\.xml"):
        check_restart_repair(original, repaired)


def test_rejects_missing_package_parts(tmp_path):
    original = document(tmp_path / "original.docx", input_xml(CASES[0]))
    repaired = document(tmp_path / "repaired.docx", corrected_numbering(), extra=False)

    with pytest.raises(ValueError, match="package parts"):
        check_restart_repair(original, repaired)


def test_accepts_numbering_serialization_changes(tmp_path):
    original = document(tmp_path / "original.docx", input_xml(CASES[0]))
    repaired = document(
        tmp_path / "repaired.docx",
        corrected_numbering().replace("w:", "word:").replace("xmlns:w=", "xmlns:word="),
    )

    check_restart_repair(original, repaired)


def test_rejects_files_without_numbering(tmp_path):
    original = tmp_path / "original.docx"
    with ZipFile(original, "w") as archive:
        archive.writestr("word/document.xml", b"original document")

    with pytest.raises(ValueError, match="numbering"):
        check_restart_repair(original, original)


def test_command_reports_failure_without_claiming_word_validation(tmp_path, capsys):
    original = document(tmp_path / "original.docx", input_xml(CASES[0]))
    repaired = document(tmp_path / "repaired.docx", corrected_numbering())

    assert main([str(original), str(original)]) == 1
    assert "Error:" in capsys.readouterr().err
    assert main([str(original), str(repaired)]) == 0
    assert "Word rendering is unverified" in capsys.readouterr().out


def test_command_reports_unreadable_documents(tmp_path, capsys):
    missing = str(tmp_path / "missing.docx")

    assert main([missing, missing]) == 1
    assert "Error:" in capsys.readouterr().err


def test_installed_command_checks_document_and_is_removed_on_uninstall(tmp_path):
    plugin = Path(__file__).parents[2]
    target = tmp_path / "bin"
    original = document(tmp_path / "original.docx", input_xml(CASES[0]))
    repaired = document(tmp_path / "repaired.docx", corrected_numbering())
    subprocess.run(
        ["bash", str(plugin / "hooks/install-scripts.sh"), "--target", str(target)],
        check=True,
        capture_output=True,
    )
    command = target / "cdx-check-docx-restarts"
    result = subprocess.run(
        [str(command), str(original), str(repaired)], capture_output=True, text=True
    )

    assert result.returncode == 0, result.stderr
    assert "PASS:" in result.stdout

    subprocess.run(
        ["bash", str(plugin / "hooks/uninstall-scripts.sh"), "--target", str(target)],
        check=True,
        capture_output=True,
    )
    assert not command.exists()
