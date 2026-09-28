+++
title = "Publishing and recovery"
description = "What a tag changes and how to avoid making a partial release worse."
weight = 30
+++

## Before tagging

Confirm the intended package version, quality gates, docs, artifacts, supported platforms
and distribution targets in a release plan. Review the consumer's actual caller ref and
inputs. Cargo.toml's current version is not the release plan. The war room does not create tags.
The manual Rust-application checklist lives in
[Release Planning](https://github.com/DreamWeave-MP/StroggForge/blob/main/Release%20Planning.md);
its [status is annotated](@/archaeology/notebooks.md) because some sections predate this site.

## Publication lanes

| Destination | Boundary | Required setup |
|---|---|---|
| GitHub Release | Successful platform staging; refreshed tag/development boundary | Contents permission; non-PR binary scan/sign setup |
| crates.io | Tagged, enabled publication | Registry token; package/version correctness |
| Workspace crates.io | Tagged `cargo_publish_workspace` | Dependency-ordered publisher; index visibility; no untracked helper checkout |
| Rustdoc Pages | Configured main push after quality gates | Pages source set to Actions; Pages/OIDC permissions |
| Changelog / benchmark docs | Refreshed release boundary | Full history for changelog; benchmark opt-in |
| AUR | Configured push after platform success | Package name, SSH secret, AUR environment |
| Nexus | Configured non-PR path after builds | API key plus platform file group IDs |
| PortMaster | Optional ARM64 native artifact | Compatible sysroot/toolchain and real device evidence; catalog submission is separate |

## A partial release is not a clean slate

GitHub refresh intentionally deletes and recreates the release; `development` also removes
its tag. Do not race later uploads against refresh. Determine whether GitHub, crates.io,
AUR or Nexus already changed before rerunning or publishing manually.

The workspace publisher skips versions already present on crates.io, handles rate limits,
and waits for index visibility. Registry publication is not undone by deleting a GitHub
Release. Read the actual job log before choosing a retry path.

## Shipping evidence

Attach the Actions run URL, relevant artifact identity, verification result and platform
smoke-test result to the plan's completed requirements. Keep a blocker explicit until
its reason is resolved. The site reports recorded evidence; it does not poll service health.

[Release board](@/releases/board.md) · [Pipeline graph](@/releases/pipelines.md) ·
[Incident desk](@/stroggforge/troubleshooting.md)
