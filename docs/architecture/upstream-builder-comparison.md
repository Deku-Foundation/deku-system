# Upstream image-builder comparison

Status: engineering decision note, reviewed 2026-10-23. This is a comparison of build paths, not a claim that any path has produced a supported Deku release.

## Recommendation

**Do not use Fedora as Deku's runtime base.** The Fedora 44/Lorax image is a labelled boot-harness fixture only: it proves that the current UEFI/QEMU test path can observe a console marker, not that a native Deku system exists. Keep this fixture isolated from the future native package closure and never promote it as a Deku release.

The production direction is a Deku-owned package closure and root filesystem assembled from reviewed, pinned components. Use upstream projects as implementation references and individually selected dependencies. First build one native package in a real unprivileged sandbox, authenticate a local repository, assemble a minimal rootfs, then use an image assembler only for boot media. Evaluate systemd/mkosi with a custom/prebuilt rootfs; it is not itself the package manager and does not imply Fedora is the base.

Do not combine three incompatible distribution stacks into one root filesystem. Reuse the best upstream technology at its intended boundary:

- Fedora/Lorax for an Anaconda/Kickstart-based live or install image.
- Nix/NixOS image generation as a reference for declarative inputs and reproducible closures, not as an implicit Fedora package manager.
- Void `void-mklive` as an alternative if a later architecture decision selects XBPS and runit.

## Comparison

| Path | What it already does | What it means for Deku | Main caveat |
|---|---|---|---|
| Fedora Lorax / `livemedia-creator` | Mature live-media/installer implementation and useful reference for boot layout and installer testing. | Keep as a reference and isolated preview fixture only; it must not define the native target package closure. | Tight Fedora/Anaconda/RPM assumptions; privileged builder is a separate trust domain. |
| Nix / Nixpkgs | Provides strong reference patterns for declarative inputs, pinned dependency graphs, sandboxed builds and independent rebuild comparison. | Borrow these properties for Deku's own manifest/build graph; NixOS itself is not the target runtime base. | Adopting NixOS modules and rootfs would be a separate architecture decision. |
| Void `void-packages` / XBPS / `void-mklive` | Useful references for recipe phases, build options, binary repository mechanics, signatures and live-media composition. | Adapt selected concepts to a Deku-owned package contract; do not silently inherit XBPS or runit. | `void-mklive` expects the Void/XBPS ecosystem and is not a general-purpose Deku image backend. |

## Independent native target

The target runtime must be composed from a Deku-owned, locked package manifest and rootfs. Individually selected upstream components (Linux, glibc or musl, systemd or another init, GRUB/firmware, utilities) remain normal dependencies with explicit versions, hashes, licences and provenance. They do not make the distribution a derivative by themselves. A copied foreign rootfs, an inherited foreign package database, or a requirement to use another distribution's repositories at runtime would violate the independence goal.

See the native architecture proposal in [deku-linux](https://github.com/Deku-Foundation/deku-linux/blob/dev/docs/architecture/native-distribution-architecture.md) and the implementation boundary in [native rootfs bootstrap](native-rootfs-bootstrap.md).

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
