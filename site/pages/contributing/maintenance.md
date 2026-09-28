+++
title = "Maintain the war room"
description = "Short, explicit local loops for projects, plans, workflows and the site."
weight = 1
+++

## Local development

Prerequisites: Rust (generator MSRV 1.88), **Zola 0.22.1**, **Node 22.13+** and Python 3.
`package-lock.json` pins the build-only Mermaid renderer and browser-test dependencies.
Install Zola from its official release or your package manager; verify `zola --version`.

From the repository root:

```sh
npm ci
cargo run --locked -p stroggforge-war-room -- build
npm run diagrams
zola --root site build
python3 scripts/war-room/check-site.py
zola --root site serve
```

The first dependency install downloads packages and Chromium for the diagram renderer.
After installation, generation/rendering uses local inputs. `cargo run --offline --locked`
works with a populated Cargo cache. No ecosystem API calls happen during a normal build.

`site/content` and `site/static/generated` are entirely generated; `site/public` is Zola's
output. They are ignored and replaced on each build. Edit `site/pages`, templates, styles
or canonical metadata instead. Running `zola serve` after generation previews the exact
same architecture as CI.

## Adding a project

1. Add a `[[sources]]` entry in `war-room/ecosystem.toml` for an actual local clone:
   source ID, path relative to your clones root, repository URL (or `local-only`), Cargo
   workspace manifest roots, and selected text evidence paths.
2. Add a `[[projects]]` record with kind, domain, lifecycle, maturity, owners, language, component
   path, real package names, distribution channels, approach and contribution areas.
3. Capture that source explicitly:

```sh
cargo run --locked -p stroggforge-war-room -- capture "$HOME/GitHub" dream-path
```

4. Declare only evidence-backed non-Cargo relationships. Rust edges and StroggForge
   caller edges are generated. Components inside one repository can be separate nodes.
5. Run `build`, render diagrams, inspect the directory/map, and review snapshot diffs.
6. Commit the canonical records, source snapshot and any explanation changes.

Capture runs `cargo metadata --no-deps --offline`; if that fails, fix the local manifest
or dependency metadata availability. Do not guess package fields. It records the current
working tree, including dirty status; review selected evidence for sensitive information.
Update the inventory's reviewed date when deliberately refreshing observations.

## Retiring something

1. Add a `[[retired]]` record to `war-room/ecosystem.toml`: kind (`repository`,
   `organization`, `build-environment` or `site`), what it was, an `https://` location, and
   evidence. Evidence is the archive flag, the deleting commit, or a maintainer's statement.
2. Add `successor = { project = "..." }` or `{ platform = "..." }` only when the migration is
   real. Otherwise leave it out; the page says no successor has been reviewed.
3. If the thing still has a source checkout and a project record, set its lifecycle to
   `superseded` or `archived` instead and add a `successor` relationship.
4. Generate. The record gets its own page with the unsupported notice.

## Refreshing the organization census

```sh
cargo run --locked -p stroggforge-war-room -- capture-organization
```

This reads the organization through `gh` (authenticated GitHub CLI) and rewrites
`war-room/sources/organization.json`, skipping the names in `war-room/census-exclusions.toml`.
Review the diff: a newly archived repository fails validation until it has a retired record.
Repositories without an inventory decision appear as **not yet reviewed**.

## Recording an infrastructure change

Add an `[[updates]]` entry at the **top** of `war-room/ecosystem.toml` (newest first is
enforced): date, title, what changes for consumers, the commits carrying it, and an optional
`plan` ID for the campaign it advances. The homepage shows the newest four; the change log
shows all of them.

## Planning a release

Add a record to `war-room/plans.toml`. This illustrative record is not an actual release commitment:

```toml
[[plans]]
id = "vfstool-next"
kind = "release"
name = "VFSTool release preparation"
projects = ["vfstool"]
version = "REPLACE-WITH-REAL-VERSION"
state = "preparing"
reviewed = "2026-09-28"
depends_on = []
platforms = ["linux-x64", "windows-x64"]
distribution = ["github-releases", "crates-io"]
notes = "Record the actual release scope and decision here."
links = ["https://github.com/DreamWeave-MP/vfstool"]

[[plans.requirements]]
id = "quality"
kind = "ci"
label = "Required quality jobs pass at the release commit"
state = "todo"
detail = "Attach the exact run URL and commit."
evidence = []

[[plans.requirements]]
id = "docs"
kind = "documentation"
label = "Review documentation and migration notes"
state = "todo"
detail = "Record the reviewed docs revision."
evidence = []

[[plans.requirements]]
id = "artifacts"
kind = "artifact"
label = "Verify configured artifacts and runtime behavior"
state = "todo"
detail = "Record signing and platform smoke-test evidence."
evidence = []
```

Use `blocked` on the requirement and plan when a blocker exists. Explain the blocker in
`detail`. Use `depends_on` for release/campaign prerequisites; cycles and unresolved IDs
fail. `ready` requires all requirements done and prerequisites completed. A real date
can be added as `target_date`; absent means no commitment. Completed releases use
`released`; completed campaigns use `done`. A campaign spans projects and uses a name,
not an invented version. Generate and inspect the board before committing.

## Changing a workflow

1. Update the executable YAML/scripts.
2. Update `.github/war-room-workflows.toml` if purpose, operational assumptions or failure
   modes change. Every reusable workflow/composite action must have notes. If a job or
   composite step is renamed, added or removed, update the `[[stage]]` references; a stale
   reference fails generation.
3. Update `war-room/toolchains.toml` when a reviewed baseline changes; assertions detect drift.
4. Generate. Input/secret/output/job tables come directly from YAML.
5. Run Actionlint and the site gate; inspect conditions and release boundary ordering.

## Refreshing the template

Follow `site/UPSTREAM.md`: compare the imported paths against a reviewed local upstream
revision, preserve the scoped StroggForge skin, and record the revision and local delta.
There is no initializer or configuration-rewriting bootstrap step.

## Validation commands

```sh
cargo fmt --all --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -W clippy::pedantic -D warnings
python3 scripts/war-room/check-generation.py
npm run diagrams
zola --root site build
python3 scripts/war-room/check-site.py
npm run test:site
```

`cargo test` loads the committed metadata and asserts that each class of broken record
(duplicate IDs, unresolved references, contradictory plan states, stale stage references,
undocumented archived repositories and more) is rejected with an actionable message.
`check-generation.py` invokes Rust twice and compares the entire generated file set.
`check-site.py` checks local href/src targets and fragment IDs under the configured base path.
The browser smoke test serves built files locally and checks desktop/mobile navigation,
search, copy controls, overflow and no-JavaScript reading. It is a development/CI tool,
not a server required by the published site.

## Publishing

Set GitHub Pages' source to **GitHub Actions**. The war-room workflow validates pull
requests and deploys successful default-branch builds through the `github-pages`
environment. A manual workflow dispatch is also available. No deployment success is
claimed by a local build; inspect the actual Pages job after merge.
