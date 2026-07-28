from __future__ import annotations

import os
import stat
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parent.parent
INSTALLER = REPO_ROOT / "plugin-runtime" / "install-executable.sh"


def run_installer(source: Path, target: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "bash",
            "-c",
            'source "$1"; install_executable "$2" "$3"',
            "install-executable",
            str(INSTALLER),
            str(source),
            str(target),
        ],
        capture_output=True,
        text=True,
        check=False,
    )


@pytest.fixture
def executable(tmp_path: Path) -> tuple[Path, Path]:
    source = tmp_path / "source"
    target = tmp_path / "bin" / "target"
    source.write_bytes(b"expected executable\n")
    return source, target


def test_installs_missing_executable(executable: tuple[Path, Path]) -> None:
    source, target = executable

    result = run_installer(source, target)

    assert result.returncode == 0, result.stderr
    assert target.read_bytes() == source.read_bytes()
    assert target.stat().st_mode & stat.S_IXUSR


def test_keeps_matching_executable_unchanged(executable: tuple[Path, Path]) -> None:
    source, target = executable
    assert run_installer(source, target).returncode == 0
    before = target.stat()

    result = run_installer(source, target)

    after = target.stat()
    assert result.returncode == 0, result.stderr
    assert "Verified target" in result.stdout
    assert after.st_ino == before.st_ino
    assert after.st_mtime_ns == before.st_mtime_ns


@pytest.mark.parametrize("content", [b"", b"corrupted executable\n"])
def test_repairs_invalid_executable(
    executable: tuple[Path, Path],
    content: bytes,
) -> None:
    source, target = executable
    target.parent.mkdir()
    target.write_bytes(content)
    target.chmod(0o755)

    result = run_installer(source, target)

    assert result.returncode == 0, result.stderr
    assert target.read_bytes() == source.read_bytes()


def test_repairs_non_executable_target(executable: tuple[Path, Path]) -> None:
    source, target = executable
    target.parent.mkdir()
    target.write_bytes(source.read_bytes())
    target.chmod(0o644)

    result = run_installer(source, target)

    assert result.returncode == 0, result.stderr
    assert os.access(target, os.X_OK)


def test_rejects_empty_source(executable: tuple[Path, Path]) -> None:
    source, target = executable
    source.write_bytes(b"")

    result = run_installer(source, target)

    assert result.returncode != 0
    assert not target.exists()


def test_runtime_helper_is_in_distribution_inputs() -> None:
    exclusions = (REPO_ROOT / "dist-exclusions.txt").read_text()

    assert "/plugin-runtime/" not in exclusions


def test_every_binary_installer_uses_integrity_helper() -> None:
    installers = sorted((REPO_ROOT / "plugins").glob("*/hooks/install-binaries.sh"))
    offenders = [
        str(path.relative_to(REPO_ROOT))
        for path in installers
        if "plugin-runtime/install-executable.sh" not in path.read_text()
    ]

    assert installers
    assert not offenders


def test_codexis_installer_repairs_cdx_cli_and_cdxctl(tmp_path: Path) -> None:
    plugin_dir = REPO_ROOT / "plugins" / "codexis"
    target_dir = tmp_path / "home" / ".local" / "bin"
    environment = {
        **os.environ,
        "HOME": str(tmp_path / "home"),
        "PLUGIN_DIR": str(plugin_dir),
        "TARGET_BIN_DIR": str(target_dir),
    }

    first = subprocess.run(
        ["bash", str(plugin_dir / "hooks" / "install-binaries.sh")],
        capture_output=True,
        text=True,
        env=environment,
        check=False,
    )
    assert first.returncode == 0, first.stderr
    (target_dir / "cdx-cli").write_bytes(b"")
    (target_dir / "cdxctl").write_bytes(b"corrupted")

    second = subprocess.run(
        ["bash", str(plugin_dir / "hooks" / "install-binaries.sh")],
        capture_output=True,
        text=True,
        env=environment,
        check=False,
    )

    assert second.returncode == 0, second.stderr
    assert (target_dir / "cdx-cli").read_bytes() == (plugin_dir / "bin" / "cdx-cli").read_bytes()
    assert (target_dir / "cdxctl").read_bytes() == (plugin_dir / "bin" / "cdxctl").read_bytes()
