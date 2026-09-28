+++
title = "Incident desk"
description = "A release job exploded. Start here."
weight = 1
+++

## The First Five Minutes

Write down the consumer repository, the commit or tag, the Actions run URL, the
StroggForge ref the caller pins, the failing job, the platform and the exact error. Then
check which ref the consumer actually pins. A caller on `@v43` gets v43's behavior, no
matter what these pages say about the current tag.

Open the failing workflow's [contract](@/stroggforge/workflows/_index.md) and find the job.
Its `needs` and `if` tell you whether it failed or was skipped because something upstream
failed first. Fix the first failure, not the loudest one.

## Where to Look

| Symptom | Look at | Do |
|---|---|---|
| Release never started | test, fmt, clippy, audit, msrv | Fix the first failing gate. The `msrv` job must exist even when its steps skip, or every release job that needs it skips too. |
| clang, lld and rustc disagree | `scripts/shared/setup-llvm.sh`, `rustc -vV` | Match rustc's LLVM major. The MSRV job has to pass its own `toolchain`, or it gets stable's LLVM. |
| l3i refuses the build flags | the consumer's `.cargo/config.toml`, L3i's `TOOLCHAIN.md` | clang, lld and `-Clinker-plugin-lto` go together. Cargo does not inherit a dependency's config, so every consumer carries its own. |
| Windows proc macro rejects `-Clinker-plugin-lto` | `CARGO_BUILD_TARGET` in the job log | setup-llvm pins the host triple. If something unset it, target rustflags reach proc macros again. |
| Windows binary "missing" after a green build | `target/<triple>/release` | That is where it went. Corprus Crucible knows; a hand-written step may not. |
| PortMaster binary links but will not run | `readelf -V` on the binary, the device's glibc | The floor is glibc 2.34 now. Devices that were happy with EL8 builds may not be. |
| Android LTO or codegen failure | NDK r27c clang vs the host rustc's LLVM | Host setup succeeding proves nothing about NDK bitcode compatibility. |
| Wrong manifest or binary built | `resolve-build-context.sh`, `binary_names`, `binary_package` | Name the package explicitly instead of trusting directory detection. |
| Library crate not found | `resolve-crate-manifest.sh`, `crate_names` | Use the Cargo `[package].name`. Hyphens and underscores are normalized; typos are not. |
| Archive missing files | `create-release-archive.sh`, `include_files` | Reproduce the staging locally before uploading anything. |
| Signing fails | `sign-release-binary.sh`, `id-token: write` | Without OIDC permission, keyless signing has no identity to sign with. |
| VirusTotal step fails | `VT_API_KEY` | Non-PR binary releases need it, even though the contract marks the secret optional. |
| Release assets vanished | `github-publish` / `release_cleanup` ordering | The refresh deletes and recreates the release. Anything uploaded before it is gone. |
| crates.io 429 or missing dependency | `publish_workspace.py`, the job log | It retries and waits for the index already. Read the log before publishing by hand. |
| Cargo refuses a dirty tree | the `.stroggforge` helper checkout | The workflow copies the publisher to `$RUNNER_TEMP` and removes the checkout first. A custom step has to do the same. |
| Changelog empty or wrong | `fetch-depth` | It needs full history. |
| Benchmark page empty | `generate-benchmark-docs.sh`, Criterion JSON | Check that benchmarks ran before blaming the upload. |
| Pages overwritten | `publish_docs` | rustdoc and a project's own site cannot both own Pages. Pick one. |
| Nexus upload fails | `NEXUS_API_KEY`, `NEXUS_GROUP_IDS` | Check the group ID for that exact OS and architecture. |
| AUR update fails | the AUR environment, package name, SSH key | Usually ownership or key access. |
| Discord says success after a failure | `call-discord-webhook.needs` | Every job that can fail a release belongs in that list. A missing webhook skips on purpose. |
| War room build rejects the TOML | the error message | It names the record and the reference. Fix `war-room/*.toml`, never the generated pages. |

## The Ref Contract

A workflow that calls `.stroggforge/scripts/...` needs a StroggForge checkout containing
that script, and a composite action needs its scripts in the tagged tree. The YAML
validates fine either way. Before blaming the command that failed, check that the tagged
files are there at all.

## Asking for Help

Open the issue in the consumer repository if the problem is specific to it, or in
[StroggForge](https://github.com/DreamWeave-MP/StroggForge/issues) if the shared pipeline
is at fault. Include the first-five-minutes notes, a minimal reproduction, and which
channels already published. crates.io does not un-publish because you deleted a GitHub
Release. Lua and content failures go to St4sh, not the Rust supply line.
