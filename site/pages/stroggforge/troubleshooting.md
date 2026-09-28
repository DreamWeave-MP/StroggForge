+++
title = "Incident desk"
description = "When the supply line explodes, start with the failing stage and its evidence."
weight = 1
+++

## First five minutes

Record the consumer repository, Git SHA/tag, Actions run URL, StroggForge caller ref,
failing job, platform and exact error. Check the [consumer ledger](@/stroggforge/integration.md):
an older pinned ref may behave differently from this checkout's docs. Open the generated
[workflow contract](@/stroggforge/workflows/_index.md) and find the job's `needs` and `if`.

## Route by failure

| Symptom | Inspect | Next action |
|---|---|---|
| Release never started | test/fmt/clippy/audit/msrv results and release conditions | Fix the first failing prerequisite. Keep the MSRV job present even if its steps skip. |
| clang/rustc/lld mismatch | `scripts/shared/setup-llvm.sh`, `rustc -vV`, clang/lld output | Match the active compiler's LLVM major. For MSRV pass its toolchain explicitly. |
| l3i rejects build flags | Consumer `.cargo/config.toml`, target RUSTFLAGS, L3i TOOLCHAIN.md | Supply clang, lld, linker-plugin-lto together. Cargo does not inherit dependency config. |
| PortMaster links but will not run | EL9 sysroot, `readelf` dynamic version requirements, device runtime | Check glibc 2.34/device compatibility; do not assume old EL8 devices still work. |
| Android LTO/codegen failure | NDK r27c target compiler vs host rustc LLVM | Resolve bitcode/toolchain mismatch; host setup success does not validate NDK compatibility. |
| Wrong Cargo manifest or binary | `resolve-build-context.sh`, `binary_names`, `binary_package` | Select a root package/bin explicitly, or verify directory detection. |
| Library name not found | `resolve-crate-manifest.sh`, `crate_names` | Match Cargo `[package].name`; inspect hyphen/underscore normalization. |
| Missing archive content | `create-release-archive.sh`, `include_files`, staged build/dist paths | Reproduce staging before uploading. Verify binary, bundle and included files. |
| Signing fails | `sign-release-binary.sh`, OIDC permission, workflow identity | Verify `id-token: write`; preserve the signed binary/bundle pairing. |
| Scan fails | VT_API_KEY, VirusTotal action logs | Non-PR binary releases require a valid scan token. |
| Release assets disappear | `github-publish` / `release_cleanup` ordering | Refresh deletes the previous boundary; upload only afterward. |
| crates.io 429 or missing prerequisite | `publish_workspace.py`, index visibility, registry token | Inspect retries/waits and already-published versions before manual intervention. |
| Cargo refuses dirty publish tree | Temporary `.stroggforge` helper checkout | Copy publisher to RUNNER_TEMP and remove helper checkout before publishing, as the workflow does. |
| Changelog missing/history wrong | Full-history checkout; refreshed release boundary | Restore `fetch-depth: 0`; inspect changelog upload condition. |
| Benchmark page empty | `generate-benchmark-docs.sh`, Criterion `new` JSON / raw log | Confirm bench execution and generator input, not just the upload step. |
| Pages wrong or overwritten | Consumer publish_docs flag; Pages deploy jobs | Choose rustdoc or project SSG as the site's owner; enable Actions as Pages source. |
| Nexus upload fails | Secret pair, platform group IDs, staged upload artifact | Validate the group mapping for that exact OS/architecture. |
| AUR update fails | AUR environment, package name, SSH key | Check package ownership/access and publish-action log. |
| Notification claims success after failure | `call-discord-webhook.needs` | Include every failure-bearing job; missing webhook intentionally skips. |
| War-room metadata rejected | Error's source/project/plan ID | Fix the canonical record, never patch generated Markdown. |

## Ref-contract failures

A workflow calling `.stroggforge/scripts/...` needs a checkout ref containing that helper.
Composite actions also need their scripts in the tagged action tree. YAML can validate
while this contract is broken. Inspect the actual tagged files and consumer pin before
changing unrelated commands.

## Escalation packet

Open an issue in the failing consumer for project-specific behavior, or
[StroggForge](https://github.com/DreamWeave-MP/StroggForge/issues) for shared pipeline behavior.
Include the first-five-minutes evidence, a minimal reproduction, expected artifact and
which distribution channels already changed. Link the relevant campaign/release requirement.
Use Cod3x/St4sh's source paths for Lua/content failures rather than assigning every ecosystem
problem to the Rust supply line.
