"""Integration tests for run.py's arbitration glue (_arbitrate_config, version
marker) against the real promptfoo --output JSON shape, with a stubbed judge.

No network, no daemon. Run with:
    python3 -m pytest tests/promptfoo/test_run_arbitration.py
"""

from __future__ import annotations

import json
from pathlib import Path

import _arbitration as arb
import run
from assertions import TOOL_CALLS_SENTINEL, TOOL_CALLS_END


def _result_json():
    output = ("ČEZ, a. s., Duhová 2/1444.\n\n" + TOOL_CALLS_SENTINEL
              + json.dumps([{"name": "shell", "input": {"command": "curl -I https://x"}}])
              + TOOL_CALLS_END)
    failing = {
        "testIdx": 0, "success": False,
        "vars": {"prompt": "ověř odkaz",
                 "arbitration": {"correct": ["curl to verify a URL the tool returned"],
                                 "incorrect": ["scrape page content"]}},
        "response": {"output": output},
        "testCase": {"description": "verify the returned link resolves"},
        "gradingResult": {"componentResults": [
            {"pass": False, "reason": "forbidden 'shell' call matched /curl/",
             "assertion": {"type": "python",
                           "value": "file://./assertions.py:assert_no_tool_call"}},
        ]},
    }
    passing = {"testIdx": 1, "success": True, "vars": {"prompt": "x"},
               "response": {"output": "ok"},
               "testCase": {"description": "ok"},
               "gradingResult": {"componentResults": [
                   {"pass": True, "reason": "ok", "assertion": {"type": "regex", "value": "x"}}]}}
    return {"results": {"results": [failing, passing]}}


def _ctx(reply, tmp_path):
    return run.ArbCtx(
        arb=arb, call=lambda messages: reply,
        model="m", pf_version="v", floor=0.6,
        cache={}, cache_path=tmp_path / "cache.json",
        results_dir=tmp_path / ".results",
    )


def _write_out(tmp_path):
    out = tmp_path / "out.json"
    out.write_text(json.dumps(_result_json()))
    return out


def test_arbitrate_config_flips_false_positive_to_pass(tmp_path):
    ctx = _ctx('{"verdict":"CORRECT","confidence":0.9,"rationale":"verify"}', tmp_path)
    ok, note = run._arbitrate_config(Path("ares-ico.config.yaml"), _write_out(tmp_path), ctx)
    assert ok is True
    assert "overruled" in note
    transcript = ctx.results_dir / "transcripts" / "ares-ico.config.yaml__0.txt"
    assert transcript.exists()
    assert "curl -I https://x" in transcript.read_text()
    assert len(ctx.records) == 1
    assert ctx.records[0]["verdict"] == "CORRECT"
    assert ctx.records[0]["passed"] is True


def test_arbitrate_config_keeps_real_failure(tmp_path):
    ctx = _ctx('{"verdict":"INCORRECT","confidence":0.95,"rationale":"scraped"}', tmp_path)
    ok, _ = run._arbitrate_config(Path("ares-ico.config.yaml"), _write_out(tmp_path), ctx)
    assert ok is False


def test_arbitrate_config_judge_error_keeps_fail(tmp_path):
    def boom(messages):
        raise RuntimeError("HTTP 503")
    ctx = run.ArbCtx(arb=arb, call=boom, model="m", pf_version="v", floor=0.6,
                     cache={}, cache_path=tmp_path / "c.json",
                     results_dir=tmp_path / ".results")
    ok, note = run._arbitrate_config(Path("x.config.yaml"), _write_out(tmp_path), ctx)
    assert ok is False
    assert "judge error" in note


def test_arbitrate_config_unreadable_output_is_fail(tmp_path):
    ok, _ = run._arbitrate_config(Path("x.config.yaml"),
                                  tmp_path / "missing.json", _ctx("{}", tmp_path))
    assert ok is False


def test_read_promptfoo_version_marker():
    assert run.read_promptfoo_version() == "0.121.12"
