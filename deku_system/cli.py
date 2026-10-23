"""Command-line validator for the Deku system manifest prototype."""
from __future__ import annotations

import argparse
import sys

from .manifest import ManifestValidationError, load_and_validate


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate a Deku system composition manifest without building an image"
    )
    parser.add_argument("manifest", help="path to a JSON system manifest")
    args = parser.parse_args()

    try:
        manifest = load_and_validate(args.manifest)
    except ManifestValidationError as exc:
        print(f"invalid system manifest: {exc}", file=sys.stderr)
        return 2

    target = manifest["target"]
    print("manifest valid (metadata-only; no image was built)")
    print(f"release: {manifest['release_id']}")
    print(f"target: {target['architecture']} / {target['profile']}")
    print(f"packages: {len(manifest['packages'])}")
    print("image assembly, signing and publication are not implemented")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
