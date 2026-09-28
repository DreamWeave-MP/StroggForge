+++
title = "Maintainer notebooks"
description = "The hand-written root documents: which parts are current, which are archaeology."
weight = 20
+++

StroggForge started as a Jekyll site that rendered a handful of root Markdown notebooks.
The war room replaced that site, not the notebooks: they remain in the repository root
and some are still maintained alongside the workflows. Read each one with its status in mind.

| Notebook | Status | Use it for |
|---|---|---|
| [StroggForge Actions](https://github.com/DreamWeave-MP/StroggForge/blob/main/StroggForge%20Actions.md) | **Current**; updated with workflow changes | Long-form workflow and helper-script reference. The [generated contracts](@/stroggforge/workflows/_index.md) are derived from the same YAML. |
| [Release Planning](https://github.com/DreamWeave-MP/StroggForge/blob/main/Release%20Planning.md) | **Mixed** | Its Rust application steps match the current workflows. Its AUR list still names delta-plugin-git, which is [retired](@/archaeology/retired/delta-plugin-git.md). Its DW-Tools, high-end resources and Sixth-House-Mod-Cache sections describe a modlist lane outside StroggForge whose current procedure has not been reviewed here. |
| [AUR team](https://github.com/DreamWeave-MP/StroggForge/blob/main/AUR-TEAM.md) | **Current** | Who maintains DreamWeave's AUR packages. |
| [DreamWeave Applications](https://github.com/DreamWeave-MP/StroggForge/blob/main/DreamWeave%20Applications.md) | **Historical** | An early application list; its TES3Merge entry is [retired](@/archaeology/retired/tes3merge.md). The [project directory](@/ecosystem/projects/_index.md) supersedes it. |
| [Draw.io map](https://github.com/DreamWeave-MP/StroggForge/blob/main/DreamWeave%20Map.drawio) and [PNG](https://github.com/DreamWeave-MP/StroggForge/blob/main/DreamWeave%20Map.png) | **Historical** | The hand-drawn organization map. The [generated ecosystem map](@/ecosystem/map.md) replaces it; the drawing is kept to show what the organization used to look like. |

## Earlier DreamWeave infrastructure

The older notebooks and map describe DW-tools, Sixth-House-Mod-Cache, Starwind/TES3MP
server tooling and a high-end resources repository. DW-tools and Sixth-House-Mod-Cache are
not retired; they sit outside the reviewed Rust/tooling core and appear in the
[census](@/ecosystem/census.md) as not yet reviewed. The Starwind/TES3MP server tooling
(DreamScripts, Starwind-Builder, dreamMounts, CoreScripts, dream-deploy) is retired. The
high-end resources repository does not appear in the public organization listing; its
fate is unknown rather than assumed.

The repositories GitHub does report archived are recorded, with evidence, in the
[retired infrastructure ledger](@/archaeology/history.md#retired-infrastructure).
Archive status alone never produces a successor edge.

## Corrections the old map needed

- dreamweave-web contains a current CHIMERA Rust manager workspace; the older claim that
  the whole repository was stale is not adopted.
- Morrobroom's current manifest no longer matches the older geometry/openmw-cfg edges.
- Makron now appears in the organization inventory; the older unresolved-location note
  should not be read as evidence of its absence.

[Evidence rules and inventory scope](@/engineering/evidence.md) explain how to expand the
curated inventory without treating every surviving repository as active architecture.
