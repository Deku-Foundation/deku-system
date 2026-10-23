# Deku System Composition

Deku System is intended to assemble verified native packages and, later, bootable system images from reviewed, pinned inputs.

## Current status

**Prototype stage.** The manifest validator is metadata-only: it does not download package artifacts, execute build commands, assemble a root filesystem, produce an initramfs, sign a release, or publish images. A separate experimental Fedora 44/Lorax path (`kickstart/deku-minimal-live.ks` plus `scripts/build-preview-iso.sh`) attempts to create a live ISO for UEFI/QEMU testing. This is an upstream-based technology preview, not an independent Deku root filesystem or a supported release. No ISO is accepted until CI produces an artifact and the VM boot test records evidence.

## Manifest contract

A schema-v1 manifest records:

- A release identifier and target architecture.
- The initial `kde-plasma` system profile.
- Immutable base and kernel revisions, each accompanied by a source digest.
- Catalog and system-configuration digests.
- The build toolchain identifier.
- A bounded package set with exact versions, architecture compatibility and SHA-256 artifact digests.

The validator rejects unknown fields, duplicate JSON keys and package names, unsupported architectures/profiles, unpinned revisions, noncanonical digests, malformed versions and over-sized JSON. It never executes commands from the manifest. The example digests are **fixtures only**, not references to real published artifacts.

## Experimental ISO builder

The `kickstart/deku-minimal-live.ks` and `scripts/build-preview-iso.sh` files define a first Fedora 44-based live-image experiment. The `Build experimental ISO` workflow runs only on pushes to `dev` that touch the builder files or by manual dispatch; it is deliberately not triggered by pull requests and receives no secrets because the upstream image builder needs elevated mount privileges. The job uses a disposable GitHub-hosted runner and an ephemeral container. Review the workflow's privilege boundary before adapting it to another runner.

This is not a supported Deku ISO. The current implementation is a Fedora 44-based technology preview; it does not establish an independent Deku root filesystem, prove the native package manager, or justify a production readiness claim. The builder currently runs only on the ephemeral GitHub-hosted runner, without secrets, because it uses a privileged container for the no-virt image assembly path. Do not run the privileged build script directly on a personal workstation. A successful build without preserved boot evidence is not acceptance.

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

## Engineering references

- [First ISO pipeline and tool evaluation](docs/architecture/first-iso-pipeline.md)
- [Upstream image-builder comparison: Fedora, NixOS and Void](docs/architecture/upstream-builder-comparison.md)
- [Rust-first/upstream-first architecture and release gates](https://github.com/Deku-Foundation/deku-linux/blob/main/docs/strategy/iso-build-gap-analysis.md)
- [Experimental ISO build workflow](.github/workflows/build-preview-iso.yml)

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
