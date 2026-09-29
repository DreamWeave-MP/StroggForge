+++
title = "Publishing and recovery"
description = "What a tag sets in motion, and how not to make a half-finished release worse."
weight = 30
+++

## Before You Tag

Put the release in a [plan](@/contributing/maintenance.md#planning-a-release) first:
version, platforms, channels, and what has to be true before it ships. Check which
StroggForge ref the consumer pins and which inputs it passes. The version in `Cargo.toml`
is not a plan, and the war room never creates tags for you.
[Release Planning](https://github.com/DreamWeave-MP/StroggForge/blob/main/Release%20Planning.md)
has the manual checklist for Rust applications.

## Where Things Get Published

| Destination | When | Needs |
|---|---|---|
| GitHub Release | After every platform build is staged; the release is refreshed first | `contents: write`; for binaries, signing and scanning setup |
| crates.io | Tags, when publishing is enabled | `CARGO_REGISTRY_TOKEN`, a correct package version |
| crates.io, whole workspace | Tags with `cargo_publish_workspace` | Dependency order and index visibility, both handled by the publisher |
| Rustdoc on Pages | Main pushes with `publish_docs`, after the gates | Pages source set to Actions, Pages and OIDC permissions |
| Changelog and benchmarks | After the release refresh | Full Git history for the changelog; benchmarks are opt-in |
| AUR | Configured pushes after the platform builds | Package name, SSH key, the AUR environment |
| Nexus | Configured non-PR runs after the builds | API key plus a file group ID per platform |
| PortMaster | Opt-in ARM64 native binary | A device that runs it. Catalog submission is a separate, manual step. |
| Mod Template site | With `mod_template`, after the GitHub Release or crates.io publish | `actions: write` for the caller; the site's `mod.lock` on the default branch is committed by CI |

## A Partial Release Is Not a Clean Slate

Refreshing a GitHub Release deletes and recreates it; for `development` it deletes the tag
too. That is intentional, and it means anything uploaded before the refresh is gone.

Before rerunning a failed release, find out what already went out. crates.io does not
forget a version because you deleted the GitHub Release. The workspace publisher skips
versions that are already published, retries rate limits and waits for the index, so
read its log before reaching for `cargo publish` yourself.

## Closing the Plan

Link the Actions run, the artifact and any device test on the requirements they finish.
Leave a blocker in place until its reason is actually gone. The site reports what was
recorded; it does not go and check.

[Release board](@/releases/board.md) · [Pipelines](@/releases/pipelines.md) ·
[Incident desk](@/stroggforge/troubleshooting.md)
