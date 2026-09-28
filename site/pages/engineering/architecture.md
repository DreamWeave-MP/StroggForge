+++
title = "War room architecture"
description = "One canonical model, multiple generated views, and no runtime service."
weight = 1
+++

## What is canonical

| Information | Source of truth |
|---|---|
| Project identity, domain, lifecycle, contribution policy, explicit non-Cargo relationships | `war-room/ecosystem.toml` `[[projects]]`, `[[relationships]]` |
| Retired repositories, organizations, build environments and sites | `war-room/ecosystem.toml` `[[retired]]` |
| Reviewed infrastructure change digest | `war-room/ecosystem.toml` `[[updates]]`, each naming its commits |
| Observed external package/workflow facts | `war-room/sources/*.json`, explicitly captured from local clones |
| Organization repository list and archive state | `war-room/sources/organization.json`, explicitly captured with `gh` |
| Current StroggForge API and job topology | `.github/workflows/*.yml`, `.github/actions/*/action.yml` |
| Workflow operational explanation and supply-chain stages | `.github/war-room-workflows.toml` |
| Release/campaign intent and blockers | `war-room/plans.toml` |
| Reviewed platform/compiler policy | `war-room/toolchains.toml`, with assertions against executable source |
| Long-form explanation | `site/pages/` |
| Presentation and shared docs foundation | `site/templates`, `site/sass`, `site/static`; import receipt in `site/UPSTREAM.md` |

## What is derived

`tools/war-room` produces `site/content/` and `site/static/generated/`:
normalized JSON, project pages, Cargo relationship graphs, workflow caller ledgers,
release rollups, input/secret/output tables, exact job dependency maps and platform tables.
Those directories are disposable and ignored by Git. They contain **no hand edits**.

Source capture is an explicit offline maintenance operation. It runs
`cargo metadata --no-deps --offline`, records Git HEAD and dirty-tree state, and saves
workflow text plus selected evidence. Ordinary builds never visit sibling clones, query
GitHub, resolve consumer dependencies, or fetch repository facts.

## Domains

Every project declares one domain: **rust** (the Rust ecosystem StroggForge serves),
**openmw-lua** (the Lua(u)/OpenMW content side, St4sh and Cod3x) or **web** (the Mod Template
and dreamweave-web). Maps group by domain before kind. A component with Cargo packages must
be in the rust domain, and a rust-domain component must list Rust among its languages. A CI
call from another domain to a StroggForge helper stays a labelled build edge; it never moves
the caller into the Rust ecosystem.

## What Rust owns

Rust owns strict parsing, enums, validation, source normalization, dependency/caller
extraction and deterministic generated text. It rejects duplicate IDs, missing package
assignments, broken references, unsupported relationships, cyclic release prerequisites,
contradictory ready/blocked states, stale platform assertions, supply-chain stages that
name a nonexistent job or step, archived repositories without a retired record, and
current projects living in archived repositories. It does not declare
an artifact healthy just because a workflow exists.

Snapshots are dated observations, not vendored dependencies. Cargo requirements retain
their requested versions, dependency kinds, optional flag and target restriction. They are
not a full resolved lockfile graph. A package-to-project assignment is explicit so that
workspace components do not become anonymous boxes or duplicate identities.

## What Zola owns

Zola/Tera owns presentation, durable routes, Markdown rendering, the search index,
breadcrumbs and local TOCs. The six-template DreamWeave docs shell supplies recursive
navigation and active branch expansion. A separate StroggForge skin supplies dense,
purple infrastructure styling. There is no SPA, database, API server or client state store.

## How Mermaid is produced

Rust generates `.mmd` source from the same normalized relationships and actual workflow
`needs` records used in tables. The supply-chain overview checks its job names against
the current workflow; long-form documentation may also supply a reviewed `.mmd` map.
The pinned Mermaid build-time renderer produces static SVG. Both source and SVG ship
with the site; ledgers and workflow tables provide a text equivalent. No Mermaid library
or CDN request runs in the visitor's browser.

## How a project gets added

Add an explicit source and project record, capture the local source, assign package
names, and add only evidence-backed non-Cargo relationships. Regenerate and inspect
the project page and graph. Rust/Cargo and StroggForge caller edges are derived, not
hand-entered. [Exact commands](@/contributing/maintenance.md#adding-a-project).

## How a release gets planned

Add a release record with one project, a version, real requirements and any prerequisites.
CI, documentation and artifact requirements are mandatory. A blocker must include a
useful detail; a completed requirement needs evidence. A ready release cannot have
unfinished requirements or unfinished prerequisites. Leave target dates absent until
someone actually commits to them. [Example](@/contributing/maintenance.md#planning-a-release).

## How history is represented

Two record types, both under Archaeology and both excluded from current maps:

- A **superseded project** still has a source checkout (dream-binder, now l3i). It keeps its
  project record with a `superseded` lifecycle and needs a typed successor edge with evidence.
- A **retired record** has no current checkout: an archived or abandoned repository, an
  abandoned organization, a replaced build environment or site. Its successor, when one is
  reviewed, is a current project or a platform; otherwise it says so explicitly.

Every historical page opens with a **no longer maintained or supported** notice. The
organization census proves coverage: GitHub's archive flag forces a retired record, and a
maintainer's statement retires a repository GitHub has not archived. An old repository name
alone is not proof of retirement.

## Automation boundary

CI validates Rust, generates twice and compares bytes, parses/renders Mermaid, builds
Zola, and checks local links and anchors. Only a successful default-branch run deploys
the artifact to GitHub Pages. External facts are refreshed by reviewed snapshot commits,
not an unreviewed scheduled crawler.
