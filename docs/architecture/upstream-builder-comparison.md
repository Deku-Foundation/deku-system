# Upstream image-builder comparison

Status: engineering decision note, reviewed 2026-10-23. This is a comparison of build paths, not a claim that any path has produced a supported Deku release.

## Recommendation

A Fedora 44-based image has now passed one UEFI smoke test. Keep Fedora + Lorax as the known-good technology-preview baseline while expanding runtime checks. Do not promote a new Fedora profile based on repository availability or metadata tests alone; verify package inputs, rebuild, boot and compare it with the baseline first.

Do not combine three incompatible distribution stacks into one root filesystem. Reuse the best upstream technology at its intended boundary:

- Fedora/Lorax for an Anaconda/Kickstart-based live or install image.
- Nix/NixOS image generation as a reference for declarative inputs and reproducible closures, not as an implicit Fedora package manager.
- Void `void-mklive` as an alternative if a later architecture decision selects XBPS and runit.

## Comparison

| Path | What it already does | What it means for Deku | Main caveat |
|---|---|---|---|
| Fedora Lorax / `livemedia-creator` | Uses Anaconda and Kickstart to create live ISOs, disk images and filesystem images. | Best fit for proving a Fedora-based UEFI preview while retaining Fedora's kernel, dracut, systemd, RPM and package metadata. | A privileged builder is required for the current containerized workflow; it is not an independent Deku rootfs. |
| NixOS / Nix image generation | Builds custom ISO configurations from NixOS modules and a pinned Nixpkgs revision. | Strong reference for immutable inputs, declarative system composition and reproducible build graphs. | Choosing a NixOS rootfs would make NixOS the base distribution; it is not a drop-in ISO generator for a Fedora rootfs. |
| Void `void-mklive` | Builds Void live ISOs and rootfs/platform images, including x86_64 and aarch64 variants. | A credible alternate base if Deku deliberately adopts XBPS and runit, with glibc/musl variants considered separately. | The upstream project warns that it is not guaranteed to work outside Void Linux or inside containers. Its package and service model differs from Fedora. |

## What should be borrowed

1. **From Fedora:** reuse upstream kernel, dracut, systemd, Anaconda/Kickstart and RPM packaging for the technology preview; keep the package set minimal.
2. **From Nix:** represent inputs explicitly, pin source revisions and builder images, record package inventories and hashes, and make the build graph declarative. Do not claim bit-for-bit reproducibility until two clean builds prove it.
3. **From Void:** study the direct live-ISO workflow, rootfs generation, architecture separation and musl/glibc split before making any long-term libc/init decision.
4. **From all three:** verify image integrity, record provenance, boot in a disposable VM, and make release promotion depend on evidence rather than on a successful compile alone.

## Promotion gates

- **Gate A — build:** the builder exits successfully and creates exactly one non-empty ISO; save the build log, builder package inventory, profile/Kickstart hashes and ISO SHA-256.
- **Gate B — UEFI boot:** QEMU/OVMF boots the ISO and the userspace marker is observed on the serial console. Preserve the boot log and machine-readable result.
- **Gate C — minimal functionality:** verify the root overlay, kernel, systemd, networking and package manifest. This is the next test layer after the first serial marker.
- **Gate D — installer:** only after A–C, add a disposable-disk installation test. Do not point an experimental workflow at a physical disk.
- **Gate E — repeatability:** run two clean builds from the same pinned inputs and compare package manifests, provenance and artifact hashes. If the ISO bytes differ, document the source of nondeterminism before making reproducibility claims.
- **Gate F — native Deku components:** only later, integrate `deku-pkg` after it can safely install, verify, upgrade and recover packages. Do not make the first ISO depend on unfinished native package tooling.

## Source documentation

- Fedora Lorax: https://github.com/weldr/lorax
- Fedora `livemedia-creator` documentation: https://github.com/weldr/lorax/blob/master/docs/livemedia-creator.rst
- NixOS custom ISO guide: https://nix.dev/tutorials/nixos/building-bootable-iso-image
- Void live-image builder: https://github.com/void-linux/void-mklive
- QEMU UEFI firmware mapping (split code/NVRAM pflash): https://github.com/qemu/qemu/blob/master/docs/interop/firmware.json

## Current limitation

The current build pipeline is an upstream-based technology preview. A successful boot proves only that the selected Fedora-based image can be built and reach the test marker. It does not establish a finished Deku distribution, a working native package manager, a secure update service, a desktop environment, a physical-disk installer or a production release process.
