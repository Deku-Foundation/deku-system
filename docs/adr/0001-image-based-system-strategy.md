# ADR 0001: Evaluate an image-based system before building a custom image pipeline

- **Status:** Proposed; no implementation choice approved
- **Date:** 2026-10-23
- **Owners:** Deku System maintainers
- **Scope:** System composition, operating-system updates, image generation and installer integration

## Context

Deku currently validates a system manifest, package/build plans and installation plans. These are metadata-only prototypes: they do not assemble a root filesystem, produce a bootable image, install to a disk or publish a release. Implementing all of those layers from scratch would create a large security and maintenance burden before the project has proven its system design.

The architecture must preserve the ability to audit inputs, pin source revisions, verify artifacts, test recovery and separate build, signing and publishing trust domains. It must also fit the initial KDE Plasma desktop and x86_64 target without turning a prototype into a production claim.

## Options to evaluate

### A. Image-based system using bootc / OCI images

Use the upstream bootc model as a reference and possible deployment foundation. bootc uses OCI/container images as a transport for transactional, in-place operating-system updates. This could avoid inventing a complete update and deployment stack, but compatibility with Deku's desired package format, system composition, boot flow, offline installation and long-term distro identity must be demonstrated.

Candidate upstream references:
- https://github.com/bootc-dev/bootc
- https://github.com/osbuild/bootc-image-builder

### B. OSTree-based composition and deployment

Evaluate OSTree's deployment and rollback model directly or through an established distribution implementation. This may offer a strong update/recovery model, but integration and maintenance cost must be measured against the bootc route rather than assumed to be lower.

### C. Custom root-filesystem and image pipeline

Continue with Deku-owned composition, image generation and update infrastructure. This gives maximum control but also makes Deku responsible for bootloader integration, crash-safe updates, rollback, filesystem layout, hardware compatibility, image formats, signing, metadata freshness, recovery and security fixes. Do not select this merely because it aligns with the current prototype repository names.

## Decision

**Do not commit to a production backend yet.** Run a small, disposable proof of concept using a maintained upstream image-based approach and compare it with the minimum custom design. The existing manifest validator remains a useful input contract, not proof that any backend has been selected.

The prototype must not write to a host disk or use release credentials. Use a disposable VM and public test keys only.

## Proof-of-concept gates

1. Build a minimal x86_64 KDE Plasma-capable image from pinned inputs in a disposable environment.
2. Boot it in QEMU and record boot logs, system identity, kernel, services and image digest.
3. Demonstrate an update, interrupted-update behavior, rollback/recovery and a failed-update path.
4. Generate an SBOM and build provenance; verify image digest and signing with non-production keys.
5. Test whether package/catalog metadata can be integrated without confusing artifact integrity with publisher authentication.
6. Record build time, storage cost, reproducibility evidence, operational complexity and upstream maintenance burden.
7. Document install-media support, offline installation, bootloader handling, kernel updates, persistent user data and configuration changes.
8. Compare the same acceptance checklist against a custom pipeline estimate.

## Acceptance and exit criteria

The evaluation is complete only when the results, commands, tool versions, pinned inputs, logs and known failures are committed to a reviewable report. A successful manifest validation alone is not an image-build success. A successful VM boot alone is not hardware support or release readiness.

A follow-up ADR must choose the backend, record rejected alternatives and explain how Deku's package manager and installer fit it. Until then, do not advertise a bootable ISO, transactional updates, reproducible releases or production signing.

## Security constraints

- Pin source revisions and toolchain versions; record artifact digests.
- Keep build workers unprivileged, disposable and network-restricted during build execution.
- Never expose signing keys or publication credentials to build jobs or pull-request code.
- Treat checksums as integrity checks, not publisher authentication.
- Test rollback and recovery before any real-device installation is attempted.
