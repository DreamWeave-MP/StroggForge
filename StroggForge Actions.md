[Home](Readme.md)  

# StroggForge Actions

This repository provides the shared CI/CD infrastructure for all DreamWeave Rust applications.
The primary entry point for consuming repositories is `rustGlobalBuild.yml`.
Everything else described here is either called internally by that workflow or available for special use cases.

## [./.github/workflows/rustGlobalBuild.yml](./.github/workflows/rustGlobalBuild.yml)

The full pipeline orchestrator. This is what downstream repositories call — everything else in this document is an implementation detail of it.

Inputs:

1. `binary_names`: Required. JSON array of binary names to build, e.g. `'["my-app"]'`. Add multiple entries for monorepos.
1. `aur_package_name`: Optional. AUR package name. Omit if the project is not on the AUR.
1. `dependent_repo_names`: Optional. Repositories to notify via issue on tagged releases, one `Owner/Repo` per line. JSON arrays are still accepted for compatibility.
1. `git_username` / `git_email`: Optional. AUR commit identity. Defaults to the DreamWeave maintainer values.
1. `publish_docs`: Optional, default `true`. Set `false` if the project uses its own static site generator for documentation.
1. `cargo_publish`: Optional, default `true`. Runs `cargo publish --dry-run` on every non-tag push, and `cargo publish` on tagged releases. Set `false` if the project does not publish to crates.io. Requires `CARGO_REGISTRY_TOKEN` secret.
1. `generate_changelog`: Optional, default `true`. Generates `CHANGELOG.md` from git history and uploads it to the release.
1. `generate_benchmarks`: Optional, default `false`. Runs `cargo bench`, generates `BENCHMARKS.md` from Criterion output when available, otherwise preserves the raw benchmark log, and uploads it to the release.

The pipeline runs these jobs:

- Quality gates (parallel, block release): `test` (full platform matrix), `fmt`, `clippy` (pedantic), `audit` (RustSec)
- Informational (parallel, does not block): `publish-dry-run`
- Release builds (after gates pass): `release` (macOS ARM + Intel, Windows), `release-linux` (AlmaLinux 8 container for glibc 2.28 compatibility)
- Doc/artifact generation (after gates pass, skipped on PRs except docs which only runs on main pushes): `docs` (GitHub Pages), `changelog`, `benchmarks`
- Post-release (after all builds): `publish` (crates.io, tag only), `aur-publish`, `call-discord-webhook`, `nag-dependents`

## [./.github/workflows/libGlobalBuild.yml](./.github/workflows/libGlobalBuild.yml)

The library equivalent of `rustGlobalBuild.yml`. Use this for crates that have no distributable binary — it runs all the same quality gates, publishing, docs, changelog, and benchmarks, but has no `corprus-crucible` release build jobs and no AUR publishing.

Inputs:

1. `crate_names`: Required. JSON array of crate names to publish, e.g. `'["my-lib"]'`. Used to locate each crate's `Cargo.toml` via `cargo metadata` (hyphens and underscores are treated as equivalent).
1. `dependent_repo_names`: Optional. Repositories to notify via issue on tagged releases, one `Owner/Repo` per line. JSON arrays are still accepted for compatibility.
1. `publish_docs`: Optional, default `true`. Set `false` if using a custom SSG.
1. `cargo_publish`: Optional, default `true`. Dry-run on non-tag pushes; real publish on tagged releases. Requires `CARGO_REGISTRY_TOKEN` secret.
1. `generate_changelog`: Optional, default `true`.
1. `generate_benchmarks`: Optional, default `false`.

## [./.github/actions/corprus-crucible/action.yml](./.github/actions/corprus-crucible/action.yml)

Composite action used internally by `rustGlobalBuild.yml`. Handles the release artifact pipeline for a single binary on a single platform. Called once per OS per binary name.

Inputs:

1. `binary_name`: Required. The executable name to build, without platform extension.
1. `include_files`: Optional. Comma-separated list of additional files to include in the release zip. Paths are relative to the build directory. Defaults to `Readme.md,LICENSE`.
1. `vt_api_key`: Required. VirusTotal API key.
1. `github_token`: Required. GitHub token for uploading release artifacts.
1. `release_name`: Required. Output from `createRelease` — either a tag name or `development`.

Build context detection: if `binary_name` matches a directory at the repo root, the action builds from that directory. Otherwise builds from `.`. This handles monorepos transparently.

On pull requests, signing and VirusTotal scanning are skipped; the binary is uploaded as a workflow artifact instead.

Corprus Crucible shell implementation details live under `scripts/corprus-crucible/`. The composite action owns GitHub Actions orchestration; the scripts own validation, build context detection, binary suffix detection, release binary staging, signing, archive creation, and VirusTotal link formatting.

## [./.github/workflows/createRelease.yml](./.github/workflows/createRelease.yml)

Reusable workflow called at the start of every pipeline. Deletes any existing release matching the current tag (or `development` on non-tag pushes), then creates a fresh one with auto-generated changelog and a workflow run ID marker.

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

1. `webhook_url`: Required. Use the org-level secret unless there is a specific reason not to.

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
