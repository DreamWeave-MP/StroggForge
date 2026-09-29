[Home](Readme.md)  

# StroggForge Actions

This repository provides the shared CI/CD infrastructure for all DreamWeave Rust applications.
The primary entry point for consuming repositories is `rustGlobalBuild.yml`.
Everything else described here is either called internally by that workflow or available for special use cases.

## [./.github/action_templates/daily_quality_template.yaml](./.github/action_templates/daily_quality_template.yaml)

Optional consumer workflow template for daily Rust maintenance checks. It runs `cargo fmt --all --check`, strict workspace Clippy, and `cargo audit` once per day at 9 AM Central Standard Time (`0 15 * * *` UTC), plus manual `workflow_dispatch`.

## [./.github/workflows/rustGlobalBuild.yml](./.github/workflows/rustGlobalBuild.yml)

The full pipeline orchestrator. This is what downstream repositories call — everything else in this document is an implementation detail of it.

Inputs:

1. `binary_names`: Required. JSON array of binary names to build, e.g. `'["my-app"]'`. Add multiple entries for monorepos.
1. `binary_package`: Optional, default empty. Cargo package that provides the binaries when they live in a workspace member other than the root, e.g. `jess-cli`. Release builds then run from the repository root as `cargo build --release --package <binary_package> --bin <binary>`, and `include_files` resolve from the repository root. Set `workspace` to build with `--workspace --bin <binary>` instead, which finds each binary in whichever member defines it. Left empty, the directory detection described under Corprus Crucible applies. With `cargo_publish` and without `cargo_publish_workspace`, a package name here is also what the `crates` job publishes.
1. `include_files`: Optional. Comma-separated files or directories to include in every platform archive, relative to the detected build directory. Defaults to `README.md,LICENSE`.
1. `aur_package_name`: Optional. AUR package name. Omit if the project is not on the AUR.
1. `dependent_repo_names`: Optional. Repositories to notify via issue on tagged releases, one `Owner/Repo` per line. JSON arrays are still accepted for compatibility. When set, requires `DW_BOT_PAT`.
1. `git_username` / `git_email`: Optional. AUR commit identity. Defaults to the DreamWeave maintainer values.
1. `publish_docs`: Optional, default `true`. Set `false` if the project uses its own static site generator for documentation.
1. `cargo_publish`: Optional, default `true`. Runs `cargo publish --dry-run` on every non-tag push, and `cargo publish` on tagged releases. Set `false` if the project does not publish to crates.io. Requires `CARGO_REGISTRY_TOKEN` secret.
1. `cargo_publish_workspace`: Optional, default `false`. With `cargo_publish`, publishes every publishable workspace member instead of one crate per binary name. See [Workspace publishing](#workspace-publishing).
1. `msrv`: Optional, default empty. Minimum supported Rust version, e.g. `'1.88'`, or `auto` to use the highest `rust-version` any workspace member declares. When set, the `lint` job runs `cargo check --workspace --all-features --all-targets` on that toolchain, and publishing waits for it like every other check. Without a `Cargo.lock`, dependencies resolve to versions that still support that Rust version (`CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback`). Empty skips the check.
1. `generate_changelog`: Optional, default `true`. Generates `CHANGELOG.md` from git history and uploads it to the release.
1. `generate_benchmarks`: Optional, default `false`. Runs `cargo bench`, generates `BENCHMARKS.md` from Criterion output when available, otherwise preserves the raw benchmark log, and uploads it to the release.
1. `generate_discord_notification`: Optional, default `true`. Set `false` to skip the Discord notification, e.g. when another workflow in the same run already sends one.
1. `enable_android`: Optional, default `false`. Builds Android ARM64 ELF release artifacts using the Android NDK at API level 23. This does not produce an APK.
1. `enable_portmaster`: Optional, default `false`. Builds Portmaster ARM64 release artifacts for `aarch64-unknown-linux-gnu` cross-compiled with clang and lld in the EL9 Portmaster builder against an EL9 AArch64 sysroot, for the same glibc 2.34 compatibility as `release-linux`.
1. `muxapp_dir`: Optional, default empty. A directory of muOS app files, such as `mux_launch.sh`. Each PortMaster archive is packaged with them as `<binary>-Portmaster-ARM64.muxapp` and published beside it. Requires `enable_portmaster`.
1. `mod_template`: Optional, default `false`. The repository is a DreamWeave Mod Template site: after `publish` and, on tags, the crates.io publish, the `mod-template` job calls `modGlobalBuild.yml` to record the archives in `mod.lock` and deploy the site, in place of rustdoc Pages. A library the site also lists, published with `cargo_publish_workspace`, is recorded from crates.io in the same run. The caller also grants `actions: write`.

The pipeline runs these jobs:

Verification and the release builds start together, so a run takes as long as its slowest platform, not the slowest test plus the slowest build. Nothing is published unless every check and build passed.

- `lint`: `cargo fmt --check`, `cargo audit` (generating a `Cargo.lock` first when the repository does not commit one), the MSRV check when `msrv` is set, and on non-tag pushes a crates.io packaging dry run per binary, or for the workspace with `cargo_publish_workspace`. The dry run is informational: it does not hold back a development build.
- `test`, one job per desktop platform (macOS ARM and Intel, Linux, Windows): pedantic Clippy with every feature, pedantic Clippy with the features that platform's release ships (from `.stroggforge/cargo-build-args.sh`, with the release's package and binary selection), then the tests. Code behind `cfg(windows)` or a feature a release leaves out is only checked where it compiles, so every platform lints.
- Release builds: `release` (macOS ARM and Intel, Windows), `release-linux` (EL9 container, CentOS Stream 9, for glibc 2.34 compatibility), optional `release-android` (Android ARM64 ELF targeting API level 23, not APK), and optional `release-portmaster` (AArch64 GNU/Linux, cross-compiled against an EL9 sysroot) build, sign, scan, package, and stage platform archives as workflow artifacts. Android and PortMaster run the same shipped-feature Clippy on their cross target first, since no test job compiles for them.
- `publish`, once all of the above passed: packs the muOS apps from the PortMaster archives when `muxapp_dir` is set, refreshes the current tag release or the shared `development` release, uploads the archives and VirusTotal notes, generates and uploads the changelog, opens an issue in each dependent repository on tags, and lists the Nexus Mods uploads.
- `nexus`: one job per staged Nexus Mods upload, after `publish`; none when Nexus Mods is not configured.
- `crates` (tags): each binary's crate, or the dependency-ordered workspace with `cargo_publish_workspace`, once everything passed.
- `aur`, `docs` (rustdoc Pages on main pushes, without `mod_template`), `benchmarks` (tags only, after `publish`), and `mod-template`. A pull request has nothing to record, so `site-check` builds its site from the start of the run instead.
- `notify` sends exactly one Discord message per push, after every job has finished. Any failed job turns it into a failure message linking the workflow run; otherwise it links the release and names where it is available (GitHub, crates.io on tagged publishes, the AUR). Cancelled runs, such as ones superseded by a newer push, send nothing.

## [./.github/workflows/modGlobalBuild.yml](./.github/workflows/modGlobalBuild.yml)

The DreamWeave Mod Template pipeline. Sites of mods call it directly from their `build_site.yml` (template: [mod_template.yaml](./.github/action_templates/mod_template.yaml)); `rustGlobalBuild` and `libGlobalBuild` call it when their caller sets `mod_template: true`. It runs the site's own `./buildSite`.

Inputs:

1. `rust_release`: Optional, default `false`. The calling Rust workflow built and published this run's release: record it in `mod.lock`, publish nothing, and leave GitHub releases alone.
1. `rust_archives`: Optional, default `false`. The calling workflow staged program archives as `github-release-*` artifacts: download them into `dist/binaries/` to record.

The pipeline runs these jobs:

- `site` (branches and pull requests): `./buildSite check` and the template's own tests, then the development build, protocol documents, schemas, the site and its links. On the default branch it also records what crates.io and GitHub have published in `mod.lock` through `scripts/mod-template/commit-records.sh`, signs and replaces the development release (mods only, drafts first), and uploads the Pages artifact.
- `deploy`: GitHub Pages, from the default branch.
- `release` (tags): `./buildSite check` and the tests, then builds the tagged release (a mod archive, or the staged program archives), signs mod archives with Sigstore when asked, records the release in `mod.lock` on the default branch, publishes a mod's GitHub release (drafts first), and starts a default-branch run of the calling workflow, because `GITHUB_TOKEN` pushes start no runs.
- `nexus-upload` (mods only): Nexus Mods, one job per file, for projects with a file group.

## [./.github/workflows/libGlobalBuild.yml](./.github/workflows/libGlobalBuild.yml)

The library equivalent of `rustGlobalBuild.yml`. Use this for crates that have no distributable binary. It has the same `lint` job, and a `test` job per desktop platform that runs pedantic Clippy with every feature and with the default features a dependent gets, then the tests. `publish` refreshes the GitHub Release, uploads the changelog and on tags opens dependent issues; `crates` publishes on tags; `docs`, `benchmarks` (tags only) and `mod-template` follow, and a pull request runs `site-check` from the start. There are no release builds and no AUR publishing.

Inputs:

1. `crate_names`: Required. JSON array of crate names to publish, e.g. `'["my-lib"]'`. Used to locate each crate's `Cargo.toml` via `cargo metadata` (hyphens and underscores are treated as equivalent).
1. `dependent_repo_names`: Optional. Repositories to notify via issue on tagged releases, one `Owner/Repo` per line. JSON arrays are still accepted for compatibility. When set, requires `DW_BOT_PAT`.
1. `publish_docs`: Optional, default `true`. Set `false` if using a custom SSG.
1. `cargo_publish`: Optional, default `true`. Dry-run on non-tag pushes; real publish on tagged releases. Requires `CARGO_REGISTRY_TOKEN` secret.
1. `cargo_publish_workspace`: Optional, default `false`. With `cargo_publish`, publishes every publishable workspace member in dependency order instead of the unordered one-job-per-crate matrix over `crate_names`. `crate_names` is then only used for the docs index redirect. See [Workspace publishing](#workspace-publishing).
1. `msrv`: Optional, default empty. Same as the application workflow: checks the workspace on the given Rust version (or `auto`) and holds back publishing and docs.
1. `generate_changelog`: Optional, default `true`.
1. `generate_benchmarks`: Optional, default `false`.
1. `generate_discord_notification`: Optional, default `true`. Set `false` when the same repository also calls `rustGlobalBuild.yml` in one workflow, so each release sends one notification.
1. `mod_template`: Optional, default `false`. The repository is a DreamWeave Mod Template site: the `mod-template` job calls `modGlobalBuild.yml`, after `publish` and, on tags, the crates.io publish, to record each version crates.io has in `mod.lock` and deploy the site, in place of rustdoc Pages. The caller also grants `actions: write`.

The library Discord notification follows the same rules as the application one: one message after every job has finished, reporting any failure.

## LLVM Toolchain

Every DreamWeave crate builds with clang and lld whose LLVM matches rustc's (22.1.8 at the time of writing). It is not optional. Every job that compiles (`lint`, `test`, `crates`, `docs`, `benchmarks`, and every release build through Corprus Crucible) runs [`scripts/shared/setup-llvm.sh`](./scripts/shared/setup-llvm.sh) first, through the `rust-setup` action or Corprus Crucible, which fails the job if it cannot put a matching clang and lld on `PATH`. The MSRV check matches the MSRV toolchain's LLVM, not stable's. The EL9 release builders (`release-linux`, and `release-portmaster` cross-compiling to AArch64) carry the stream's clang and lld, and the same check fails clearly if the stream's LLVM major and rustc's ever differ.

On Apple targets a crate's `.cargo/config.toml` links with clang and lld but leaves out `-Clinker-plugin-lto`: rustc passes that flag's GNU `-plugin-opt` arguments to the linker, and `ld64.lld` rejects them.

StroggForge supplies the compilers; each repository decides how its build uses them, in its own `.cargo/config.toml` (Cargo does not inherit a dependency's). l3i's `TOOLCHAIN.md` has the `rustflags` and `CXX` lines its dependents carry.

## Submodules

Every checkout of the consuming repository in both workflows passes `submodules: true`, so crates that vendor a dependency as a git submodule (l3i carries Luau that way) build in every job. Repositories without submodules are unaffected.

## Workspace publishing

`cargo_publish_workspace: true` (both workflows) publishes the workspace instead of one crate per name:

- The `lint` job's dry run becomes `cargo publish --workspace --dry-run` on non-tag pushes, which packages and verifies every member against the others even before any of them is on crates.io. Informational, like the per-crate dry run.
- The `crates` job runs [`.github/scripts/publish_workspace.py`](./.github/scripts/publish_workspace.py) on tagged releases: after the release builds for applications, after the checks for libraries. It publishes one crate at a time in dependency order read from `cargo metadata`, so a crate is only uploaded once everything it depends on is on crates.io.

What the publish script does:

- Leaves out members with `publish = false`, or a `publish` list without `crates-io`. That is the way to keep a member off crates.io; there is no separate exclude input.
- Orders by normal, build, and versioned dev-dependencies. Path-only dev-dependencies are ignored because Cargo strips them when packaging; a cycle through versioned dependencies fails the job with the cycle printed.
- Skips any crate whose version is already in the crates.io index, so a run that failed halfway can simply be rerun.
- Retries on crates.io 429 responses, waiting until the time crates.io names in the error (ten minutes if it names none), up to 30 attempts per crate. crates.io lets an account publish a small burst of new crates and then roughly one new crate per ten minutes, so a first release of a large workspace can take a while; the job has a 240-minute timeout and never runs two workspace publishes for the same repository at once.
- Waits for each uploaded crate to appear in the crates.io index before building the next one.

Run it locally from a workspace root to see the plan: `python3 .github/scripts/publish_workspace.py --list` prints the order without touching the network, and `--dry-run` also checks the crates.io index and reports which crates would be uploaded.

## [./.github/actions/corprus-crucible/action.yml](./.github/actions/corprus-crucible/action.yml)

Composite action used internally by `rustGlobalBuild.yml`. Handles the release artifact pipeline for a single binary on a single platform. Called once per OS per binary name.

Inputs:

1. `binary_name`: Required. The executable name to build, without platform extension.
1. `include_files`: Optional. Comma-separated list of additional files or directories to include in the release zip. Paths are relative to the build directory. Defaults to `README.md,LICENSE`. Included `README.md`/`Readme.md` and `LICENSE` files are archived as `{binary}-README.md` and `{binary}-LICENSE` so multiple application archives can be unpacked into the same directory without their docs trampling each other. Directories are copied recursively with their relative paths preserved; files inside subdirectories (e.g. `libs/lua/jess.d.luau`) land at the archive root under their own name. Other license files such as `LICENSE-MIT` and `LICENSE-APACHE` keep their names, so dual-licensed projects list both. File-name matching falls back to case-insensitive lookup, so existing consumers that pass `Readme.md` still package a conventional `README.md` on Linux.
1. `vt_api_key`: Required for non-PR release builds. VirusTotal API key.
1. `release_name`: Required. Caller-supplied release identifier — either the tag name or `development`.
1. `nexus_api_key`: Optional. Nexus Mods API key. Provide with `nexus_group_ids` to upload release archives to Nexus Mods. Passed through the environment so JSON secrets are not damaged by shell quoting.
1. `nexus_group_ids`: Optional. Nexus Mods file group IDs as JSON. Provide with `nexus_api_key` to upload release archives to Nexus Mods. Values may be strings or integers; booleans are rejected so `false` cannot accidentally become a file group ID.
1. `platform_os` / `platform_arch`: Optional. Override artifact platform naming for cross builds. Native builds default to the runner OS and architecture.
1. `rust_target`: Optional. Rust target triple for cross-compiled release builds, e.g. `aarch64-linux-android`.
1. `cargo_package`: Optional. Cargo package that provides the binary. When set, the action builds from `.` with `--package <cargo_package> --bin <binary_name>`, or with `--workspace --bin <binary_name>` when set to `workspace`. `rustGlobalBuild.yml` passes its `binary_package` input here.

Build context detection: with no `cargo_package`, if `binary_name` matches a directory at the repo root, the action builds from that directory. Otherwise builds from `.`. This handles monorepos transparently. When the binary lives in a workspace member whose directory is not named after it (say `libs/cli` building `jess`), set `cargo_package` instead.

Registry and target caches are keyed on `Cargo.lock`, or on the `Cargo.toml` files when the repository does not commit a lockfile, so lockfile-less workspaces still get fresh caches when their manifests change.

Release builds may customize Cargo feature policy by adding an executable `.stroggforge/cargo-build-args.sh` script to the consuming repository. This is a fixed convention, not another workflow input pretending to be useful. If the file exists, Corprus Crucible calls it before `cargo build` as:

```bash
.stroggforge/cargo-build-args.sh "$platform_os" "$platform_arch" "$rust_target" "$binary_name"
```

The hook is deliberately narrow: it may only choose Cargo feature flags. Think of it as a small platform policy script, not a way to replace the release build command. Its stdout is parsed as Cargo arguments, so it must print one extra Cargo feature argument per line and send any diagnostics to stderr. Blank lines are ignored; shell quoting is not interpreted. Only `--features`, `-F`, `--no-default-features`, and `--all-features` are accepted; everything else fails the build. For `--features` or `-F`, print the feature list on the next line, or use `--features=gui` / `-F=gui`. Corprus Crucible still owns `--release`, `--target`, `--target-dir`, `--manifest-path`, package selection, binary selection, and the expected binary path. With `cargo_package` set, feature names are resolved against the selected package; with `workspace`, use the `package/feature` form so it is unambiguous which member a feature belongs to. Use `scripts/cargo-build-args.example.sh` as a starting point; it demonstrates desktop builds using the `gui` feature and Android/Portmaster builds using `--no-default-features`.

Stable platform tuples currently passed to the hook are `macOS-ARM64`, `macOS-x64`, `Windows-x64`, `Linux-x64`, `Android-ARM64`, and `Portmaster-ARM64`. Native desktop builds pass an empty Rust target; Android passes `aarch64-linux-android`; Portmaster passes `aarch64-unknown-linux-gnu`.

On pull requests, signing, VirusTotal scanning, Nexus Mods artifact staging, and GitHub Release artifact staging are skipped; the binary is uploaded as a workflow artifact instead. On release builds, Corprus Crucible stages GitHub Release archives as workflow artifacts; `rustGlobalBuild.yml` publishes them later from its `publish` job after that job refreshes the release. Pull requests restore the default branch's caches, so they do not compile everything cold.

Nexus Mods upload is enabled by setting both `NEXUS_API_KEY` and `NEXUS_GROUP_IDS` secrets on the consuming repository or organization. `NEXUS_GROUP_IDS` is a JSON object keyed by `{platform}-{channel}`; use `.github/nexus_group_ids.template.json` as the template. Supported platform keys are `linux-x64`, `windows-x64`, `macos-x64`, `macos-arm64`, `android-arm64`, and `portmaster-arm64`, with `stable` for tagged releases and `dev` for the `development` release. Stable keys are required for tagged releases for every enabled release platform; optional platform keys such as `android-arm64` and `portmaster-arm64` are only required when those builds are enabled. Development keys are optional; missing development keys skip Nexus upload for that platform. Each platform archive is copied to a Nexus-specific filename of `{binary}-{platform}-{release}.zip`, uploaded with that display name, and uses the release name as the Nexus version. Development builds set `archive_existing_file` so the previous development upload for that file group is archived. The Nexus file description includes BBCode-formatted VirusTotal analysis links generated earlier in the release pipeline; GitHub Release notes keep the Markdown version.

Corprus Crucible shell implementation details live under `scripts/corprus-crucible/`. The composite action owns GitHub Actions orchestration; the scripts own validation, build context detection, binary suffix detection, release binary staging, signing, archive creation, VirusTotal link formatting, GitHub Release artifact staging, and Nexus Mods archive preparation.

## [./.github/actions/setup-llvm/action.yml](./.github/actions/setup-llvm/action.yml)

Composite action that installs clang and lld whose LLVM major matches a Rust toolchain's LLVM, then verifies that the `clang` and the `ld.lld` (or `lld-link`) on `PATH` really are that major. StroggForge's own jobs run its script through the `rust-setup` action and Corprus Crucible; the action is kept for callers of their own (see [LLVM Toolchain](#llvm-toolchain)). l3i's build script refuses to build unless clang++ compiles the C++ side, rustc links through clang and lld with `-Clinker-plugin-lto`, and both share an LLVM major (see l3i's `TOOLCHAIN.md`). Cargo does not inherit a dependency's `.cargo/config.toml`, so every repository carries the `rustflags` and `CXX` lines itself; this action only supplies the compilers.

Inputs:

1. `toolchain`: Optional. The rustup toolchain whose LLVM major to match, e.g. `1.88`. Empty uses the default `rustc`. The MSRV check passes the resolved MSRV toolchain.
1. `llvm_version`: Optional. A full LLVM release to install instead of the one rustc reports, e.g. `22.1.8`.

Outputs: `llvm_major`, `llvm_version`.

How it installs, per runner OS (all in [`scripts/shared/setup-llvm.sh`](./scripts/shared/setup-llvm.sh)):

- Linux with apt: `apt.llvm.org`'s `llvm.sh <major>` plus `clang-<major>` and `lld-<major>`, then unsuffixed `clang`, `clang++`, `ld.lld`, `lld`, `lld-link`, and `llvm-*` links in `/usr/local/bin`.
- Linux with dnf (the EL9 builder image): `dnf install clang lld` from the stream's AppStream, which tracks current LLVM releases.
- Linux with neither: the official `LLVM-<version>-Linux-X64.tar.xz` release, added to `PATH`. Those binaries need glibc 2.34 and GCC 12's libstdc++ (Ubuntu 22.04 or Debian 12; no EL release qualifies).
- macOS: Homebrew `llvm@<major>` and `lld@<major>`, or `llvm` and `lld` when no versioned formula exists, with both `bin` directories on `PATH`. Homebrew ships lld as a separate formula; it provides the `ld64.lld` that `clang -fuse-ld=lld` needs for Mach-O. Apple's own clang is never used: its version numbers are not LLVM's.
- Windows: the official `clang+llvm-<version>-<arch>-pc-windows-msvc.tar.xz` release, extracted with 7-Zip and added to `PATH`. Dependents build their C++ with `clang-cl` and link with `lld-link` there. The script also sets `CARGO_BUILD_TARGET` to the host triple for the rest of the job: without an explicit target, Cargo applies `[target]` rustflags to build scripts and proc macros, and rustc refuses `-Clinker-plugin-lto` on MSVC proc macros, which Cargo builds with `-Cprefer-dynamic`. Build output lands under `target/<triple>/` as a result, and Corprus Crucible looks for the release binary there.

When rustc's exact LLVM version has no release (a snapshot), the newest release of the same major is used; only the major has to match.

## [./.github/workflows/createRelease.yml](./.github/workflows/createRelease.yml)

Reusable workflow that refreshes the current tag release, or the shared `development` release on non-tag pushes. StroggForge's own workflows no longer call it: `rustGlobalBuild.yml` and `libGlobalBuild.yml` run the same script in their `publish` jobs. It is kept for callers of their own.

Output: `release_name` — the tag name on tag pushes, `development` otherwise.

Requires access to the GitHub token; call with `secrets: inherit`.

## [./.github/workflows/discord.yml](./.github/workflows/discord.yml)

Reusable workflow that posts an embed to a Discord channel. Generally called `if: always()` to surface pipeline failures.

Inputs:

1. `avatar_url`: Optional. Defaults to the DreamWeave logo.
1. `title`: Optional. Defaults to `{repo} has been updated.`
1. `description`: Optional. Defaults to a link to the latest release.
1. `footer_text`: Optional. Defaults to workflow ID, triggering actor, and timestamp.

Secrets:

1. `WEBHOOK_URL`: Optional. If omitted, the workflow logs a skip and sends no Discord notification. Callers usually pass `DISCORD_CHANNEL_WEBHOOK || DW_TOOLS_CHANNEL_WEBHOOK`.

## [./.github/workflows/dependent.yml](./.github/workflows/dependent.yml)

Reusable workflow that opens dependency update issues in downstream repositories. It accepts the same newline-separated repository list as the public workflows and fans out internally, attempting every repository before reporting aggregate failure.

Inputs:

1. `aur_package_name`: Optional. Used by application releases to render an AUR package link in the issue body. Omit for libraries or applications that are not published to the AUR.
1. `dependent_repo_names`: Optional. Repositories to notify, one `Owner/Repo` per line. JSON arrays are still accepted for compatibility.

Secrets:

1. `DW_BOT_PAT`: Required. Use the org-level PAT created for this purpose.

Created issues use the default `enhancement` label when the target repository has it; otherwise the issue is created without labels. Custom DreamWeave-only labels are not assumed to exist in downstream repositories, because that would be optimistic in the way YAML usually punishes.

## [./.github/scripts/gen_benchmarks.py](./.github/scripts/gen_benchmarks.py)

Python script used by `scripts/shared/generate-benchmark-docs.sh`. Reads Criterion output from `target/criterion/**/new/{benchmark,estimates}.json` and writes `BENCHMARKS.md` with summary tables and Mermaid bar charts. If the repository uses a custom benchmark harness that does not create Criterion JSON, it falls back to `benchmark-output.txt`.

Can also be run locally after `cargo bench`:

```
python3 /path/to/StroggForge/.github/scripts/gen_benchmarks.py
```

## [./scripts/shared/generate-benchmark-docs.sh](./scripts/shared/generate-benchmark-docs.sh)

Shared shell script used by both application and library benchmark jobs. Runs `cargo bench`, preserves the raw log in `benchmark-output.txt`, then runs the Python benchmark documentation generator to create `BENCHMARKS.md`.

## [./scripts/shared/setup-llvm.sh](./scripts/shared/setup-llvm.sh)

Shared shell script behind the `setup-llvm` action and the LLVM step of every Corprus Crucible build. Takes an optional rustup toolchain name and an optional LLVM version override, reads `rustc -vV` for the LLVM version, installs a matching clang and lld for the runner OS, pins `CARGO_BUILD_TARGET` on Windows, verifies the result, and writes `llvm_major`/`llvm_version` to `GITHUB_OUTPUT`.

## [./scripts/shared/changelog.sh](./scripts/shared/changelog.sh)

Shared shell script used by both application and library workflows to generate `CHANGELOG.md` from git history. The workflows still own checkout and release upload; the script only owns changelog content generation.

## [./scripts/shared/resolve-crate-manifest.sh](./scripts/shared/resolve-crate-manifest.sh)

Shared shell script used by library publishing jobs. Given a crate name, it uses `cargo metadata` to find the matching `Cargo.toml`, treating hyphens and underscores as equivalent.

## [./scripts/shared/docs-index.sh](./scripts/shared/docs-index.sh)

Shared shell script used by both application and library docs jobs. Given the workflow's JSON array of binary or crate names, it creates the GitHub Pages `target/doc/index.html` redirect to the first generated rustdoc package path.

## [./scripts/shared/create-dependent-update-issues.sh](./scripts/shared/create-dependent-update-issues.sh)

Shared shell script used by `dependent.yml` to parse newline-separated or JSON-array dependent repository lists and call `create-dependent-update-issue.sh` for each repository. It attempts every target and exits non-zero after the loop if any notification failed.

## [./scripts/shared/create-dependent-update-issue.sh](./scripts/shared/create-dependent-update-issue.sh)

Shared shell script used by `create-dependent-update-issues.sh` to create one dependency update issue. It renders the AUR link only when an AUR package name is provided and applies the `enhancement` label only when the target repository has that label.

## [./.github/action_templates/rust_template.yaml](./.github/action_templates/rust_template.yaml)

Workflow template for new Rust binary repositories. Copy it to `.github/workflows/build.yml` in the target repo and replace `ENTER_BINARY_NAME_HERE` with the binary name. Uncomment optional inputs as needed.

## [./.github/action_templates/lib_template.yaml](./.github/action_templates/lib_template.yaml)

Workflow template for library crates. Copy it to `.github/workflows/build.yml` and replace `ENTER_CRATE_NAME_HERE` with the crate name from `Cargo.toml`. Uncomment optional inputs as needed.
