# Deku build pipeline: initial design

**Status:** Proposed. This is not yet a working image builder.

## Pipeline stages

1. **Resolve inputs:** select exact source versions, immutable commits, source URLs, and expected digests.
2. **Review metadata:** validate licensing, dependency declarations, patch inventory, and build instructions before executing upstream code.
3. **Build in isolation:** use a disposable, unprivileged environment with bounded CPU, memory, disk, and network access. Do not mount host credentials or the Docker socket into untrusted builds.
4. **Test artifacts:** inspect file lists, ownership, permissions, setuid bits, symlinks, dependencies, and package metadata. Run package-provided tests when feasible.
5. **Record provenance:** retain input revisions, toolchain versions, environment description, logs, and artifact digests.
6. **Compare rebuilds:** attempt independent rebuilds and compare outputs before making reproducibility claims.
7. **Sign and publish:** a separate trusted release job verifies approved artifacts and provenance before signing or publishing. Pull-request builds must not access signing credentials.

## Initial scope

Start with a small, harmless test artifact and deterministic checks. Do not begin with a complete ISO, bootstrap compiler chain, or production package repository until the small pipeline is repeatable.

## Failure policy

Fail closed when an input hash is missing or mismatched, metadata is malformed, required tests fail, an artifact contains unexpected privileged files, or provenance cannot be associated with the exact source revision. Any temporary exception must be explicit, reviewed, and time-bounded.

## Open decisions

- Initial architecture target and build host support.
- Whether to adapt XBPS/xbps-src or another mature build backend.
- Required network access model during build phases.
- Artifact format and signing design.
- How to provision independent rebuild workers.
