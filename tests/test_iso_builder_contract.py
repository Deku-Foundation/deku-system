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

    def test_boot_acceptance_script_checks_runtime_contract(self):
        self.assertIn("deku-boot-acceptance.sh", self.kickstart)
        self.assertIn("After=NetworkManager.service systemd-user-sessions.service", self.kickstart)
        self.assertIn("DEKU_BOOT_FAIL:", self.kickstart)
        self.assertIn("check package-manifest test -s /etc/deku-package-manifest.txt", self.kickstart)
        self.assertIn("DEKU_PACKAGE_MANIFEST_BEGIN", self.kickstart)
        self.assertIn("DEKU_PACKAGE_MANIFEST_END", self.kickstart)
        self.assertIn("LC_ALL=C sort", self.kickstart)
        self.assertIn("extract_target_package_manifest.py", self.workflow)
        self.assertIn("systemctl is-active --quiet NetworkManager", self.kickstart)
        self.assertIn("systemctl is-enabled --quiet sshd", self.kickstart)
        self.assertIn("/sys/fs/selinux/enforce", self.kickstart)

    def test_boot_marker_unit_has_no_target_ordering_cycle(self):
        self.assertIn("WantedBy=multi-user.target", self.kickstart)
        self.assertIn("systemd-user-sessions.service", self.kickstart)
        self.assertNotIn("After=multi-user.target", self.kickstart)
        self.assertIn("DEKU_BOOT_OK:minimal-x86_64", self.kickstart)
        self.assertIn("scripts/write_provenance.py", self.workflow)
        self.assertIn("--timeout 420", self.workflow)

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
        self.assertEqual(self.workflow.count("persist-credentials: false"), 4)
        self.assertNotIn("--network=host", self.workflow)
        self.assertIn("permissions:", self.workflow)
        self.assertIn("  contents: read", self.workflow)
        self.assertIn("  contents: read", self.workflow)

    def test_artifact_ownership_is_fixed_before_host_side_boot_evidence(self):
        normalize = self.workflow.index("Prepare generated artifact permissions for host-side checks")
        boot = self.workflow.index("Boot ISO in disposable UEFI QEMU")
        final_cleanup = self.workflow.index("Normalize artifact permissions")
        self.assertLess(normalize, boot, "root-owned container output must be writable before the harness starts")
        self.assertLess(boot, final_cleanup, "final artifact normalization must remain after the boot test")
        self.assertIn('sudo chown -R "$(id -u):$(id -g)" deku-system/artifacts', self.workflow)

    def test_rust_profile_validator_runs_before_privileged_image_build(self):
        validate = self.workflow.index("Validate profile and Kickstart in Rust before privileged build")
        build = self.workflow.index("Build in disposable Fedora 44 container")
        self.assertLess(validate, build)
        self.assertIn("repository: Deku-Foundation/deku-build", self.workflow)
        self.assertIn("validate-profile", self.workflow)

    def test_uefi_boot_harness_receives_code_and_writable_vars(self):
        self.assertIn(' --firmware "$OVMF_CODE" \\', self.workflow)
        self.assertIn(' --firmware-vars "$OVMF_VARS" \\', self.workflow)
        self.assertIn("OVMF_VARS_4M.fd", self.workflow)

    def test_fedora44_repo_urls_and_security_defaults_are_explicit(self):
        self.assertIn('url --url="https://download.fedoraproject.org/pub/fedora/linux/releases/44/Everything/x86_64/os/"', self.kickstart)
        self.assertIn('repo --name="updates" --baseurl="https://download.fedoraproject.org/pub/fedora/linux/updates/44/Everything/x86_64/"', self.kickstart)
        self.assertNotIn("/45/", self.kickstart)
        self.assertIn("rootpw --lock", self.kickstart)
        self.assertIn("selinux --enforcing", self.kickstart)
        self.assertIn("firewall --enabled", self.kickstart)
        self.assertRegex(self.kickstart, r'services --disabled="[^"]*sshd')

    def test_profile_preflight_uses_pinned_locked_rust_dependencies(self):
        self.assertIn("ref: 20bd35be9db6b99cf26bbfb6482914ec333a8ac5", self.workflow)
        self.assertIn("uses: dtolnay/rust-toolchain@b725eda525362e7cf34af9a7243cac1e3f01a7ec", self.workflow)
        self.assertIn("toolchain: 1.99.1", self.workflow)
        self.assertIn("cargo run --locked -- validate-profile", self.workflow)

    def test_in_image_package_manifest_has_single_newline_escape(self):
        self.assertIn(r"%{ARCH}\n' | LC_ALL=C sort > /etc/deku-package-manifest.txt", self.kickstart)
        self.assertNotIn(r"%{ARCH}\\n' | LC_ALL=C sort > /etc/deku-package-manifest.txt", self.kickstart)

    def test_privileged_build_runs_only_from_dev_even_on_manual_dispatch(self):
        self.assertIn("workflow_dispatch:", self.workflow)
        self.assertIn("if: github.ref == 'refs/heads/dev'", self.workflow)

    def test_builder_and_source_revisions_are_pinned(self):
        self.assertIn("fedora:44@sha256:43b29f65a41eb9c35e1cd5323e3bdf3b655c2357a9f4f1ff2f9c2798e5045d80", self.workflow)
        refs = [line.strip()[5:] for line in self.workflow.splitlines() if line.strip().startswith("ref: ")]
        self.assertEqual(len(refs), 3, "profile, Rust validator and test harness must use explicit commit refs")
        self.assertTrue(all(re.fullmatch(r"[0-9a-f]{40}", ref) for ref in refs), f"mutable or invalid checkout ref found: {refs}")

if __name__ == "__main__":
    unittest.main()
