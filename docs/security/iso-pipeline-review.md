# Experimental ISO Security Review

Reviewed: 2026-10-23
Scope: Fedora 44/Lorax technology-preview ISO workflow, its artifacts, and the UEFI/QEMU acceptance harness.

## Result

One workflow run built an ISO and passed the QEMU/OVMF console-marker test. This is evidence that a specific Fedora-based preview image can reach the project's marker in one virtual-machine configuration. It is not a security certification, a complete system acceptance test, or evidence of a production-ready distribution.

Evidence:
- Workflow: https://github.com/Deku-Foundation/deku-system/actions/runs/40003527897
- ISO SHA-256: a5d604ea3964a8a05c7a42b0c515586204e0f5c23a1a5e412fa585649eca5a58
- Temporary artifact ZIP SHA-256: 847bf4d9a0f38f080f0d619cf5c33b551b1ec91e758469ad263f63d8c773ffaa
- The artifact ZIP was configured to expire on 2026-10-30. Do not depend on it as a release location.

## Controls present

- The Fedora builder container is selected by image digest, and the profile/build-tool/test-harness repositories are checked out at full commit SHAs.
- Profile and Kickstart metadata are checked before the privileged image build.
- GitHub Actions is configured with contents: read and does not persist checkout credentials.
- The build workflow does not trigger on pull_request, has no configured secrets, and uploads logs/checksums with the experimental artifact.
- The privileged builder is designed to run on a fresh GitHub-hosted runner, not a persistent self-hosted runner.
- The ISO is verified against its SHA-256 before boot. QEMU runs in a disposable VM with paired OVMF code and a fresh writable UEFI variable store. The test waits for a specific serial marker, and logs are retained.
- The Kickstart locks the root account, enables SELinux enforcing mode and the firewall, and disables sshd.
- Rust profile validation and unit/contract tests reject several malformed inputs, package-set inconsistencies and unsafe workflow regressions.

## Material risks and limitations

### Privileged image builder

The workflow runs a privileged Docker container because Lorax/Anaconda's no-virt image creation needs elevated mount-related capabilities. A privileged container shares the host kernel and is not a strong security sandbox. Keep this job on disposable GitHub-hosted runners, never mount secrets or host home directories, and do not give it release-signing or publication credentials.

The workflow is triggered from dev changes and manual dispatch. Restrict manual execution to dev and ensure that the dev branch can only be updated through the required review process. At review time, GitHub reported both main and dev as unprotected for deku-system and deku-build. This is a governance gap: passing CI alone does not prevent an unreviewed writer from changing the privileged workflow.

### Mutable package sources

The image builder container and source commits are pinned, but Fedora release and updates repositories are not an immutable RPM snapshot. Package resolution can therefore change even if Git commits remain the same. The current manifest and in-image RPM list improve traceability but do not prove a reproducible build.

Before a release candidate, record exact target package NEVRAs and cryptographic digests, the repository metadata identity and all builder package versions. Compare two clean builds. A matching ISO hash is strong exact-byte evidence for one artifact, but a changing ISO hash alone does not identify the source of nondeterminism.

### Supply-chain evidence

The pipeline emits a checksum, a text build manifest and CI logs. It does not yet publish a complete standardized SBOM, signed SLSA provenance, a signed ISO, or signed package repository metadata. A checksum is not publisher authentication. Do not refer to this artifact as signed or independently reproducible.

### Acceptance coverage

The boot marker proves a narrow userspace milestone. It does not yet assert root-overlay integrity, failed systemd units, network readiness, package manifest consistency, graphical login, BIOS boot, hardware boot, Secure Boot, disk installation, upgrade, recovery or rollback. Non-fatal image-build warnings were recorded for a GRUB probe encountering the builder's overlay filesystem, a missing systemd-tmpfiles-clear unit during presetting, and unavailable D-Bus in the no-virt environment. Classify these warnings; do not silently discard them.

### Artifact availability

The test image was stored as a short-retention Actions artifact. The artifact ZIP is not a durable download or a release. A later workflow should generate a standard evidence bundle and only publish it after required tests, SBOM/provenance verification, signing and release review.

## Release-blocking actions

1. Protect main and dev with PR review, required status checks and no force-push/deletion; verify these settings directly in GitHub before relying on them.
2. Restrict the privileged image workflow's manual dispatch to dev and keep it off pull requests.
3. Pin the Rust validator revision, toolchain and Cargo.lock and invoke it with --locked.
4. Keep Fedora 44 as the canonical tested baseline until Fedora 45 final media and repositories are verified; test any profile change end to end.
5. Expand VM smoke tests to check systemd health, networking state, SSH disabled and the package manifest.
6. Capture exact resolved target packages; produce an SPDX or CycloneDX SBOM and provenance, then prove repeatability with independent builds.
7. Add disposable virtual-disk installation and recovery tests before advertising installer capability.
8. Add signing, key rotation/revocation procedures and publisher-authenticated release metadata before publishing a supported image.

## Upstream basis

- Fedora Lorax builds live media from Anaconda/Kickstart: https://github.com/weldr/lorax/blob/master/docs/livemedia-creator.rst
- NixOS image generation is a useful reference for declarative configuration and pinned input graphs, but is not a drop-in Fedora package manager: https://nixos.org/manual/nixos/stable/
- Void's xbps-src and void-mklive demonstrate build isolation and image-generation patterns, but are tied to the Void/XBPS system model: https://github.com/void-linux/void-mklive
- SLSA distinguishes basic provenance, signed hosted provenance and hardened build platforms: https://slsa.dev/spec/v1.0/levels
