+++
title = "Evidence and inventory scope"
description = "What an observation proves, what it does not, and how to refresh it."
weight = 15
+++

## Evidence hierarchy

1. The checked-out source wins over historical diagrams and descriptions.
2. A Cargo declaration establishes a direct package requirement, including its kind,
   version, target condition and optionality. It does not prove a release is published.
3. A reusable workflow caller establishes repository-level integration at its recorded
   ref. It does not prove a successful run or that every package in the repo is shipped.
4. Release plans express reviewed intent and completion evidence.
5. Lifecycle/contribution classifications are reviewed editorial policy, not inferred
   from the latest commit timestamp.

## Initial research boundary

The September 28, 2026 pass inspected the local clones under `$HOME/GitHub`, the current
Mod Template and St4sh/Cod3x docs shell, and the public DreamWeave-MP inventory using
`gh repo list`. The directory focuses on the source-backed Rust/tooling/docs core.
It does not silently classify unlisted organization repositories as archived. The
[organization census](@/ecosystem/census.md) lists every repository as documented, retired
or not yet reviewed; validation fails if GitHub reports a repository archived and no
retired record documents it. Dead repositories that are not archived are recorded as
retired on a maintainer's statement, with that statement as the evidence.

Engine forks, Starwind/TES3MP content, supporting vendor forks and distribution-only
repositories remain relevant context. Expand them into canonical nodes when their
current operational role and source receipts have been reviewed. Useful entry points:
[DreamWeave engine](https://github.com/DreamWeave-MP/DreamWeave),
[dream-deps-build](https://github.com/DreamWeave-MP/dream-deps-build),
[rubic0n](https://github.com/DreamWeave-MP/rubic0n),
[Starwind-R3mastered](https://github.com/DreamWeave-MP/Starwind-R3mastered),
[Makron](https://github.com/DreamWeave-MP/makron), and
[shader compilation](https://github.com/DreamWeave-MP/dreamweave-shader-compilation).
These links imply no new dependency edge.

## Important corrections to older maps

- dreamweave-web contains a current CHIMERA Rust manager workspace. Calling the entire
  repository retired from an old description would be wrong.
- dream-binder is the local directory/former package name; its current package is l3i.
  It has no configured Git remote in the inspected clone. No public URL is invented.
- Morrobroom's current manifest no longer matches the older geometry/openmw-cfg edges.
- Optional Lua features in current foundations still show their declared bindings;
  the LLVM/L3i campaign does not magically create l3i dependencies everywhere.
- Cod3x is published inside St4sh, not a separate runtime or hosted API.
- LLVM migration is not complete. EL9 configuration is distinct from verified artifacts.

## Snapshot freshness

Every project page records observation date, Git revision and dirty-tree status.
The normalized [generated model](../../generated/model.json) carries the same receipts.
A dirty snapshot is intentionally identified as a working-tree observation rather than
falsely attributed wholly to a commit. Before a release decision, refresh affected source
snapshots, inspect the diff and attach the actual Actions/artifact evidence to the plan.

Network access is needed to install build tools/dependencies initially. It is not needed
to retrieve ecosystem facts or render the site once those tools are available locally.
