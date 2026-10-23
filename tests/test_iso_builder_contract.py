import re
import unittest
from pathlib import Path

ROOT = Path(__file__).parents[1]
KICKSTART = ROOT / "kickstart" / "deku-minimal-live.ks"
BUILDER = ROOT / "scripts" / "build-preview-iso.sh"
WORKFLOW = ROOT / ".github" / "workflows" / "build-preview-iso.yml"


class MinimalLiveIsoContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.kickstart = KICKSTART.read_text(encoding="utf-8")
        cls.builder = BUILDER.read_text(encoding="utf-8")
        cls.workflow = WORKFLOW.read_text(encoding="utf-8")
        match = re.search(r"(?ms)^%packages\s*\n(.*?)^%end\s*$", cls.kickstart)
        if match is None:
            raise AssertionError("Kickstart %packages section is missing")
        cls.package_entries = [
            line.strip().split()[0]
            for line in match.group(1).splitlines()
            if line.strip() and not line.lstrip().startswith(("#", "@", "-"))
        ]
        cls.packages = set(cls.package_entries)

    def test_iso_contains_bios_and_uefi_grub_inputs(self):
        required = {
            "grub2-pc",
            "grub2-pc-modules",
            "grub2-tools",
            "grub2-tools-extra",
            "grub2-efi-x64",
            "grub2-efi-x64-cdboot",
            "grub2-efi-x64-modules",
            "efibootmgr",
        }
        self.assertTrue(required <= self.packages, f"missing packages: {sorted(required - self.packages)}")

    def test_package_declarations_are_unique(self):
        self.assertEqual(
            len(self.package_entries),
            len(set(self.package_entries)),
            "duplicate package declarations make profile intent ambiguous",
        )

    def test_live_runtime_dependencies_are_explicit(self):
        required = {"kernel", "kernel-modules", "dracut-network", "dracut-config-generic", "dracut-live", "NetworkManager"}
        self.assertTrue(required <= self.packages, f"missing packages: {sorted(required - self.packages)}")

    def test_boot_marker_unit_has_no_target_ordering_cycle(self):
        self.assertIn("WantedBy=multi-user.target", self.kickstart)
        self.assertIn("After=systemd-user-sessions.service", self.kickstart)
        self.assertNotIn("After=multi-user.target", self.kickstart)
        self.assertIn("DEKU_BOOT_OK:minimal-x86_64", self.kickstart)

    def test_builder_fails_closed_on_missing_boot_packages(self):
        for package in ("grub2-pc", "grub2-pc-modules", "grub2-tools", "grub2-efi-x64", "efibootmgr"):
            self.assertIn(package, self.builder)
        self.assertIn("missing_live", self.builder)

    def test_builder_manifest_fields_are_separate_lines(self):
        lines = [line for line in self.builder.splitlines() if "builder_image=" in line]
        self.assertEqual(len(lines), 1)
        self.assertNotIn('echo "builder_fedora_release=', lines[0])
        self.assertIn('echo "builder_fedora_release=', self.builder)

    def test_image_build_is_not_triggered_by_pull_requests(self):
        self.assertIn("workflow_dispatch:", self.workflow)
        self.assertIn("branches: [dev]", self.workflow)
        self.assertNotIn("pull_request:", self.workflow)

    def test_privileged_builder_has_no_checkout_credentials_or_host_network(self):
        self.assertEqual(self.workflow.count("persist-credentials: false"), 3)
        self.assertNotIn("--network=host", self.workflow)
        self.assertIn("permissions:\n  contents: read", self.workflow)

    def test_builder_and_source_revisions_are_pinned(self):
        self.assertIn("fedora:44@sha256:43b29f65a41eb9c35e1cd5323e3bdf3b655c2357a9f4f1ff2f9c2798e5045d80", self.workflow)
        self.assertIn("ref: 0b000f0ebfaa021fa0153cb306db61866024b70d", self.workflow)
        self.assertIn("ref: 618ac8a9fff706bfa446f482ab23d749f7511a6d", self.workflow)


if __name__ == "__main__":
    unittest.main()
