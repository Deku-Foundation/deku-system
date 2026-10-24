# Distribution build infrastructure — active design

**Status:** proposal and component contract, not a deployed build service. Fedora/Lorax ISO production is paused; image building is not the project priority.

## End-to-end package flow

1. Review a recipe: immutable source revision/digest, patch digests, SPDX license expression, ABI/architecture, dependencies, outputs and tests.
2. Acquire sources in a network-enabled fetch stage with redirect/IP/size/time limits. Verify each digest and preserve the exact fetched bytes.
3. Run build steps in a disposable VM from a pinned build-environment identity. The build guest receives no secrets and has no network by default. The host retains no untrusted mount points or Docker socket access.
4. Validate produced archives: normalized relative paths, traversal, symlink policy, duplicate entries, setuid/setgid, devices/FIFOs, ownership policy, entry counts and decompression/resource limits.
5. Emit package artifact, package manifest, tests, logs and provenance bound to recipe revision, patches, toolchain and build environment.
6. Compare independent rebuilds. Normalize timestamps and build paths where supported; record exceptions. Do not assert reproducibility without comparing outputs.
7. Publish only after artifact verification and approval. Signing metadata runs separately from builds and PR jobs.
8. Install through a transaction planner that verifies signed repository metadata, artifact digest, dependencies, conflicts, disk space and target root; journal staged changes and test interruption recovery.

## Lessons from upstream systems

- Void separates recipe/build phases from the XBPS binary package and repository tools.
- Nix makes inputs and dependency graphs explicit and enables isolated rebuild-oriented execution.
- Debian's buildinfo/rebuild workflows expose the actual build environment and compare output bytes.
- TUF separates repository trust roles and freshness from transport checksums.
- These are architectural references, not an instruction to stack all their package formats or root-update models together.

## Non-goals

- No active Fedora ISO workflow exists while the pause remains in effect; ISO production is not the project priority.
- No arbitrary recipe shell executing on the coordinator or host.
- No build or publication credentials in workers.
- No production signing, package repository or update claims yet.

See the [infrastructure-first plan](https://github.com/Deku-Foundation/deku-linux/blob/dev/docs/strategy/infrastructure-first-roadmap.md) for milestones and source-reuse policy.
