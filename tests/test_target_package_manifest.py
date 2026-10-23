import hashlib
import tempfile
import unittest
from pathlib import Path

from scripts.extract_target_package_manifest import ManifestError, extract_nevras, write_manifest


class TargetPackageManifestTests(unittest.TestCase):
    def test_extracts_one_sorted_nevra_block(self):
        serial = (
            "firmware boot\n"
            "DEKU_PACKAGE_MANIFEST_BEGIN\n"
            "bash-5.2.26-3.fc44.x86_64\n"
            "glibc-2.39-20.fc44.x86_64\n"
            "systemd-256.7-1.fc44.x86_64\n"
            "DEKU_PACKAGE_MANIFEST_END\n"
            "DEKU_BOOT_OK:minimal-x86_64\n"
        )
        self.assertEqual(
            extract_nevras(serial),
            [
                "bash-5.2.26-3.fc44.x86_64",
                "glibc-2.39-20.fc44.x86_64",
                "systemd-256.7-1.fc44.x86_64",
            ],
        )

    def test_requires_exactly_one_marker_pair(self):
        for serial in ("no inventory", "DEKU_PACKAGE_MANIFEST_BEGIN\na-1-1.x86_64", "DEKU_PACKAGE_MANIFEST_BEGIN\nx-1-1.x86_64\nDEKU_PACKAGE_MANIFEST_END\nDEKU_PACKAGE_MANIFEST_END"):
            with self.subTest(serial=serial):
                with self.assertRaises(ManifestError):
                    extract_nevras(serial)

    def test_rejects_unsorted_or_duplicate_entries(self):
        with self.assertRaisesRegex(ManifestError, "sorted"):
            extract_nevras("DEKU_PACKAGE_MANIFEST_BEGIN\nz-1-1.x86_64\na-1-1.x86_64\nDEKU_PACKAGE_MANIFEST_END")
        with self.assertRaisesRegex(ManifestError, "duplicate"):
            extract_nevras("DEKU_PACKAGE_MANIFEST_BEGIN\na-1-1.x86_64\na-1-1.x86_64\nDEKU_PACKAGE_MANIFEST_END")

    def test_rejects_paths_or_non_nevra_data(self):
        with self.assertRaisesRegex(ManifestError, "invalid RPM NEVRA"):
            extract_nevras("DEKU_PACKAGE_MANIFEST_BEGIN\n../../escape.rpm\nDEKU_PACKAGE_MANIFEST_END")

    def test_writes_deterministic_manifest_digest_and_build_evidence(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            log = base / "serial.log"
            out = base / "target-package-manifest.txt"
            build = base / "build-manifest.txt"
            log.write_text(
                "DEKU_PACKAGE_MANIFEST_BEGIN\na-1-1.x86_64\nb-2-3.noarch\nDEKU_PACKAGE_MANIFEST_END\n",
                encoding="utf-8",
            )
            build.write_text("status=experimental\n", encoding="utf-8")
            result = write_manifest(log, out, build)
            expected = hashlib.sha256(b"a-1-1.x86_64\nb-2-3.noarch\n").hexdigest()
            self.assertEqual(result["entry_count"], 2)
            self.assertEqual(result["sha256"], expected)
            self.assertEqual(out.read_bytes(), b"a-1-1.x86_64\nb-2-3.noarch\n")
            self.assertIn(expected, out.with_name(out.name + ".sha256").read_text())
            self.assertIn(f"target_package_manifest_sha256={expected}", build.read_text())


if __name__ == "__main__":
    unittest.main()
