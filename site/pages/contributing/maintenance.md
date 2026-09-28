+++
title = "Maintain the war room"
description = "The short loops for projects, plans, workflows and the site."
weight = 1
+++

## Running It Locally

You need Rust, Zola 0.22.1, Node 22.13 or newer (Mermaid rendering happens at build time)
and Python 3 (the link check). From the repository root:

```sh
npm ci
cargo run -p stroggforge-war-room -- build
npm run diagrams
zola --root site serve
```

Edit `war-room/*.toml`, `site/pages`, templates or styles. `site/content` and
`site/static/generated` are regenerated every build; changes there vanish.

## Adding a Project

1. Add a `[[projects]]` record to `war-room/ecosystem.toml`: kind, domain, lifecycle,
   repository, packages, distribution, the StroggForge workflows its repository calls,
   approach and contribution areas.
2. Add `[[relationships]]` for the real connections: Rust dependencies, documentation
   inputs, containment, succession. Arrows point consumer → dependency.
3. Build and look at the project page and the map. If the map looks wrong, the data is
   wrong.

## Retiring Something

Set a project's lifecycle to `superseded` or `archived`, and add a `successor`
relationship if something replaced it. Infrastructure with no project record, such as a
builder image, gets a `[[retired]]` entry saying what replaced it and why.

Only record retirements that explain how things work now. A dead repository nobody will
ever ask about does not need a headstone.

## Recording an Infrastructure Change

Add an `[[updates]]` entry to `war-room/ecosystem.toml` with the date, a title, what
changes for consumers, the commits, and optionally the `plan` it advances. The homepage
shows the newest four; the [change log](@/releases/changes.md) shows all of them.

Write it for the person whose release just broke. "Windows output moved under
`target/<triple>/release`" is useful. "Improved Windows support" is not.

## Planning a Release

Add a record to `war-room/plans.toml`:

```toml
[[plans]]
id = "vfstool-next"
kind = "release"
name = "VFSTool release preparation"
projects = ["vfstool"]
version = "0.11.0"
state = "preparing"
reviewed = "2026-09-28"
depends_on = []
platforms = ["linux-x64", "windows-x64"]
distribution = ["github-releases", "crates-io"]
notes = "What is in this release and why now."
links = ["https://github.com/DreamWeave-MP/vfstool"]

[[plans.requirements]]
id = "quality"
kind = "ci"
label = "Quality jobs pass at the release commit"
state = "todo"
detail = "Link the run once it is green."
evidence = []
```

That version is an example. Do not commit a plan for a release nobody decided on.

A `blocked` requirement makes the plan `blocked`, and the generator refuses the file if
you forget. `ready` means every requirement is done. Add `target_date` only when somebody
has actually committed to one. Releases end as `released`, campaigns as `done`.

## Changing a Workflow

Change the YAML and scripts. Inputs, secrets, jobs and conditions show up on the site
without further effort. Then:

- Update `war-room/workflows.toml` if the workflow's purpose, assumptions or failure modes
  changed, or if you renamed a job or composite step that a `[[stage]]` names. The build
  tells you about the second case.
- Update `war-room/toolchains.toml` if a platform baseline or compiler policy moved.

## Checks

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -W clippy::pedantic -D warnings
actionlint
cargo run -p stroggforge-war-room -- build
npm run diagrams
zola --root site build
python3 scripts/war-room/check-site.py
```

`check-site.py` walks the built site and fails on any local link, asset or anchor that
points nowhere. CI in `.github/workflows/war-room.yml` runs the same list and deploys
`main` to GitHub Pages. The repository's Pages source has to be set to GitHub Actions,
once, by hand.

## Refreshing the Docs Shell

The shell comes from DreamWeave-Mod-Template. `site/UPSTREAM.md` records which revision
and what we changed. Diff against a newer revision, apply what matters, keep the
StroggForge skin in its own files, update the receipt.
