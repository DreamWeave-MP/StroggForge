+++
title = "Reproducibility"
description = "Deterministic site generation and honest limits on moving build inputs."
weight = 30
+++

## This site

The Rust generator has a committed Cargo lockfile and no runtime network client. It reads
committed canonical records and source snapshots, sorts output deterministically, and
emits no wall-clock timestamp or machine-absolute path. Reviewed dates are data.

Generated content is **build-only**. CI generates it twice and compares the complete
Markdown/data/Mermaid file set before rendering. Pinned Mermaid CLI and Zola versions
produce SVG/HTML. A contributor runs the same [local loop](@/contributing/maintenance.md).

The first tool/dependency installation needs network access. Subsequent Rust generation
can use `--offline --locked`; site content rendering does not consult GitHub or sibling clones.

## Consumer release builds

Stable Rust, hosted runner labels, Stream packages and upstream action refs may move.
Matching LLVM major is a compatibility policy, not a reproducible compiler patch pin.
The matrix makes these distinctions visible instead of promising byte-identical binaries.

When reproducing a release failure, record:

1. Consumer commit/tag and Cargo.lock.
2. Exact reusable workflow/action/helper refs and builder image.
3. `rustc -vV`, clang and lld versions; relevant Cargo target flags and sysroot.
4. Platform runner, target triple, glibc/deployment floor and feature selection.
5. Build logs, artifact hash and verification bundle identity.

An MSRV declaration is separate from a successful MSRV run. A configured PortMaster target
is separate from a device test. Keep those receipts in the plan before marking work done.
