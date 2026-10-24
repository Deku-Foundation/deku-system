import unittest
from pathlib import Path

ROOT = Path(__file__).parents[1]


class InfrastructureFirstPolicyTests(unittest.TestCase):
    def test_no_active_fedora_iso_pipeline(self):
        self.assertFalse((ROOT / ".github/workflows/build-preview-iso.yml").exists())
        self.assertFalse((ROOT / "scripts/build-preview-iso.sh").exists())
        self.assertFalse((ROOT / "kickstart/deku-minimal-live.ks").exists())

    def test_iso_page_records_pause_not_success(self):
        doc = (ROOT / "docs/architecture/first-iso-pipeline.md").read_text(encoding="utf-8")
        self.assertIn("paused", doc.lower())
        self.assertIn("did **not** pass", doc)
        self.assertIn("No ISO workflow should be scheduled", doc)

    def test_active_build_doc_prioritizes_package_infrastructure(self):
        doc = (ROOT / "docs/build-pipeline.md").read_text(encoding="utf-8")
        self.assertIn("End-to-end package flow", doc)
        self.assertIn("No active Fedora ISO workflow", doc)
        self.assertNotIn("Build experimental ISO workflow", doc)


if __name__ == "__main__":
    unittest.main()
