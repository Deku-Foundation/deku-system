# Deku System Composition

Deku System is intended to assemble verified native packages and, later, bootable system images from reviewed, pinned inputs.

## Current status

**Prototype stage.** The system-manifest validator remains metadata-only. Separately, an experimental Fedora 44/Lorax ISO has been generated and passed one UEFI/QEMU serial-marker boot test: [workflow run](https://github.com/Deku-Foundation/deku-system/actions/runs/40003527897). The ISO SHA-256 was a5d604ea3964a8a05c7a42b0c515586204e0f5c23a1a5e412fa585649eca5a58. This proves a limited upstream-based technology-preview boot only; it is not an independent Deku root filesystem, official release, hardware-certification result or production installer.

## Manifest contract

A schema-v1 manifest records:

- A release identifier and target architecture.
- The initial `kde-plasma` system profile.
- Immutable base and kernel revisions, each accompanied by a source digest.
- Catalog and system-configuration digests.
- The build toolchain identifier.
- A bounded package set with exact versions, architecture compatibility and SHA-256 artifact digests.

The validator rejects unknown fields, duplicate JSON keys and package names, unsupported architectures/profiles, unpinned revisions, noncanonical digests, malformed versions and over-sized JSON. It never executes commands from the manifest. The example digests are **fixtures only**, not references to real published artifacts.

## Infrastructure-first status

The experimental Fedora/Lorax ISO path is **paused and its workflow, Kickstart and build script have been removed from `dev`**. It is not the project priority. The failed end-to-end job did not pass the UEFI smoke test: the host-side harness could not create its evidence directory because the build container left root-owned files. No accepted ISO or supported release is claimed.

The active priority is the distribution infrastructure: package recipe and build contracts, isolated workers, native Rust package-manager transactions, repository metadata and trust, installer architecture and upstream-license compliance. See [the infrastructure-first platform plan](https://github.com/Deku-Foundation/deku-linux/blob/dev/docs/strategy/infrastructure-first-roadmap.md) and [the paused ISO experiment record](docs/architecture/first-iso-pipeline.md).

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

## Native Deku architecture

The Fedora 44 image is only a technology-preview test fixture. The production path is a Deku-owned package closure and root filesystem, built from reviewed/pinned components and assembled into media only after native package/repository contracts exist. See [native rootfs and ISO boundary](docs/architecture/native-rootfs-bootstrap.md) and [upstream builder comparison](docs/architecture/upstream-builder-comparison.md).

## Engineering references

- [First ISO pipeline and tool evaluation](docs/architecture/first-iso-pipeline.md)
- [Upstream image-builder comparison: Fedora, NixOS and Void](docs/architecture/upstream-builder-comparison.md)
- [Rust-first/upstream-first architecture and release gates](https://github.com/Deku-Foundation/deku-linux/blob/main/docs/strategy/iso-build-gap-analysis.md)
- [Experimental ISO build workflow](.github/workflows/build-preview-iso.yml)
- [Experimental ISO pipeline security review](docs/security/iso-pipeline-review.md)
- [Upstream image-builder comparison](docs/architecture/upstream-builder-comparison.md)

## Acceptance gates before a supported release

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
