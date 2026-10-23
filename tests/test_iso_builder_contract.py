import re
import unittest
from pathlib import Path

ROOT = Path(__file__).parents[1]
KICKSTART = ROOT / "kickstart" / "deku-minimal-live.ks"
BUILDER = ROOT / "scripts" / "build-preview-iso.sh"


class MinimalLiveIsoContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.kickstart = KICKSTART.read_text(encoding="utf-8")
        cls.builder = BUILDER.read_text(encoding="utf-8")
        match = re.search(r"(?ms)^%packages\s*\n(.*?)^%end\s*$", cls.kickstart)
        if match is None:
            raise AssertionError("Kickstart %packages section is missing")
        cls.packages = {
            line.strip().split()[0]
            for line in match.group(1).splitlines()
            if line.strip() and not line.lstrip().startswith(("#", "@", "-"))
        }

    def test_iso_contains_bios_and_uefi_grub_inputs(self):
        required = {
            "grub2-pc",
            "grub2-pc-modules",
            "grub2-tools",
            "grub2-efi-x64",
            "efibootmgr",
        }
        self.assertTrue(required <= self.packages, f"missing packages: {sorted(required - self.packages)}")

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

    def test_image_build_is_not_triggered_by_pull_requests(self):
        workflow = (ROOT / ".github" / "workflows" / "build-preview-iso.yml").read_text(encoding="utf-8")
        self.assertIn("workflow_dispatch:", workflow)
        self.assertIn("branches: [dev]", workflow)
        self.assertNotIn("pull_request:", workflow)

    def test_no_host_network_mode_is_used(self):
        workflow = (ROOT / ".github" / "workflows" / "build-preview-iso.yml").read_text(encoding="utf-8")
        self.assertNotIn("--network=host", workflow)


if __name__ == "__main__":
    unittest.main()
