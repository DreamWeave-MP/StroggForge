+++
title = "War room architecture"
description = "Four small TOML files, a Rust generator, and Zola."
weight = 1
+++

Human intent lives in a few small TOML files. Facts about StroggForge come from
StroggForge. Rust checks the references and generates the pages. Zola renders them. Git
remembers everything else.

{{ schematic(data_path="data/schematics/war-room-build.json") }}

## Where Things Live

| Information | Home |
|---|---|
| Projects, domains, lifecycles, relationships, retired infrastructure, change log | `war-room/ecosystem.toml` |
| Releases and campaigns, their requirements and blockers | `war-room/plans.toml` |
| Target platforms and compiler policy | `war-room/toolchains.toml` |
| Workflow operating notes and supply-chain stages | `war-room/workflows.toml` |
| Workflow inputs, secrets, jobs, runners, conditions | the YAML itself, read at build time |
| Long-form writing | `site/pages/` |
| Presentation | `site/templates`, `site/sass`, `site/static` |

Anything about another repository, like which foundation a tool depends on or which
workflow a project calls, is a reviewed line in `ecosystem.toml`. StroggForge does not
clone other repositories to find out. It would be wrong half the time and slow the other
half.

`site/content` and `site/static/generated` are rebuilt from nothing on every run and
ignored by Git. Editing them is a fine way to lose an afternoon.

## What the Generator Refuses

Duplicate IDs. References to projects, plans, platforms, workflows, jobs or composite
steps that do not exist. A plan whose state disagrees with its requirements: blocked
without a blocker, or ready with work outstanding. Cycles in prerequisites or successions.
Unknown fields and enum values fail when the TOML parses.

That is the whole list, on purpose. Whether a project is really `active`, or whether a
retirement is worth recording, is a judgment call for review, not for a compiler.

## Mermaid

Rust writes `.mmd` sources from the same records the tables use, so the diagrams cannot
drift from the text. `npm run diagrams` renders them to SVG once, at build time. The
browser receives an image and the source next to it; no Mermaid runs in the visitor's
tab.

## Domains

Every project belongs to one domain: `rust`, `openmw-lua` or `web`. Maps group by domain
before anything else, because the most common misreading of DreamWeave is assuming every
repository runs through the Rust supply line. St4sh calling the `createRelease` helper is
CI usage, not membership.

## History

A superseded project keeps its record with a `superseded` lifecycle and a `successor`
relationship. Infrastructure with no project record, like a replaced builder image, gets
a `[[retired]]` entry saying what replaced it and why. Archaeology is curated. Record a
retirement when it explains the current architecture, not because something got old.

[Maintenance loops](@/contributing/maintenance.md)
