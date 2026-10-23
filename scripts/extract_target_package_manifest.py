#!/usr/bin/env python3
"""Extract a validated RPM NEVRA inventory emitted by the booted live image.

This is a traceability manifest, not a complete SBOM: it contains installed
NEVRA identifiers only and does not claim package digests, licenses, or
repository signature evidence.
"""
from __future__ import annotations

import argparse
import hashlib
import re
import sys
from pathlib import Path

BEGIN = "DEKU_PACKAGE_MANIFEST_BEGIN"
END = "DEKU_PACKAGE_MANIFEST_END"
NEVRA = re.compile(r"^[A-Za-z0-9][A-Za-z0-9+_.:-]*\.[A-Za-z0-9_]+$")


class ManifestError(ValueError):
    """Raised when the serial log does not contain one sound package inventory."""


def extract_nevras(serial_log: str) -> list[str]:
    begin_count = serial_log.count(BEGIN)
    end_count = serial_log.count(END)
    if begin_count != 1 or end_count != 1:
        raise ManifestError("serial log must contain exactly one package-manifest begin/end pair")
    before, rest = serial_log.split(BEGIN, 1)
    inventory, after = rest.split(END, 1)
    if after.find(BEGIN) >= 0 or before.find(END) >= 0:
        raise ManifestError("package-manifest markers are out of order")
    entries = [line.strip() for line in inventory.splitlines() if line.strip()]
    if not entries:
        raise ManifestError("package manifest is empty")
    if any(not NEVRA.fullmatch(line) for line in entries):
        invalid = next(line for line in entries if not NEVRA.fullmatch(line))
        raise ManifestError(f"invalid RPM NEVRA entry: {invalid!r}")
    if entries != sorted(entries):
        raise ManifestError("package manifest entries must be sorted")
    if len(entries) != len(set(entries)):
        raise ManifestError("package manifest contains duplicate NEVRA entries")
    return entries


def write_manifest(serial_path: Path, output_path: Path, build_manifest_path: Path | None = None) -> dict[str, object]:
    text = serial_path.read_text(encoding="utf-8", errors="replace")
    entries = extract_nevras(text)
    payload = ("\n".join(entries) + "\n").encode("utf-8")
    digest = hashlib.sha256(payload).hexdigest()

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_bytes(payload)
    checksum_path = output_path.with_name(output_path.name + ".sha256")
    checksum_path.write_text(f"{digest}  {output_path.name}\n", encoding="utf-8")

    evidence: dict[str, object] = {
        "entry_count": len(entries),
        "sha256": digest,
        "manifest_path": str(output_path),
        "checksum_path": str(checksum_path),
    }
    if build_manifest_path is not None:
        with build_manifest_path.open("a", encoding="utf-8") as stream:
            stream.write(f"target_package_manifest_count={len(entries)}\n")
            stream.write(f"target_package_manifest_sha256={digest}\n")
    return evidence


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--serial-log", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--build-manifest", type=Path)
    args = parser.parse_args()
    try:
        evidence = write_manifest(args.serial_log, args.output, args.build_manifest)
    except (OSError, UnicodeError, ManifestError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2
    print(f"Validated {evidence['entry_count']} target RPM NEVRA entries")
    print(f"SHA-256: {evidence['sha256']}")
    print("Note: this inventory is not a complete SBOM and does not establish package publisher authenticity.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
