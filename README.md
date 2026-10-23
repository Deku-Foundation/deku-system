# Deku Linux build system

This repository will contain the tooling that assembles packages and, later, bootable images from reviewed inputs.

## Status

Bootstrap stage. There is no supported image builder, ISO, or release pipeline yet.

## Design constraints

- Start with a minimal, auditable build pipeline before attempting full distribution images.
- Run untrusted build steps in isolated, unprivileged environments.
- Pin source inputs and record exact versions, hashes, toolchain versions, and build configuration.
- Keep build execution separate from signing and publishing credentials.
- Add tests for clean builds, failure handling, and artifact inspection.
- Do not claim reproducible builds until independent rebuilds have been compared.

## Planned milestones

1. Document the target platform and build threat model.
2. Compare mature upstream build approaches and record an ADR.
3. Build one harmless test artifact in CI.
4. Add provenance and independent rebuild checks.
5. Only then prototype a bootable image.

Architecture decisions remain proposals until measured and reviewed.
