from __future__ import annotations

import re
import tempfile
from pathlib import Path
from unittest.mock import patch

import pytest
from atlasctl.configs.command import normalize_config_key
from atlasctl.core.context import RunContext
from atlasctl.core.fs import ensure_evidence_path
from hypothesis import given, settings
from hypothesis import strategies as st


@pytest.mark.unit
@given(st.from_regex(r"[A-Za-z][A-Za-z0-9_-]{0,31}", fullmatch=True))
def test_config_key_normalization_is_stable_for_valid_keys(raw: str) -> None:
    normalized = normalize_config_key(raw)
    assert normalized == raw.strip().replace("-", "_").upper()
    assert re.fullmatch(r"[A-Z][A-Z0-9_]*", normalized)


@pytest.mark.unit
@given(st.from_regex(r"[a-z0-9-]{1,16}", fullmatch=True))
def test_run_id_format_contains_git_sha_suffix(profile: str) -> None:
    with patch.dict("os.environ", {}, clear=True):
        ctx = RunContext.from_args(None, None, profile, False)
        assert re.fullmatch(r"atlas-\d{8}-\d{6}-[0-9a-f]{7,40}", ctx.run_id)


@pytest.mark.unit
@given(st.lists(st.from_regex(r"[a-z0-9_-]{1,10}", fullmatch=True), min_size=1, max_size=5))
@settings(deadline=None)
def test_evidence_paths_stay_under_allowed_root(chunks: list[str]) -> None:
    with tempfile.TemporaryDirectory() as td:
        ctx = RunContext.from_args("t-prop", str(Path(td) / "evidence"), "test", False)
        rel = ctx.evidence_root
        for chunk in chunks:
            rel = rel / chunk
        rel = rel.with_suffix(".json")
        out = ensure_evidence_path(ctx, rel)
        assert ctx.evidence_root.resolve() in out.parents


@pytest.mark.unit
@given(st.from_regex(r"[0-9][A-Za-z0-9_-]{0,15}", fullmatch=True))
def test_config_key_normalization_rejects_invalid_prefix(raw: str) -> None:
    with pytest.raises(ValueError):
        normalize_config_key(raw)
