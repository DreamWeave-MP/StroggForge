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
1. `aur_package_name`: Optional. AUR package name. Omit if the project is not on the AUR.
1. `dependent_repo_names`: Optional. Repositories to notify via issue on tagged releases, one `Owner/Repo` per line. JSON arrays are still accepted for compatibility. When set, requires `DW_BOT_PAT`.
1. `git_username` / `git_email`: Optional. AUR commit identity. Defaults to the DreamWeave maintainer values.
1. `publish_docs`: Optional, default `true`. Set `false` if the project uses its own static site generator for documentation.
1. `cargo_publish`: Optional, default `true`. Runs `cargo publish --dry-run` on every non-tag push, and `cargo publish` on tagged releases. Set `false` if the project does not publish to crates.io. Requires `CARGO_REGISTRY_TOKEN` secret.
1. `generate_changelog`: Optional, default `true`. Generates `CHANGELOG.md` from git history and uploads it to the release.
1. `generate_benchmarks`: Optional, default `false`. Runs `cargo bench`, generates `BENCHMARKS.md` from Criterion output when available, otherwise preserves the raw benchmark log, and uploads it to the release.
1. `enable_android`: Optional, default `false`. Builds Android ARM64 ELF release artifacts using the Android NDK at API level 23. This does not produce an APK.
1. `enable_portmaster`: Optional, default `false`. Builds Portmaster ARM64 release artifacts for `aarch64-unknown-linux-gnu` using an AlmaLinux 8 AArch64 sysroot for old glibc compatibility.

The pipeline runs these jobs:

- Quality gates (parallel, block release): `test` (full platform matrix), `fmt`, `clippy` (pedantic), `audit` (RustSec)
- Informational (parallel, does not block): `cargo-publish-dry-run`
- Release builds (after gates pass): `release` (macOS ARM + Intel, Windows), `release-linux` (AlmaLinux 8 container for glibc 2.28 compatibility), optional `release-android` (Android ARM64 ELF targeting API level 23, not APK), and optional `release-portmaster` (AArch64 GNU/Linux with an AlmaLinux 8 sysroot) build, sign, scan, package, and stage platform archives as workflow artifacts. They do not mutate the GitHub Release directly.
- Release preparation: `release_cleanup` runs after the application release builds succeed and refreshes the current tag or shared `development` release.
- GitHub Release publish: `github-publish` uploads the staged platform archives and VirusTotal notes after `release_cleanup` succeeds.
- Doc/artifact generation: `docs` deploys GitHub Pages on main pushes after gates pass; `changelog` and `benchmarks` upload release files after `github-publish` succeeds.
- External publish/notification: `cargo-publish` (crates.io, tag only), `aur-publish`, and `nexus-publish` fan out after builds; `call-discord-webhook` waits for the mandatory release path, changelog, and optional external publish jobs so it can report their failures, while `nag-dependents` waits for the GitHub Release publish boundary.

## [./.github/workflows/libGlobalBuild.yml](./.github/workflows/libGlobalBuild.yml)

The library equivalent of `rustGlobalBuild.yml`. Use this for crates that have no distributable binary — it runs all the same quality gates, publishing, docs, changelog, and benchmarks, but has no `corprus-crucible` release build jobs and no AUR publishing.

Library release cleanup runs after the mandatory quality gates pass. Changelog and benchmark release files upload after release cleanup succeeds; dependent repository notifications wait for that GitHub Release boundary.

Inputs:

1. `crate_names`: Required. JSON array of crate names to publish, e.g. `'["my-lib"]'`. Used to locate each crate's `Cargo.toml` via `cargo metadata` (hyphens and underscores are treated as equivalent).
1. `dependent_repo_names`: Optional. Repositories to notify via issue on tagged releases, one `Owner/Repo` per line. JSON arrays are still accepted for compatibility. When set, requires `DW_BOT_PAT`.
1. `publish_docs`: Optional, default `true`. Set `false` if using a custom SSG.
1. `cargo_publish`: Optional, default `true`. Dry-run on non-tag pushes; real publish on tagged releases. Requires `CARGO_REGISTRY_TOKEN` secret.
1. `generate_changelog`: Optional, default `true`.
1. `generate_benchmarks`: Optional, default `false`.

## [./.github/actions/corprus-crucible/action.yml](./.github/actions/corprus-crucible/action.yml)

Composite action used internally by `rustGlobalBuild.yml`. Handles the release artifact pipeline for a single binary on a single platform. Called once per OS per binary name.

Inputs:

1. `binary_name`: Required. The executable name to build, without platform extension.
1. `include_files`: Optional. Comma-separated list of additional files to include in the release zip. Paths are relative to the build directory. Defaults to `Readme.md,LICENSE`. Included `README.md`/`Readme.md` and `LICENSE` files are archived as `{binary}-README.md` and `{binary}-LICENSE` so multiple application archives can be unpacked into the same directory without their docs trampling each other.
1. `vt_api_key`: Required for non-PR release builds. VirusTotal API key.
1. `release_name`: Required. Caller-supplied release identifier — either the tag name or `development`.
1. `nexus_api_key`: Optional. Nexus Mods API key. Provide with `nexus_group_ids` to upload release archives to Nexus Mods. Passed through the environment so JSON secrets are not damaged by shell quoting.
1. `nexus_group_ids`: Optional. Nexus Mods file group IDs as JSON. Provide with `nexus_api_key` to upload release archives to Nexus Mods. Values may be strings or integers; booleans are rejected so `false` cannot accidentally become a file group ID.
1. `platform_os` / `platform_arch`: Optional. Override artifact platform naming for cross builds. Native builds default to the runner OS and architecture.
1. `rust_target`: Optional. Rust target triple for cross-compiled release builds, e.g. `aarch64-linux-android`.

Build context detection: if `binary_name` matches a directory at the repo root, the action builds from that directory. Otherwise builds from `.`. This handles monorepos transparently.

Release builds may customize Cargo feature policy by adding an executable `.stroggforge/cargo-build-args.sh` script to the consuming repository. This is a fixed convention, not another workflow input pretending to be useful. If the file exists, Corprus Crucible calls it before `cargo build` as:

```bash
.stroggforge/cargo-build-args.sh "$platform_os" "$platform_arch" "$rust_target" "$binary_name"
```

The script must print one extra Cargo feature argument per line. Blank lines are ignored; shell quoting is not interpreted. Only `--features`, `-F`, `--no-default-features`, and `--all-features` are accepted; everything else fails the build. Corprus Crucible still owns `--release`, `--target`, `--target-dir`, `--manifest-path`, package selection, binary selection, and the expected binary path. Use `scripts/cargo-build-args.example.sh` as a starting point; it demonstrates desktop builds using the `gui` feature and Android/Portmaster builds using `--no-default-features`.

Stable platform tuples currently passed to the hook are `macOS-ARM64`, `macOS-Intel`, `Windows-x64`, `Linux-x64`, `Android-ARM64`, and `Portmaster-ARM64`. Native desktop builds pass an empty Rust target; Android passes `aarch64-linux-android`; Portmaster passes `aarch64-unknown-linux-gnu`.

On pull requests, signing, VirusTotal scanning, Nexus Mods artifact staging, and GitHub Release artifact staging are skipped; the binary is uploaded as a workflow artifact instead. On release builds, Corprus Crucible stages GitHub Release archives as workflow artifacts; `rustGlobalBuild.yml` publishes them later after release cleanup succeeds.

Nexus Mods upload is enabled by setting both `NEXUS_API_KEY` and `NEXUS_GROUP_IDS` secrets on the consuming repository or organization. `NEXUS_GROUP_IDS` is a JSON object keyed by `{platform}-{channel}`; use `.github/nexus_group_ids.template.json` as the template. Supported platform keys are `linux-x64`, `windows-x64`, `macos-x64`, `macos-arm64`, `android-arm64`, and `portmaster-arm64`, with `stable` for tagged releases and `dev` for the `development` release. Stable keys are required for tagged releases for every enabled release platform; optional platform keys such as `android-arm64` and `portmaster-arm64` are only required when those builds are enabled. Development keys are optional; missing development keys skip Nexus upload for that platform. Each platform archive is copied to a Nexus-specific filename of `{binary}-{platform}-{release}.zip`, uploaded with that display name, and uses the release name as the Nexus version. Development builds set `archive_existing_file` so the previous development upload for that file group is archived. The Nexus file description includes BBCode-formatted VirusTotal analysis links generated earlier in the release pipeline; GitHub Release notes keep the Markdown version.

Corprus Crucible shell implementation details live under `scripts/corprus-crucible/`. The composite action owns GitHub Actions orchestration; the scripts own validation, build context detection, binary suffix detection, release binary staging, signing, archive creation, VirusTotal link formatting, GitHub Release artifact staging, and Nexus Mods archive preparation.

## [./.github/workflows/createRelease.yml](./.github/workflows/createRelease.yml)

Reusable workflow used by release-producing jobs after their mandatory prerequisites pass. Deletes any existing release matching the current tag (or `development` on non-tag pushes), then creates a fresh one with auto-generated changelog and a workflow run ID marker.

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

1. `WEBHOOK_URL`: Required. Use the org-level secret unless there is a specific reason not to.

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
