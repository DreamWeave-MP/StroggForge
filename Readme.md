# StroggForge

StroggForge is DreamWeave's shared build and release infrastructure for Rust applications
and Rust ecosystem components: reusable GitHub workflows, the Corprus Crucible composite
action, the `setup-llvm` action, and the shell helpers they run. It verifies, builds, signs,
packages, documents and publishes the Rust side of DreamWeave.

It also hosts the **DreamWeave War Room** — <https://dreamweave-mp.github.io/StroggForge/> —
the engineering console for the whole ecosystem: what exists, how it relates, what is being
released, what is blocked, which toolchains and platforms are standard, and what has been
retired. The site documents the Lua(u)/OpenMW and web sides of DreamWeave too, without
making them depend on the Rust supply line.

## Using the workflows

- Binary repositories: copy [`.github/action_templates/rust_template.yaml`](.github/action_templates/rust_template.yaml) (calls `rustGlobalBuild.yml`).
- Library repositories: copy [`.github/action_templates/lib_template.yaml`](.github/action_templates/lib_template.yaml) (calls `libGlobalBuild.yml`).
- Scheduled quality checks: [`.github/action_templates/daily_quality_template.yaml`](.github/action_templates/daily_quality_template.yaml).

The war room's generated contract pages list every input, secret, job and condition.
[StroggForge Actions](./StroggForge%20Actions.md) is the long-form reference.

## Working on the war room

| Where | What |
|---|---|
| `war-room/ecosystem.toml` | Projects, domains, lifecycles, relationships, retired infrastructure, change digest |
| `war-room/plans.toml` | Releases and campaigns, with requirements and blockers |
| `war-room/toolchains.toml` | Platform and compiler policy, asserted against the workflows |
| `war-room/sources/` | Captured Cargo/workflow snapshots and the organization census |
| `.github/war-room-workflows.toml` | Workflow operating notes and supply-chain stages |
| `tools/war-room/` | The Rust generator and validator |
| `site/` | Zola site on the DreamWeave Mod Template docs shell |

```sh
npm ci
cargo run --locked -p stroggforge-war-room -- build
npm run diagrams
zola --root site serve
```

Maintenance loops (adding a project, retiring one, planning a release, changing a
workflow) are documented on the site under *Contributing → Maintain the war room*, and in
[`site/pages/contributing/maintenance.md`](site/pages/contributing/maintenance.md). The
source-of-truth contract is [`site/pages/engineering/architecture.md`](site/pages/engineering/architecture.md).

## Other notebooks

- [Release Planning](./Release%20Planning.md): the manual release checklist (partly historical; see the site's notebook index).
- [AUR team](./AUR-TEAM.md): AUR package maintainers.
- [DreamWeave Applications](./DreamWeave%20Applications.md) and `DreamWeave Map.drawio`/`.png`: historical, superseded by the generated project directory and maps.
