+++
title = "Documentation architecture"
description = "Where DreamWeave's docs come from and how they reach a browser."
weight = 20
+++

{{ diagram(name="documentation") }}

## Two Kinds of Docs

API docs and manuals are different animals, and pretending otherwise ends badly.

| | Input | Rendered by | Published by |
|---|---|---|---|
| API docs | Cargo source | rustdoc, in rustdoc's own UI | The public workflows' `docs` job on configured main pushes |
| Manuals and this site | Markdown plus generated pages | The DreamWeave docs shell on Zola | The project's own Pages workflow |

This site links to API docs; it does not try to push rustdoc output through Tera. A project
with its own Zola site should set `publish_docs: false` on its Rust workflow, or the two
deployments will take turns overwriting each other.

## The Mod Template

DreamWeave-Mod-Template supplies the docs shell: recursive sidebar, breadcrumbs, a local
table of contents, drawers for both on narrow screens, scoped styles and no framework.
StroggForge imports that shell (the receipt is in `site/UPSTREAM.md`) and puts its own skin,
search and copy buttons on top in separate files, so upstream improvements can still be
pulled in.

## St4sh and Cod3x

St4sh is the Lua(u)/OpenMW side of DreamWeave. It publishes mods, their Lua tooling and
their docs, and Cod3x, the OpenMW Lua field manual, lives inside it under
`content/cod3x/docs`. It uses the same docs shell. Its site workflow calls StroggForge's
`createRelease` helper and nothing else; no Rust job ever builds, signs or ships it.

## When Docs Break

- API docs missing: the consumer's `docs` job, its `cargo doc` command, Pages permissions.
- This site wrong: fix the TOML or `site/pages` and rebuild. Never the generated pages.
- Layout or navigation broken: the imported shell, `site/UPSTREAM.md`, then the skin.
