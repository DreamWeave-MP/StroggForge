+++
title = "StroggForge"
description = "The Rust supply line: workflow contracts, integration and the incident desk."
weight = 30
sort_by = "weight"
template = "docs/section.html"
page_template = "docs/page.html"
+++

StroggForge is the build and release layer for DreamWeave's Rust applications and
libraries. It verifies, builds, signs, packages, documents and publishes them. It also
hosts this site. Showing up on a map here does not make a mod depend on it.

- **Applications** call `rustGlobalBuild.yml` with a JSON array of `binary_names`.
- **Libraries** call `libGlobalBuild.yml` with Cargo `crate_names`.
- **Corprus Crucible** builds, signs and packages one binary. It does not run tests; the
  calling workflow already did.
- **setup-llvm** puts clang and lld matching rustc's LLVM on every compiling job.

The [workflow contracts](@/stroggforge/workflows/_index.md) are generated from the YAML,
the [input reference](@/stroggforge/inputs.md) lines every knob up side by side, and
[integration](@/stroggforge/integration.md) has the caller templates. When something
breaks, go to the [incident desk](@/stroggforge/troubleshooting.md).
[StroggForge Actions](https://github.com/DreamWeave-MP/StroggForge/blob/main/StroggForge%20Actions.md)
is the long-form reference for the helper scripts.

St4sh and the Mod Template call `createRelease` to refresh the release their static site
uploads to. That is borrowing a helper, not joining the Rust ecosystem.
