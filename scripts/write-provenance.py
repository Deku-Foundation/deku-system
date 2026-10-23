#!/usr/bin/env python3
"""Create an unsigned in-toto Statement for a Deku experimental ISO build.

This statement improves traceability; it is not signed, does not authenticate its
publisher, and is not a claim of SLSA compliance or reproducible output.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from pathlib import Path
from typing import Any


PREDICATE_TYPE = "https://deku-foundation.github.io/attestations/iso-build/v1"


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return {}
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"invalid JSON evidence at {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise ValueError(f"evidence at {path} must be a JSON object")
    return value


def build_statement(iso: Path, manifest: Path, boot_result: Path, env: dict[str, str]) -> dict[str, Any]:
    if not iso.is_file() or iso.stat().st_size == 0:
        raise FileNotFoundError(f"ISO is missing or empty: {iso}")
    if not manifest.is_file():
        raise FileNotFoundError(f"build manifest is missing: {manifest}")

    boot = load_json(boot_result)
    manifest_text = manifest.read_text(encoding="utf-8")
    source_sha = env.get("DEKU_SOURCE_SHA", "unknown")
    profile_sha = env.get("DEKU_PROFILE_SHA", "unknown")
    validator_sha = env.get("DEKU_VALIDATOR_SHA", "unknown")
    harness_sha = env.get("DEKU_TEST_HARNESS_SHA", "unknown")
    run_id = env.get("DEKU_WORKFLOW_RUN_ID", "unknown")
    workflow_url = env.get("DEKU_WORKFLOW_URL", "")
    builder_image = env.get("DEKU_BUILDER_IMAGE", "unknown")

    materials = [
        {"name": "Deku system source", "uri": "https://github.com/Deku-Foundation/deku-system", "revision": source_sha},
        {"name": "Deku profile source", "uri": "https://github.com/Deku-Foundation/deku-profiles", "revision": profile_sha},
        {"name": "Deku Rust validator", "uri": "https://github.com/Deku-Foundation/deku-build", "revision": validator_sha},
        {"name": "Deku boot harness", "uri": "https://github.com/Deku-Foundation/deku-tests", "revision": harness_sha},
        {"name": "Builder container", "uri": builder_image, "digest": builder_image.split("@sha256:", 1)[1] if "@sha256:" in builder_image else None},
    ]
    predicate = {
        "schemaVersion": 1,
        "statementStatus": "unsigned-traceability-only",
        "build": {
            "workflow": workflow_url,
            "runId": run_id,
            "sourceCommit": source_sha,
            "builderImage": builder_image,
            "materials": materials,
        },
        "artifact": {
            "name": iso.name,
            "sizeBytes": iso.stat().st_size,
            "sha256": sha256_file(iso),
        },
        "buildManifest": {
            "name": manifest.name,
            "sha256": sha256_file(manifest),
            "declaredBuilderImage": next(
                (line.split("=", 1)[1] for line in manifest_text.splitlines() if line.startswith("builder_image=")),
                "unknown",
            ),
        },
        "bootAcceptance": {
            "status": boot.get("status", "not-run"),
            "marker": boot.get("marker"),
            "markerFound": boot.get("marker_found", False),
            "failure": boot.get("failure"),
            "elapsedSeconds": boot.get("elapsed_seconds"),
            "isoSha256VerifiedByHarness": boot.get("actual_sha256"),
        },
        "limitations": [
            "This statement is unsigned and does not authenticate the publisher.",
            "Fedora release and updates repositories are mutable package inputs.",
            "A successful console boot is not proof of reproducibility, hardware support, or production readiness.",
        ],
    }
    return {
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{"name": iso.name, "digest": {"sha256": sha256_file(iso)}}],
        "predicateType": PREDICATE_TYPE,
        "predicate": predicate,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iso", required=True, type=Path)
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--boot-result", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    if not args.iso.is_file() or not args.manifest.is_file():
        print("SKIP: ISO or build manifest missing; no artifact statement emitted", file=sys.stderr)
        return 0
    try:
        statement = build_statement(args.iso, args.manifest, args.boot_result, dict(os.environ))
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(statement, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")
    except (OSError, ValueError) as exc:
        print(f"ERROR: cannot create build statement: {exc}", file=sys.stderr)
        return 2
    print(f"Wrote unsigned in-toto statement for {args.iso.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
