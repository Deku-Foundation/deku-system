from copy import deepcopy
import json
import tempfile
import unittest
from pathlib import Path

from deku_system.manifest import (
    MAX_MANIFEST_BYTES,
    ManifestValidationError,
    load_and_validate,
    validate_manifest,
)

EXAMPLE = Path(__file__).parents[1] / "examples" / "system-manifest.json"


class SystemManifestTests(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads(EXAMPLE.read_text(encoding="utf-8"))

    def test_example_is_valid(self):
        self.assertEqual(validate_manifest(self.manifest), self.manifest)

    def test_load_example(self):
        self.assertEqual(load_and_validate(EXAMPLE)["target"]["profile"], "kde-plasma")

    def test_unknown_top_level_fields_are_rejected(self):
        manifest = deepcopy(self.manifest)
        manifest["execute"] = "shell"
        with self.assertRaisesRegex(ManifestValidationError, "exactly"):
            validate_manifest(manifest)

    def test_boolean_schema_version_is_rejected(self):
        manifest = deepcopy(self.manifest)
        manifest["schema_version"] = True
        with self.assertRaisesRegex(ManifestValidationError, "schema_version"):
            validate_manifest(manifest)

    def test_unknown_profile_is_rejected(self):
        manifest = deepcopy(self.manifest)
        manifest["target"]["profile"] = "gnome"
        with self.assertRaisesRegex(ManifestValidationError, "kde-plasma only"):
            validate_manifest(manifest)

    def test_invalid_source_revision_is_rejected(self):
        manifest = deepcopy(self.manifest)
        manifest["inputs"]["kernel_revision"] = "main"
        with self.assertRaisesRegex(ManifestValidationError, "immutable source revision"):
            validate_manifest(manifest)

    def test_noncanonical_digest_is_rejected(self):
        manifest = deepcopy(self.manifest)
        manifest["inputs"]["catalog_sha256"] = "A" * 64
        with self.assertRaisesRegex(ManifestValidationError, "canonical lowercase SHA-256"):
            validate_manifest(manifest)

    def test_duplicate_package_name_is_rejected(self):
        manifest = deepcopy(self.manifest)
        manifest["packages"].append(deepcopy(manifest["packages"][0]))
        with self.assertRaisesRegex(ManifestValidationError, "duplicate package"):
            validate_manifest(manifest)

    def test_package_architecture_must_match_target(self):
        manifest = deepcopy(self.manifest)
        manifest["packages"][0]["architecture"] = "aarch64"
        with self.assertRaisesRegex(ManifestValidationError, "does not match target"):
            validate_manifest(manifest)

    def test_noarch_package_is_allowed(self):
        manifest = deepcopy(self.manifest)
        manifest["packages"][0]["architecture"] = "noarch"
        self.assertEqual(validate_manifest(manifest), manifest)

    def test_unsafe_package_version_is_rejected(self):
        manifest = deepcopy(self.manifest)
        manifest["packages"][0]["version"] = "../escape"
        with self.assertRaisesRegex(ManifestValidationError, "canonical version token"):
            validate_manifest(manifest)

    def test_unknown_package_fields_are_rejected(self):
        manifest = deepcopy(self.manifest)
        manifest["packages"][0]["run"] = "untrusted-command"
        with self.assertRaisesRegex(ManifestValidationError, "only"):
            validate_manifest(manifest)

    def test_duplicate_json_keys_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "duplicate.json"
            path.write_text('{"schema_version":1,"schema_version":1}', encoding="utf-8")
            with self.assertRaisesRegex(ManifestValidationError, "duplicate JSON object key"):
                load_and_validate(path)

    def test_oversized_json_is_rejected_before_deserialization(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "large.json"
            path.write_bytes(b" " * (MAX_MANIFEST_BYTES + 1))
            with self.assertRaisesRegex(ManifestValidationError, "input limit"):
                load_and_validate(path)

    def test_invalid_utf8_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "invalid.json"
            path.write_bytes(b'{"invalid":"' + bytes([0xFF]) + b'"}')
            with self.assertRaisesRegex(ManifestValidationError, "valid UTF-8 JSON"):
                load_and_validate(path)

    def test_excessive_json_nesting_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "deep.json"
            path.write_bytes(b"[" * 10_000 + b"0" + b"]" * 10_000)
            with self.assertRaisesRegex(ManifestValidationError, "valid UTF-8 JSON"):
                load_and_validate(path)


if __name__ == "__main__":
    unittest.main()
