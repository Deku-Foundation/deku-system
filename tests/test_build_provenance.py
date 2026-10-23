import hashlib
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parents[1]))
from scripts.write_provenance import PREDICATE_TYPE, build_statement


class BuildProvenanceTests(unittest.TestCase):
    def test_statement_binds_iso_digest_and_records_failed_boot(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            iso = root / "preview.iso"
            manifest = root / "build-manifest.txt"
            boot = root / "boot-result.json"
            iso.write_bytes(b"sample-iso-bytes")
            manifest.write_text("builder_image=fedora:44@sha256:" + "a" * 64 + "\n", encoding="utf-8")
            boot.write_text(json.dumps({
                "status": "failed",
                "marker": "DEKU_BOOT_OK:minimal-x86_64",
                "marker_found": False,
                "failure": "guest acceptance check failed: selinux-not-enforcing",
                "actual_sha256": hashlib.sha256(b"sample-iso-bytes").hexdigest(),
            }), encoding="utf-8")
            statement = build_statement(iso, manifest, boot, {
                "DEKU_SOURCE_SHA": "b" * 40,
                "DEKU_PROFILE_SHA": "c" * 40,
                "DEKU_VALIDATOR_SHA": "d" * 40,
                "DEKU_TEST_HARNESS_SHA": "e" * 40,
                "DEKU_WORKFLOW_RUN_ID": "123",
                "DEKU_WORKFLOW_URL": "https://example.invalid/run/123",
                "DEKU_BUILDER_IMAGE": "fedora:44@sha256:" + "a" * 64,
            })
            digest = hashlib.sha256(b"sample-iso-bytes").hexdigest()
            self.assertEqual(statement["_type"], "https://in-toto.io/Statement/v1")
            self.assertEqual(statement["predicateType"], PREDICATE_TYPE)
            self.assertEqual(statement["subject"][0]["digest"]["sha256"], digest)
            self.assertEqual(statement["predicate"]["bootAcceptance"]["status"], "failed")
            self.assertEqual(statement["predicate"]["bootAcceptance"]["failure"], "guest acceptance check failed: selinux-not-enforcing")
            self.assertEqual(len(statement["predicate"]["build"]["materials"]), 5)
            self.assertIn("unsigned", statement["predicate"]["statementStatus"])

    def test_missing_iso_fails_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = root / "manifest.txt"
            manifest.write_text("fixture\n", encoding="utf-8")
            with self.assertRaises(FileNotFoundError):
                build_statement(root / "missing.iso", manifest, root / "missing.json", {})


if __name__ == "__main__":
    unittest.main()
