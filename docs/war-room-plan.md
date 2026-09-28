# War room implementation plan

## Inspection findings (2026-09-28)

- StroggForge currently uses the GitHub Pages Hacker/Jekyll configuration and root Markdown pages. There is no Rust application workspace yet.
- The current local Mod Template (`d86d2cb50fe6539132a9f6e1d37d13b9699e8a17`) has the reusable six-file `templates/docs/` shell, scoped `sass/docs.sass`, and `static/docs/docs.js`. St4sh (`86cbc26146de7daf30f5fbf57d791d847dd8ac9b`) uses that shell for Cod3x and mod API docs. Import that foundation with provenance; keep StroggForge styling separate.
- Local Cargo sources correct the older map: CHIMERA is a manager workspace inside dreamweave-web; the dream-binder checkout now builds `l3i`; Cod3x is content inside St4sh. Neither a missing public repository nor an old description establishes retirement.
- Public organization inventory was checked with `gh repo list`; local manifests/workflows take precedence. Existing untracked ecosystem research and concurrent release/toolchain edits belong to the user.
- LLVM migration is **in progress**, per maintainer. LLVM 22.1.8 is a measured/reference configuration, not evidence that every shipping artifact uses that exact version. Current workflow configuration and release validation must be separate.

## Vertical implementation

1. Add a small `tools/war-room` Rust workspace member. Parse strict typed TOML inventory and planning files; capture reviewable source snapshots from explicit local clones with Cargo metadata, Git provenance, and workflow text. Normal site generation uses only committed files.
2. Derive package edges and StroggForge callers from snapshots; derive workflow contracts, job dependencies, runners and containers directly from this checkout. Validate IDs, references, lifecycle/migration edges, release prerequisites and contradictory ready/blocked states.
3. Generate a disposable `site/content` and `site/static/generated` tree from hand-authored `site/pages` plus the canonical model. One command produces the entire site, without mutating canonical configuration. Generated output is ignored; CI builds twice and compares bytes.
4. Import the Mod Template docs foundation with a small host shell and scoped engineering skin. Render a useful homepage first, then directory, release/campaign board, workflow contracts, pipeline graphs, platform policy, history, contributor paths and incident procedures.
5. Render Mermaid into static SVG during the documentation build, retaining downloadable Mermaid source and textual tables. The site remains useful without JavaScript or a network request. JavaScript only enhances search/copy/navigation.
6. Add PR validation and gated Pages deployment: Rust fmt/tests/Clippy, generation determinism, Mermaid parsing/rendering, Zola build and local links/anchors. Document pinned tool installation and the exact maintenance loop.
7. Exercise the twelve acceptance questions against the built site. Preserve old engineering notes as explicitly historical sources and remove the replaced Jekyll configuration.

## Source boundaries

Inventory and lifecycle/contribution policy: `war-room/ecosystem.toml`. External observed package/workflow facts: `war-room/sources/*.json`, refreshed explicitly from local clones. Intent and blockers: `war-room/plans.toml`. Reviewed platform expectations: `war-room/toolchains.toml`. Current workflow API: `.github/workflows` and composite action metadata. Explanations: `site/pages`. Rendering: Zola/Tera. Generated Markdown, JSON and Mermaid: Rust. Git remains the audit log; there is no live-health claim or runtime API.

## Completion pass (2026-09-28)

- Rendered the reviewed `[[updates]]` digest (commit-linked, newest first, optional campaign link) on the homepage and a change log page; removed the homepage's hard-coded campaign and migration prose.
- Supply-chain stages became data in `.github/war-room-workflows.toml`; every `<workflow>/<job>` or `<action>/<step>` reference resolves against the YAML. The stage graph, table and homepage line are generated from it. Stages are ordered by earliest start, not strict sequence.
- Added a required project `domain` (rust, openmw-lua, web). St4sh and Cod3x are the Lua(u)/OpenMW side and not Rust components; maps group by domain, and callers are grouped by the workflow they call so a `createRelease`-only static site is not presented as a Rust consumer.
- Added `[[retired]]` records (repository, organization, build environment, site) with typed successors, one page each carrying a "no longer maintained or supported" notice. The organization census (`capture-organization`) forces a retired record for every archived repository; maintainer-declared dead repositories are recorded with that statement as evidence.
- Added an input/secret cross-reference, the daily quality template, platform assertions for the Windows explicit-target and Apple linker-plugin-lto differences, and moved policy assertions into `check`.
- Added validation tests that mutate the committed model and require specific rejections.
- Removed the Jekyll `_config.yml`; the root notebooks stay, annotated as current or historical.
