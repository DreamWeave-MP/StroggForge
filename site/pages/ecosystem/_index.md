+++
title = "Ecosystem"
description = "What DreamWeave contains and how the pieces connect."
weight = 10
sort_by = "weight"
template = "docs/section.html"
page_template = "docs/page.html"
+++

DreamWeave is three different kinds of work that happen to share an organization.

- **The Rust ecosystem:** OpenMW tooling, the foundation libraries under it, and the L3i
  runtime. StroggForge builds, signs and ships this side, and only this side.
- **Lua(u) / OpenMW:** mods, OpenMW Lua tooling and the Cod3x field manual, all living in
  St4sh. It is documented here because you need to know it exists. It does not run
  through the Rust supply line.
- **Web and documentation sites:** the Mod Template and dreamweave-web. St4sh and most of the
  Rust programs and libraries publish Mod Template sites: a page, its docs, and the protocol
  manifest a DreamWeave client or index reads, with every release and its digests.

Sharing an organization is not a dependency. The [overview](@/ecosystem/overview.md) lists
everything by domain, the [map](@/ecosystem/map.md) shows what actually depends on what,
and the [project directory](@/ecosystem/projects/_index.md) has a page per component.
