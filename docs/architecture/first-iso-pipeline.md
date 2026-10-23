# First ISO Pipeline

Status: the experimental Fedora 44 ISO has been generated and uploaded as a short-retention CI artifact, but it has **not passed the UEFI boot gate**. The first completed VM attempt failed before firmware startup because the harness passed OVMF as a legacy `-bios` image. The harness now uses a matching OVMF pflash code/variable-store pair, and CI is retesting that fix. No successful boot is claimed.

## Decision
Prove bootability with a mature upstream base and image tool first. This is a technology preview, not proof that Deku's native package manager is ready or that the system is independently bootstrapped.

## Tool evaluation and current proposal
The current proposal in [ADR-0001](https://github.com/Deku-Foundation/deku-linux/blob/dev/docs/adr/0001-first-iso-and-rust-first-architecture.md) is to evaluate Fedora 44 plus Lorax/livemedia-creator for the first actual ISO, with Fedora 45 only after final repository and builder compatibility are verified. Lorax's documented `livemedia-creator --make-iso` path requires Anaconda install media, a Kickstart file and a suitable build environment; CI feasibility is a gate, not an assumption. Evaluate systemd/mkosi in parallel for disk/system-tree images, not as if it directly emits an ISO. If the prototype base is Void, use the Void-supported void-mklive/XBPS path instead. Pin tool and package inputs before implementation.

## Current implementation and failure evidence

The current workflow lives at `.github/workflows/build-preview-iso.yml` and runs only for selected builder-file changes on `dev` or manual dispatch. It checks out a pinned Fedora 44 container image and a pinned profile commit, then runs `livemedia-creator --make-iso --no-virt --iso-only` inside an ephemeral privileged container without host networking or secrets. It retains builder inventory, build logs and artifacts. The privilege boundary is acceptable only for a disposable GitHub-hosted runner; do not run this workflow's privileged container on a personal workstation.

The first attempt installed the target packages, configured systemd, and generated the live initramfs, but the final ISO template failed because `grub2-mkimage` could not find `/usr/lib/grub/i386-pc/moddep.lst` in Lorax's staging tree. The Kickstart and profile now explicitly include `grub2-pc`, `grub2-pc-modules`, `grub2-tools`, `grub2-tools-extra`, and the UEFI boot packages. The marker service ordering was changed from `After=multi-user.target` to `After=systemd-user-sessions.service` to avoid a target ordering cycle. The rerun must prove the fix; source changes and passing contract tests alone are not proof.

A later build completed image authoring and uploaded the ISO, manifest, checksums and logs. Its VM step then failed with `qemu: could not load PC BIOS '/usr/share/OVMF/OVMF_CODE_4M.fd'`. The root cause was a harness invocation error, not evidence that the ISO itself failed to boot: OVMF code must be loaded as read-only pflash with a separate writable variables image, not through QEMU's legacy `-bios` option. `deku-tests` now copies the matching variable-store template into a temporary directory and launches QEMU with paired pflash drives. The next run must prove the firmware starts and the serial boot marker appears.

The profile and builder container are pinned by commit/digest, but Fedora's release and updates repositories are still rolling package inputs. The in-image RPM manifest and external build manifest improve traceability but do not yet make the entire system build reproducible. A future release gate must snapshot or otherwise lock all resolved package versions and preserve provenance/SBOM evidence.

## Image stages
1. Validate the pinned profile and require the Kickstart to include its package set plus live-image bootloader requirements.
2. Resolve the Fedora 44 release/updates package set; record the limitation that these repositories are not yet an immutable snapshot.
3. Generate a build manifest containing the source commit, profile revision, builder image digest, Lorax version, Kickstart/profile checksums and builder package inventory.
4. Use Lorax/Anaconda to assemble an x86_64 live root filesystem, initramfs and bootable ISO.
5. Require one non-empty ISO and generate a SHA-256 sidecar; retain build logs.
6. Boot in disposable QEMU with UEFI firmware. Capture serial output and QEMU stderr.
7. Require the marker service to prove userspace reached the declared target. This is a console smoke test, not a desktop or hardware certification.
8. Store the ISO, checksums, manifest, boot result and logs as short-retention workflow artifacts, not a public release.

## First target
- Architecture: x86_64.
- Firmware: UEFI in the first CI gate.
- User experience: console login before any desktop requirement.
- Desktop: KDE Plasma remains the target direction, but defer it until the minimal image boots reliably.
- No physical disk installation, production signing, Secure Boot claim or automatic updater in the first gate.

## Hard requirements
- Never use floating package versions for a release candidate without recording the resolved snapshot.
- Never include signing keys in the image-build workflow.
- Fail if package sources are unavailable or checksums differ.
- Fail if the VM does not reach the declared userspace target.
- Publish no ISO link unless the artifact exists and boot-test evidence is available.

## Immediate engineering tasks
1. Verify the paired OVMF pflash invocation and confirm QEMU reaches firmware and the live kernel.
2. Require the boot-marker service to emit `DEKU_BOOT_OK:minimal-x86_64`; inspect preserved serial output and QEMU stderr if it does not.
3. Keep the ISO private to short-retention CI artifacts until boot evidence is recorded; do not promote on metadata-only checks.
4. Snapshot or lock resolved Fedora package inputs before making reproducibility claims.
5. After a successful console boot, compare the upstream-base demo against an independent native package/rootfs track, and record the long-term base/libc/init/update decisions as ADRs.
