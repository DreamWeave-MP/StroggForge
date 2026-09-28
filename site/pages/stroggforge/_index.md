+++
title = "StroggForge"
description = "Canonical Rust workflow contracts and the incident desk."
weight = 30
sort_by = "weight"
template = "docs/section.html"
page_template = "docs/page.html"
+++

StroggForge is the shared build/release layer for Rust applications and Rust ecosystem
components. It verifies, manufactures, signs, documents and publishes configured consumers.
It also hosts this engineering inventory. Visualizing a mod here does not make that mod
depend on StroggForge.

- **Applications:** `rustGlobalBuild.yml`, with JSON `binary_names`.
- **Libraries:** `libGlobalBuild.yml`, with Cargo `crate_names`.
- **Binary manufacturing:** `corprus-crucible`; tests run in the calling workflow.
- **Compiler setup:** `setup-llvm`, after Rust setup in compiling jobs.

Use the [workflow reference](@/stroggforge/workflows/_index.md),
[input and secret reference](@/stroggforge/inputs.md),
[integration templates and actual callers](@/stroggforge/integration.md), and
[incident desk](@/stroggforge/troubleshooting.md). The hand-maintained long-form reference,
[StroggForge Actions](https://github.com/DreamWeave-MP/StroggForge/blob/main/StroggForge%20Actions.md),
is updated alongside workflow changes and covers the helper scripts in prose.

Static sites on the Lua(u)/OpenMW and web sides (St4sh, the Mod Template) call only the
`createRelease` helper. That is a CI convenience, not membership in the Rust ecosystem.
