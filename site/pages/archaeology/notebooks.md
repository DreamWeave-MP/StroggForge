+++
title = "Maintainer notebooks"
description = "The hand-written root documents: which are current, which are history."
weight = 20
+++

StroggForge started as a Jekyll site rendering a few root Markdown notebooks. The war room
replaced that site, not the notebooks: they remain in the repository root, and some are
still maintained alongside the workflows.

| Notebook | Status | Use it for |
|---|---|---|
| [StroggForge Actions](https://github.com/DreamWeave-MP/StroggForge/blob/main/StroggForge%20Actions.md) | **Current**; updated with workflow changes | Long-form workflow and helper-script reference. The [generated contracts](@/stroggforge/workflows/_index.md) read the same YAML. |
| [Release Planning](https://github.com/DreamWeave-MP/StroggForge/blob/main/Release%20Planning.md) | **Mixed** | Its Rust application steps match the current workflows. Its modlist sections describe work outside StroggForge. |
| [AUR team](https://github.com/DreamWeave-MP/StroggForge/blob/main/AUR-TEAM.md) | **Current** | Who maintains DreamWeave's AUR packages. |
| [DreamWeave Applications](https://github.com/DreamWeave-MP/StroggForge/blob/main/DreamWeave%20Applications.md) | **Historical** | An early application list, replaced by the [project directory](@/ecosystem/projects/_index.md). |
| [Draw.io map](https://github.com/DreamWeave-MP/StroggForge/blob/main/DreamWeave%20Map.drawio) and [PNG](https://github.com/DreamWeave-MP/StroggForge/blob/main/DreamWeave%20Map.png) | **Historical** | The hand-drawn organization map, replaced by the [generated map](@/ecosystem/map.md). |

## Corrections the old map needed

- dreamweave-web contains the current CHIMERA Rust manager workspace; it is not stale.
- Morrobroom's current manifest no longer matches the old map's geometry and config edges.
