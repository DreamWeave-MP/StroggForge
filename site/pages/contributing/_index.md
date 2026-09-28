+++
title = "Contributing"
description = "Pick a problem, find its repository, and start."
weight = 50
sort_by = "weight"
template = "docs/section.html"
page_template = "docs/page.html"
+++

## Start here

DreamWeave has a lot of repositories. You do not need to understand all of them, and
nobody expects you to.

1. Skim the [ecosystem overview](@/ecosystem/overview.md) and pick a domain.
2. Open the component's page in the [directory](@/ecosystem/projects/_index.md). It tells
   you the repository, who looks after it, how hard it is to get into, and what it
   depends on.
3. Check [good targets](@/contributing/targets.md), then the repository's own issues.
   Those contribution areas are suggestions, not claimed tickets.
4. For anything that crosses projects, look at the [release board](@/releases/board.md)
   first. Somebody may already be halfway through it.
5. Follow the repository's own `AGENTS.md`, contribution notes and quality gate.

## Pick a lane

- **New here:** improve a CLI recipe, reproduce an installation bug, or break the docs
  shell's keyboard and mobile navigation and tell us how.
- **Rust asset tooling:** dream-path and openmw-config fixtures are the gentlest entry;
  archive and VFS behavior come after. Use synthetic assets or ones you can redistribute.
- **OpenMW Lua:** St4sh and its Cod3x field manual.
- **Build engineering:** reproduce a failing consumer job at its pinned StroggForge ref.
  A compiler migration is not done until an artifact actually runs where it should.
- **Runtime internals:** L3i and dream-net. Bring an appetite for lifetimes, codegen and
  wire formats.

[Maintaining the war room](@/contributing/maintenance.md) · [Engineering standards](@/contributing/standards.md)
