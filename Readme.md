# StroggForge

StroggForge is DreamWeave's shared build and release infrastructure for Rust applications
and Rust ecosystem components: reusable GitHub workflows, the Corprus Crucible composite
action, the `setup-llvm` action, and the shell helpers they run. It verifies, builds, signs,
packages, documents and publishes the Rust side of DreamWeave.

It also hosts the **DreamWeave War Room** at <https://dreamweave-mp.github.io/StroggForge/>:
what DreamWeave contains, what depends on what, what ships next, what is stuck, and which
platforms and toolchains are standard. It covers the Lua(u)/OpenMW and web sides of
DreamWeave too, without pretending they run through the Rust supply line.

## Using the workflows

- Binary repositories: copy [`.github/action_templates/rust_template.yaml`](.github/action_templates/rust_template.yaml) (calls `rustGlobalBuild.yml`).
- Library repositories: copy [`.github/action_templates/lib_template.yaml`](.github/action_templates/lib_template.yaml) (calls `libGlobalBuild.yml`).
- Scheduled quality checks: [`.github/action_templates/daily_quality_template.yaml`](.github/action_templates/daily_quality_template.yaml).

The war room's generated contract pages list every input, secret, job and condition.
[StroggForge Actions](./StroggForge%20Actions.md) is the long-form reference.

## Working on the war room

| Where | What |
|---|---|
| `war-room/ecosystem.toml` | Projects, domains, lifecycles, relationships, retired infrastructure, change log |
| `war-room/plans.toml` | Releases and campaigns, with requirements and blockers |
| `war-room/toolchains.toml` | Target platforms and compiler policy |
| `war-room/workflows.toml` | Workflow operating notes and supply-chain stages |
| `tools/war-room/` | The Rust generator: validates references, writes the site's pages |
| `site/` | Zola site on the DreamWeave Mod Template docs shell |

```sh
npm ci
cargo run -p stroggforge-war-room -- build
npm run diagrams
zola --root site serve
```

The maintenance loops (adding a project, planning a release, changing a workflow) are in
[`site/pages/contributing/maintenance.md`](site/pages/contributing/maintenance.md), and
where everything lives is in
[`site/pages/engineering/architecture.md`](site/pages/engineering/architecture.md).

## Other notebooks

- [Release Planning](./Release%20Planning.md): the manual release checklist (partly historical; see the site's notebook index).
- [AUR team](./AUR-TEAM.md): AUR package maintainers.
- [DreamWeave Applications](./DreamWeave%20Applications.md) and `DreamWeave Map.drawio`/`.png`: historical, superseded by the generated project directory and maps.
