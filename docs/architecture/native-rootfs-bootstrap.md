# Native Root Filesystem and ISO Boundary

- **Status:** Proposed implementation contract.
- **Goal:** Make the first Deku-native ISO independent of Fedora, Void and NixOS runtime payloads.
- **Current evidence:** A Fedora 44/Lorax preview booted once in UEFI/QEMU. That is a harness fixture only.

## Required dependency direction

deku-recipes (reviewed source declarations)
→ deku-build (verified fetch + isolated build)
→ native Deku package artifacts
→ deku-pkg (authenticated repository + dependency closure)
→ locked deku-profiles system manifest
→ deku-system rootfs assembler
→ kernel + initramfs + UEFI boot assets
→ ISO layout/assembler
→ deku-tests VM acceptance
→ deku-release signed release manifest.

The arrow represents a required integration contract, not a claim that every stage currently exists. The existing validators and TUF wrapper are prototypes; they do not yet make packages or install a system.

## Runtime independence invariant

The final target root must not be populated by copying a Fedora/Void/NixOS installation or by running its installer. It must not require that distribution's package manager, release repositories or package database to update. Upstream projects may supply individually reviewed components (for example Linux, glibc, systemd, dracut, GRUB, firmware and utilities), but each component must have a Deku package record with exact source/version, digest, licence, patches, build recipe and provenance. Importing a prebuilt upstream binary is allowed only through an explicit bootstrap exception with a recorded digest and provenance; it must not silently become the native package pipeline.

The existing Fedora preview remains a separate test target and is not a candidate for promotion.

## Image assembly boundary

The image assembler receives a prebuilt rootfs and a machine-readable boot manifest. It must not resolve arbitrary target packages from Fedora/Void repositories. Evaluate systemd/mkosi in its custom/prebuilt-rootfs mode first because it supports a custom rootfs boundary; compare it with a narrow ISO layout implementation only if the interface cannot meet requirements. This is an engineering evaluation, not a decision to adopt Fedora as a base. Record the exact mkosi version/commit and dependencies before using it in CI.

Image assembly must:
- accept only a rootfs produced from the locked Deku manifest;
- validate required paths and reject unexpected setuid/setgid files, device nodes and escaping symlinks;
- install the pinned kernel/initramfs/bootloader assets declared by the manifest;
- reset /etc/machine-id and other instance-specific credentials safely;
- avoid carrying build credentials, caches or host state into the image;
- emit a machine-readable file/package inventory, SBOM, source manifest, build logs and ISO digest;
- produce an image that is tested in a disposable UEFI VM before release promotion.

## First native profile

The first profile should be console-only and x86_64 UEFI. Desktop packages are deliberately excluded until the rootfs and update model work. Kernel, initramfs, bootloader, firmware, libc, init/service manager, shell, core utilities, networking and certificate roots must be explicit package inputs. A profile cannot refer to mutable repository labels without locking the exact metadata and package artifacts used for that build.

## Immediate work sequence

1. Accept or revise the architecture proposal in deku-linux/docs/architecture/native-distribution-architecture.md.
2. Add a schema-versioned native profile and locked package-closure format. Do not overload the Fedora Kickstart profile with native semantics.
3. Implement a real, unprivileged build worker in deku-build; its first task should build one tiny, licence-clear test package from pinned source.
4. Build a local TUF repository fixture and test trust bootstrap, expiry, rollback, key rotation/revocation and target tampering against the existing Rust wrapper.
5. Define the native package artifact format and make deku-pkg consume it without installing to the host.
6. Assemble the rootfs from that local authenticated closure, then evaluate mkosi only for media layout.
7. Add runtime VM tests beyond the serial marker: root mounted, expected files present, systemd failed-unit check, networking state, console/login and clean shutdown.
8. Produce two clean builds and compare package closure, rootfs inventory, SBOM and ISO bytes. Explain every known nondeterministic field.

## Safety boundary

Do not run the privileged Fedora preview builder against physical machines or a persistent self-hosted runner. Do not grant signing keys or publication credentials to package build jobs. Do not claim that the native image exists until the native package/rootfs stages and VM tests actually pass.
