# First ISO Pipeline

Status: plan for a technology-preview image; not a working ISO builder.

## Decision
Prove bootability with a mature upstream base and image tool first. This is a technology preview, not proof that Deku's native package manager is ready or that the system is independently bootstrapped.

## Tool evaluation and current proposal
The current proposal in [ADR-0001](https://github.com/Deku-Foundation/deku-linux/blob/dev/docs/adr/0001-first-iso-and-rust-first-architecture.md) is to evaluate Fedora 44 plus Lorax/livemedia-creator for the first actual ISO, with Fedora 45 only after final repository and builder compatibility are verified. Lorax's documented `livemedia-creator --make-iso` path requires Anaconda install media, a Kickstart file and a suitable build environment; CI feasibility is a gate, not an assumption. Evaluate systemd/mkosi in parallel for disk/system-tree images, not as if it directly emits an ISO. If the prototype base is Void, use the Void-supported void-mklive/XBPS path instead. Pin tool and package inputs before implementation.

## Image stages
1. Validate a declarative profile from deku-profiles.
2. Resolve exact package versions and upstream repository snapshots.
3. Generate an input manifest containing builder version, profile revision, package source, package version and checksums where available.
4. Build an x86_64 UEFI image with a kernel, initramfs, bootloader and minimal userspace.
5. Generate ISO and SHA-256 checksum; retain build logs and configuration.
6. Boot in disposable QEMU with UEFI firmware. Capture serial output.
7. Verify kernel reaches userspace, root is mounted, init/service manager is alive and a test marker exists. A graphical desktop is a later acceptance gate.
8. Store test logs and artifacts with the source commit and tool versions.

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
1. Add a versioned profile schema and minimal console profile.
2. Add a pinned builder/tooling environment and one CI build job.
3. Build the image and store it as a workflow artifact, not a public release.
4. Add the disposable UEFI VM smoke test.
5. Decide the long-term base/libc/init/package strategy using measured results, not preference alone.
