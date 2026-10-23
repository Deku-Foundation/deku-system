# First ISO Pipeline

Status: **the first experimental Fedora 44 ISO has been built and booted to the declared console marker in UEFI/QEMU**. This proves the basic build-to-boot path for one run. It does not prove production readiness, hardware compatibility, a native Deku package system, reproducibility, signed updates, or an independent root filesystem.

## Verified evidence

- Successful workflow: [Build experimental ISO — run 40003527897](https://github.com/Deku-Foundation/deku-system/actions/runs/40003527897).
- System source commit used by that run: 7425a13d507a9e909c2dc546c81269f1daf6ff43.
- ISO SHA-256: a5d604ea3964a8a05c7a42b0c515586204e0f5c23a1a5e412fa585649eca5a58.
- Uploaded Actions artifact: deku-linux-experimental-iso-40003527897, artifact ID 12524770503.
- Artifact ZIP SHA-256: 847bf4d9a0f38f080f0d619cf5c33b551b1ec91e758469ad263f63d8c773ffaa.
- Artifact ZIP size: 1,832,681,650 bytes; retained until 2026-10-30T18:44:42Z. This is the ZIP size, not the raw ISO size.
- UEFI QEMU smoke test observed DEKU_BOOT_OK:minimal-x86_64 after approximately 190 seconds. Build, Rust profile preflight, artifact ownership fix-up, boot test and artifact upload all passed.
- The profile used at that time was commit 2924c0eabe745542f61985062ad164133e5e006f (Fedora 44), the validator was 9bc9426cb685f91b46af46237b97d03c2c7abf34, and the boot harness was d1c9b69b65e9c72dcd33c68b54559dc8424ef9f8.

**Integrity note:** the ISO digest and Actions ZIP digest identify different files. The ISO digest verifies the image bytes; the ZIP digest verifies the downloaded evidence bundle. The temporary artifact is not a permanent download or an official release.

## Architecture choice

Reuse proven tools at their intended boundaries rather than combining incompatible distribution stacks:

- **Fedora + Lorax/livemedia-creator** is the current path for a real live ISO and Anaconda/Kickstart integration.
- **Nix/NixOS** is the reference for declarative system composition, explicit input graphs and clean rebuild comparisons; NixOS image generation is not a drop-in Fedora ISO backend.
- **Void + void-mklive/XBPS** remains a viable alternative only if Deku explicitly selects the XBPS/runit ecosystem. Upstream warns that void-mklive is not guaranteed to work outside Void or inside containers.
- **Rust** is Deku's implementation choice for the control plane we own: profile/manifest validation, dependency planning, build orchestration, artifact verification and release tooling. Do not rewrite the Linux kernel, libc, systemd, KDE or Lorax just to increase the Rust percentage.

References: [Fedora Lorax](https://github.com/weldr/lorax), [NixOS image building](https://nixos.org/manual/nixos/stable/), [Void live-image builder](https://github.com/void-linux/void-mklive), [SLSA build levels](https://slsa.dev/spec/v1.0/levels).

## What the passing run proves — and what it does not

The workflow built one non-empty Fedora 44-based image, calculated its SHA-256 and launched it under QEMU with a matching OVMF code/variable-store pair. The smoke harness observed the serial marker emitted by the custom systemd oneshot service after systemd-user-sessions.service.

It does **not** yet prove:
- real-hardware boot, BIOS boot, Secure Boot, installation to a disk or successful recovery;
- a full login/network/package-manager acceptance suite, a graphical KDE Plasma desktop or support for other architectures;
- that deku-pkg can install, authenticate, upgrade or roll back packages;
- independent reproducibility, an immutable Fedora package snapshot, a complete SPDX/CycloneDX SBOM, signed release provenance or a production signing/update service.

The build log also contains non-fatal environment warnings, including a GRUB probe warning about the container's overlay filesystem, a missing systemd-tmpfiles-clear.service during presets, and a D-Bus connection unavailable in the no-virt builder. The image still booted to the marker, but those warnings must be classified and not silently treated as irrelevant.

## Current pipeline

The [workflow](../.github/workflows/build-preview-iso.yml) now:
1. checks out immutable revisions for the profile, Rust validator and boot harness;
2. runs `deku-build validate-profile` before the privileged image-building stage;
3. builds in an ephemeral, digest-pinned Fedora 44 container without secrets or host networking;
4. uses Lorax/Anaconda to build the live ISO and emits a checksum and build manifest;
5. changes ownership only on the generated artifact directory before the host-side QEMU test;
6. verifies the ISO digest and boots with OVMF pflash code plus a fresh writable variable store;
7. runs guest-side checks for release metadata, installed package inventory, active NetworkManager, disabled SSH service, and enforcing SELinux before emitting the boot-success marker;
8. streams the installed RPM inventory over serial, validates its syntax/order/uniqueness, and records its digest in the build manifest;
9. emits an unsigned in-toto Statement binding the ISO digest to source revisions, builder image, package-inventory digest and boot result;
10. uploads the ISO, checksum, package inventory, build manifest, unsigned statement and diagnostics as a short-retention Actions artifact.

The current workflow pins the Rust validator to a revision containing Cargo.lock, pins the toolchain to 1.99.1, and runs cargo run --locked -- validate-profile. A subsequent end-to-end run must verify the updated canonical profile and locked validator together.

## Supply-chain limits and security boundaries

- The builder container digest and source checkout revisions are pinned, and the workflow uses read-only repository permissions and does not persist checkout credentials.
- The image build needs a privileged container and downloads packages from Fedora's HTTPS release and updates repositories. It runs on an ephemeral GitHub-hosted runner with no secrets; do not move this workflow to a persistent self-hosted runner without a separately reviewed sandbox.
- Fedora's release/updates repositories are rolling package inputs, not immutable snapshots. Recording build inputs and installed package inventory improves traceability, but does not make the entire image reproducible.
- The in-toto statement is unsigned traceability data, not publisher authentication or a SLSA compliance claim. The RPM inventory is not a complete SBOM: package file hashes, declared licenses, dependency relationships and repository signature evidence are still missing.
- The build has no release signature and no permanent artifact retention.
- The ISO is an experimental artifact only. It must not be advertised as a stable release or recommended for physical installation.

## Acceptance gates

### Gate A — artifact and UEFI console boot: passed once
Build exits successfully; a non-empty ISO and SHA-256 exist; QEMU/OVMF boots it; the marker is observed; logs and hashes are preserved.

### Gate B — current-profile rebuild and basic system checks: in progress
Rerun the workflow with the canonical profile and locked validator. Add post-boot checks for root overlay, expected package manifest, systemd readiness, networking state, SSH disabled and failed units. Retain serial, QEMU stderr and machine-readable results.

### Gate C — build integrity
Capture exact resolved target RPM identifiers and hashes, include epoch in the package inventory, resolve the mutable-repository limitation, emit a standards-conformant SBOM and authenticated provenance, and compare at least two clean builds. Do not claim reproducibility until outputs are compared.

### Gate D — installation and recovery
Only after Gate B passes, test installation to a disposable virtual disk, interrupted operations, recovery and rollback. No physical disk access in CI.

### Gate E — independent Deku system
Design and implement the native recipe/build/package/rootfs path in small increments. Keep Fedora as an explicitly labelled bootstrap/technology-preview base until the native stack passes build, install, update, rollback and security gates.

### Gate F — release
Publish only after signed release metadata, key rotation/revocation procedures, SBOM/provenance, supported scope, known limitations and release CI are in place. The current temporary Actions artifact is not a public release.
