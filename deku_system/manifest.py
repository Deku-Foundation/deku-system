"""Strict validation for non-executing Deku system composition manifests."""
from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

MAX_MANIFEST_BYTES = 2 * 1024 * 1024
MAX_PACKAGES = 2_048

RELEASE_ID = re.compile(r"^[a-z0-9][a-z0-9._+-]{2,63}$")
PACKAGE_NAME = re.compile(r"^[a-z0-9][a-z0-9+._-]{0,127}$")
VERSION = re.compile(r"^[A-Za-z0-9][A-Za-z0-9.+:~_-]{0,127}$")
REVISION = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
ARCHITECTURES = {"x86_64", "aarch64"}
PACKAGE_ARCHITECTURES = ARCHITECTURES | {"noarch"}
PROFILES = {"kde-plasma"}


class ManifestValidationError(ValueError):
    """Raised when a system composition manifest is invalid."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ManifestValidationError(message)


def _unique_object_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    """Reject duplicate JSON keys to avoid ambiguous or overwritten build inputs."""
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ManifestValidationError(f"duplicate JSON object key: {key}")
        result[key] = value
    return result


def _digest(value: Any, label: str) -> None:
    require(
        isinstance(value, str) and SHA256.fullmatch(value) is not None,
        f"{label} must be a canonical lowercase SHA-256 digest",
    )


def _revision(value: Any, label: str) -> None:
    require(
        isinstance(value, str) and REVISION.fullmatch(value) is not None,
        f"{label} must be a full 40-character lowercase immutable source revision",
    )


def validate_manifest(manifest: Any) -> dict[str, Any]:
    """Validate manifest metadata; do not fetch inputs, execute commands, or build an image."""
    require(isinstance(manifest, dict), "manifest must be a JSON object")
    required_top = {
        "schema_version",
        "release_id",
        "target",
        "inputs",
        "packages",
    }
    require(
        set(manifest) == required_top,
        "manifest must contain exactly schema_version, release_id, target, inputs, and packages",
    )
    require(
        type(manifest["schema_version"]) is int and manifest["schema_version"] == 1,
        "unsupported schema_version",
    )
    require(
        isinstance(manifest["release_id"], str)
        and RELEASE_ID.fullmatch(manifest["release_id"]) is not None,
        "release_id must be a lowercase release identifier",
    )

    target = manifest["target"]
    require(
        isinstance(target, dict) and set(target) == {"architecture", "profile"},
        "target must contain architecture and profile only",
    )
    require(
        isinstance(target["architecture"], str) and target["architecture"] in ARCHITECTURES,
        "unsupported target architecture",
    )
    require(
        isinstance(target["profile"], str) and target["profile"] in PROFILES,
        "unsupported system profile; schema v1 supports kde-plasma only",
    )

    inputs = manifest["inputs"]
    required_inputs = {
        "base_revision",
        "base_sha256",
        "kernel_revision",
        "kernel_sha256",
        "catalog_sha256",
        "toolchain_id",
        "configuration_sha256",
    }
    require(
        isinstance(inputs, dict) and set(inputs) == required_inputs,
        "inputs must contain the seven documented source, digest, and toolchain fields only",
    )
    _revision(inputs["base_revision"], "inputs.base_revision")
    _digest(inputs["base_sha256"], "inputs.base_sha256")
    _revision(inputs["kernel_revision"], "inputs.kernel_revision")
    _digest(inputs["kernel_sha256"], "inputs.kernel_sha256")
    _digest(inputs["catalog_sha256"], "inputs.catalog_sha256")
    _digest(inputs["configuration_sha256"], "inputs.configuration_sha256")
    toolchain_id = inputs["toolchain_id"]
    require(
        isinstance(toolchain_id, str)
        and 1 <= len(toolchain_id) <= 128
        and toolchain_id == toolchain_id.strip()
        and all(char.isascii() and (char.isalnum() or char in "._+-") for char in toolchain_id),
        "inputs.toolchain_id must be a non-empty canonical toolchain identifier",
    )

    packages = manifest["packages"]
    require(
        isinstance(packages, list) and 0 < len(packages) <= MAX_PACKAGES,
        f"packages must be a non-empty array with at most {MAX_PACKAGES} entries",
    )
    seen: set[str] = set()
    expected_arch = target["architecture"]
    for index, package in enumerate(packages):
        prefix = f"packages[{index}]"
        require(
            isinstance(package, dict)
            and set(package) == {"name", "version", "architecture", "sha256"},
            f"{prefix} must contain name, version, architecture, and sha256 only",
        )
        name = package["name"]
        require(
            isinstance(name, str) and PACKAGE_NAME.fullmatch(name) is not None,
            f"{prefix}.name is not a valid package identifier",
        )
        require(name not in seen, f"duplicate package name: {name}")
        seen.add(name)
        require(
            isinstance(package["version"], str) and VERSION.fullmatch(package["version"]) is not None,
            f"{prefix}.version is not a canonical version token",
        )
        architecture = package["architecture"]
        require(
            isinstance(architecture, str) and architecture in PACKAGE_ARCHITECTURES,
            f"{prefix}.architecture is unsupported",
        )
        require(
            architecture == expected_arch or architecture == "noarch",
            f"{prefix}.architecture {architecture!r} does not match target {expected_arch!r}",
        )
        _digest(package["sha256"], f"{prefix}.sha256")

    return manifest


def load_and_validate(path: str | Path) -> dict[str, Any]:
    """Load bounded UTF-8 JSON and reject ambiguous keys before validation."""
    try:
        with Path(path).open("rb") as stream:
            raw = stream.read(MAX_MANIFEST_BYTES + 1)
    except OSError as exc:
        raise ManifestValidationError(f"cannot read system manifest: {exc}") from exc

    if len(raw) > MAX_MANIFEST_BYTES:
        raise ManifestValidationError(
            f"manifest exceeds the {MAX_MANIFEST_BYTES}-byte input limit"
        )

    try:
        text = raw.decode("utf-8")
        manifest = json.loads(text, object_pairs_hook=_unique_object_pairs)
    except ManifestValidationError:
        raise
    except (UnicodeDecodeError, json.JSONDecodeError, RecursionError, ValueError) as exc:
        raise ManifestValidationError(f"cannot read valid UTF-8 JSON manifest: {exc}") from exc

    return validate_manifest(manifest)
