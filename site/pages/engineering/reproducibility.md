+++
title = "Reproducibility"
description = "What this site guarantees, and what a release build honestly cannot."
weight = 30
+++

## This Site

The generator reads the committed TOML and this checkout's workflows, has a locked
dependency set and makes no network requests. Same inputs, same pages. Mermaid and Zola
are pinned. CI runs the same [loop](@/contributing/maintenance.md) you run locally.

## Release Builds

Release builds are a different story. Stable Rust moves, hosted runner images move,
CentOS Stream packages move, upstream actions move. Matching the LLVM major is a
compatibility policy, not a patch-level pin. StroggForge does not promise byte-identical
binaries, and this site does not pretend it does.

To reproduce a release failure, collect:

1. The consumer's commit or tag and its `Cargo.lock`.
2. The exact StroggForge workflow, action and helper refs, plus the builder image.
3. `rustc -vV`, the clang and lld versions, and the relevant Cargo target flags and sysroot.
4. Runner, target triple, glibc or deployment floor, and enabled features.
5. The build log, artifact hash and verification bundle.

A declared MSRV is not a passing MSRV job. A configured PortMaster target is not a binary
that ran on a handheld. Record both separately in the plan.
