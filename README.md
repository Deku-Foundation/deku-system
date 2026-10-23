# Deku System Composition

Deku System is intended to assemble verified native packages and, later, bootable system images from reviewed, pinned inputs.

## Current status

**Prototype stage.** The repository now contains a metadata-only system composition manifest validator and example fixture. It does not download package artifacts, execute build commands, assemble a root filesystem, produce an initramfs, create an ISO, sign a release, or publish images. There is no supported image builder or release pipeline yet.

## Manifest contract

A schema-v1 manifest records:

- A release identifier and target architecture.
- The initial `kde-plasma` system profile.
- Immutable base and kernel revisions, each accompanied by a source digest.
- Catalog and system-configuration digests.
- The build toolchain identifier.
- A bounded package set with exact versions, architecture compatibility and SHA-256 artifact digests.

The validator rejects unknown fields, duplicate JSON keys and package names, unsupported architectures/profiles, unpinned revisions, noncanonical digests, malformed versions and over-sized JSON. It never executes commands from the manifest. The example digests are **fixtures only**, not references to real published artifacts.

## Run validation

With Python 3.12 or newer:

```sh
python -m unittest discover -s tests -v
PYTHONPATH=. python -m deku_system.cli examples/system-manifest.json
```

A successful validation means the metadata conforms to the schema and basic consistency rules. It is not evidence that the input artifacts exist, their publisher is authenticated, a root filesystem can be assembled, or an image can boot.

## Architecture constraints

- Keep manifest parsing separate from acquisition, building, signing and publishing.
- Pin source commits, artifact digests, configuration and toolchain before attempting assembly.
- Fetch and verify inputs before entering a non-root, disposable, resource-limited build environment with network access disabled during build steps.
- Do not let recipes or manifest values become shell commands.
- Keep release-signing keys outside builders and source control.
- Do not introduce host-changing workflows until an image-build backend, SBOM/provenance, signing and recovery tests are implemented.

## Open design decision: ownership of the system root

Before choosing how updates switch the live host system, compare a generation-based root design with an image-based approach such as OSTree/bootc in disposable VMs. The comparison must include crash consistency, bootloader and kernel integration, `/etc` merge rules, disk-space limits, upgrades, recovery, and rollback. Deku should select one transaction owner for the system root rather than stacking competing managers over the same paths.

## Acceptance gates before an ISO

1. Validate and authenticate all artifact/repository metadata.
2. Assemble a minimal root filesystem in an isolated worker using verified inputs.
3. Emit machine-readable provenance, SBOM and signed output metadata.
4. Test independent rebuilds for selected components.
5. Boot clean installation images in a disposable VM.
6. Test upgrades, interrupted builds/updates, previous-kernel recovery and rollback.
7. Publish checksums, signatures, sources, known limitations and test evidence.

## Experimental language work: Rukn

The repository includes a temporary **Stage-0 prototype** for Rukn (رُكن), an Arabic-first programming language. It currently provides a Rust-hosted lexer, parser, static checks for a small type set, and a bounded interpreter. It is not a native compiler or a self-hosted language yet.

Start with [the Rukn overview](experiments/rukn/README.md), then read the [language specification](experiments/rukn/LANGUAGE-SPEC.md), [architecture](experiments/rukn/ARCHITECTURE.md), [security notes](experiments/rukn/SECURITY.md), [research and positioning](experiments/rukn/RESEARCH.md), and [roadmap](experiments/rukn/ROADMAP.md).

Rukn is incubated here temporarily; before a public alpha it should move to a dedicated repository with its own releases, issues and governance. No language feature is considered complete without tests and an updated specification.
