#!/usr/bin/env python3
"""Validate the Rust integration-test layout keeps target size bounded."""

from __future__ import annotations

from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
CARGO_TOML = ROOT / "Cargo.toml"
TESTS_DIR = ROOT / "tests"
INTEGRATION_HARNESS = TESTS_DIR / "integration.rs"
WORKFLOW = ROOT.parents[2] / ".github" / "workflows" / "ci.yml"


def fail(message: str) -> None:
    print(f"rust test layout validation failed: {message}", file=sys.stderr)
    sys.exit(1)


def standalone_test_files() -> list[Path]:
    return sorted(
        path
        for path in TESTS_DIR.glob("s*.rs")
        if path.name != "integration.rs"
    )


def cargo_package_autotests_is_false(text: str) -> bool:
    package = re.search(r"(?ms)^\[package\]\s*(.*?)(?:^\[|\Z)", text)
    return bool(package and re.search(r"(?m)^\s*autotests\s*=\s*false\s*$", package.group(1)))


def cargo_has_single_integration_harness(text: str) -> bool:
    matches = re.findall(r"(?ms)^\[\[test\]\]\s*(.*?)(?=^\[|\Z)", text)
    found = [
        block
        for block in matches
        if re.search(r'(?m)^\s*name\s*=\s*"integration"\s*$', block)
        and re.search(r'(?m)^\s*path\s*=\s*"tests/integration\.rs"\s*$', block)
    ]
    return len(found) == 1


def main() -> None:
    cargo_text = CARGO_TOML.read_text(encoding="utf-8")

    if not cargo_package_autotests_is_false(cargo_text):
        fail("Cargo package must set autotests = false so tests/s*.rs are not built as 133 separate crates")

    if not cargo_has_single_integration_harness(cargo_text):
        fail("Cargo.toml must define exactly one integration test harness at tests/integration.rs")

    if not INTEGRATION_HARNESS.exists():
        fail("tests/integration.rs is required")

    harness = INTEGRATION_HARNESS.read_text(encoding="utf-8")
    files = standalone_test_files()
    missing = [path.name for path in files if f'include!("{path.name}");' not in harness]
    if missing:
        fail(f"integration harness does not include all test files: {', '.join(missing[:10])}")

    for path in files:
        text = path.read_text(encoding="utf-8")
        if re.search(r"(?m)^\s*mod\s+common\s*;", text):
            fail(f"{path.relative_to(ROOT)} must use crate::common instead of declaring a per-test common module")

    workflow = WORKFLOW.read_text(encoding="utf-8")
    forbidden_workflow_fragments = [
        "git clean -fd -e source/opc/backend/target/",
        "actions/cache",
    ]
    for fragment in forbidden_workflow_fragments:
        if fragment in workflow:
            fail(f"CI workflow must not preserve or cache backend target: {fragment}")


if __name__ == "__main__":
    main()
