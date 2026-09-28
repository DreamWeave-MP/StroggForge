+++
title = "Contributing"
description = "Choose a useful first task and find its real repository."
weight = 50
sort_by = "weight"
template = "docs/section.html"
page_template = "docs/page.html"
+++

## Start here

1. Read the [ecosystem overview](@/ecosystem/overview.md). Pick a problem, not forty repositories.
2. Find the component in the [directory](@/ecosystem/projects/_index.md). Each page identifies
   the checkout/component path, maintainers, docs, difficulty and downstream relationships.
3. Check [good contribution targets](@/contributing/targets.md) and the repository's issues.
   Contribution areas are suggestions, not assignments or claims of open tickets.
4. For cross-project work, consult the [release board](@/releases/board.md).
5. Follow that repository's `AGENTS.md`, contribution instructions and quality gate.

## Choose a lane

- **New to DreamWeave:** improve a CLI recipe, reproduce an installation issue, or test the
  Mod Template's keyboard/mobile docs experience.
- **Rust asset tooling:** start with dream-path/openmw-config fixtures; then archive/VFS
  behavior. Use synthetic assets or redistributable examples.
- **OpenMW Lua:** St4sh and its Cod3x field manual are the content/documentation entry point.
- **Build engineering:** reproduce a failing consumer job at its pinned StroggForge ref.
  A compiler migration needs both configuration and artifact/runtime evidence.
- **Runtime internals:** L3i/dream-net need lifetime, codegen and wire-format expertise.

[Maintenance loop](@/contributing/maintenance.md) · [Engineering standards](@/contributing/standards.md)
