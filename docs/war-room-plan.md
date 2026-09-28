# War room plan

StroggForge's site used to be a Jekyll render of a few root notebooks. It is now the place
to look up what DreamWeave is doing: what exists, what depends on what, what ships next,
what is stuck, and how the supply line works.

## Shape

Human intent lives in four small TOML files under `war-room/`: the ecosystem inventory,
release and campaign plans, platform policy, and workflow notes. Facts about StroggForge
come from StroggForge's own workflow YAML. A small Rust generator (`tools/war-room`)
checks references and writes Markdown and Mermaid. Mermaid is rendered to static SVG at
build time. Zola renders the rest on the DreamWeave Mod Template docs shell. Git holds
history. There is no server, database or client-side framework.

## Decisions worth remembering

- **No cross-repository capture.** An earlier version cloned consumer repositories and
  froze their Cargo metadata, workflows and selected files into StroggForge, then
  published the lot. It was slow, went stale, recorded dirty working trees as facts, and
  was one careless evidence file away from leaking something. Cross-project facts are now
  reviewed lines in `ecosystem.toml`.
- **Validation is structural.** IDs, references, plan states and cycles. An earlier version
  also enforced an organization census, forbidden words and literal strings in shell
  scripts. Those were editorial policy wearing a compiler's clothes and are gone.
- **Archaeology is curated.** A retirement is recorded when it explains the current
  architecture, not because GitHub reports something archived.
- **Domains.** Projects belong to the Rust ecosystem, the Lua(u)/OpenMW side or the web
  sites. Only the first runs through StroggForge.
- **Structure is rendered, not drawn in glyphs.** Flows use the schematic shortcode
  imported from St4sh; graphs use Mermaid rendered to SVG.
- **Sass, indented syntax.** Not SCSS.

## Open items

Tracked in `war-room/plans.toml` under the `war-room` campaign: review lifecycles, switch
the repository's Pages source to GitHub Actions, and check the deployed site.
