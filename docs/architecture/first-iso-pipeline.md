# ISO experiment — paused

**Status: paused by project direction, 2026-10-24.** ISO production is not the current priority. The active Fedora 44/Lorax workflow, build script, Kickstart and tests were removed from `dev`. Do not reactivate or recreate this pipeline unless a future architecture decision makes an integration image necessary for validating lower-level components.

## Why it was paused

The experiment had too much distribution-specific work in the system repository before Deku had stable package recipes, isolated build workers, a native package transaction model, trusted repository metadata and an installer architecture. That ordering risks building a Fedora-derived image rather than building the infrastructure that makes Deku independent.

The last reviewed end-to-end attempt did **not** pass the boot acceptance gate. The job's image-generation stage completed and uploaded artifacts, but the host-side QEMU test exited before completing because it could not create the evidence directory under a root-owned output path. That is a test-harness permissions failure, not proof of successful boot. Do not describe this attempt as a verified boot or a supported ISO.

## Retained learning

- Image-generation tools solve a narrow packaging/integration problem; they do not replace a recipe system, package repository, update trust model, or release process.
- Fedora's Anaconda/Lorax code may be evaluated for selective code reuse, but Deku must not quietly inherit Fedora's package manager, release assumptions, branding or system dependencies.
- All future proof runs must be disposable, secret-free, digest-pinned, independently reviewable and have saved logs even on failure.
- The first useful vertical slice is infrastructure-level: a reviewed recipe -> isolated build artifact -> verified manifest -> signed repository metadata -> deterministic install transaction. An ISO is downstream of that slice.

## Research references

- [Fedora Anaconda installer source](https://github.com/rhinstaller/anaconda) — GPL-2.0, requires license and dependency audit before copying or deriving code.
- [Void XBPS package manager](https://github.com/void-linux/xbps) and [void-packages build system](https://github.com/void-linux/void-packages) — useful package/repository and source-recipe design references.
- [Nixpkgs](https://github.com/NixOS/nixpkgs) — declarative recipes and immutable build-input graphs; its MIT license applies to repository expression code, not automatically to package sources.
- [TUF specification](https://theupdateframework.github.io/specification/) — repository metadata roles, freshness, rollback resistance and key rotation.
- [Reproducible Builds documentation](https://reproducible-builds.org/docs/) — source of the input/environment normalization policy.

## Rule

Keep this page as historical architecture context. No ISO workflow should be scheduled or auto-triggered while the infrastructure roadmap remains the priority.
