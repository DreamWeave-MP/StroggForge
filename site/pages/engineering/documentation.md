+++
title = "Documentation architecture"
description = "Source-driven docs, a shared DreamWeave shell, and static publication."
weight = 20
+++

{{ diagram(name="documentation") }}

## Two documentation paths

| Path | Input | Presentation | Publication |
|---|---|---|---|
| Consumer API docs | Cargo workspace / rustdoc | rustdoc's own UI | Public workflows' `docs` job on configured main pushes |
| Project manuals and this war room | Markdown, reviewed metadata, generated reference | Mod Template docs shell / Zola | Project-specific Pages workflow |

The war room does not pretend rustdoc output automatically passes through Tera. It links
to API docs while its own Rust generator supplies Markdown/data for Zola. Projects with
their own SSG should set `publish_docs: false` on the Rust workflow to avoid competing
Pages deployments.

## Where Mod Template fits

DreamWeave-Mod-Template supplies the small reusable docs architecture: recursive sidebar,
breadcrumbs, page-local TOC, scoped styles, copy controls and progressive enhancement.
StroggForge vendors that foundation with an import receipt and a separate tooling skin.
The normal project/storefront shell is not required to host this docs-only war room.

## Where St4sh and Cod3x fit

St4sh is the **Lua(u)/OpenMW side** of DreamWeave, not part of the Rust ecosystem. It
publishes OpenMW mods, their Lua tooling and docs; Cod3x is its engineering field manual
under `content/cod3x/docs`. That is the onboarding route for Lua/mod engineering. St4sh
uses the Mod Template docs shell, and its site workflow calls only StroggForge's
`createRelease` helper to refresh the release its static site uploads to. No Rust
manufacturing job builds, signs or ships anything for it.

## Documentation failure routing

- Broken API docs: inspect the consumer `docs` job, Cargo doc command and Pages permissions.
- Broken war-room content: validate source metadata and regenerate before editing templates.
- Broken navigation/layout: inspect the imported docs shell, `site/UPSTREAM.md` and the scoped skin.
- Competing deployments: disable one publisher; do not let rustdoc and Zola race for the same Pages site.
